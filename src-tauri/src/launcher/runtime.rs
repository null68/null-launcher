use md5::Md5;
use sha1::Digest;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::launcher::asset_orchestrator::verify_instance_files;
use crate::launcher::download::DownloadObject;
use crate::launcher::minecraft_dir::get_minecraft_dir;
use crate::minecraft::client_json::{read_client_json_from_disk, resolve_client_json};
use crate::minecraft::java_runtime;
use crate::minecraft::maven::resolve_library_artifact;
use crate::minecraft::{
    client_json::{Argument, ArgumentValue, ClientJson},
    libraries::is_library_allowed,
};

const TERMINAL_WINDOW_LABEL: &str = "terminal";

#[cfg(target_os = "windows")]
fn find_java_on_path() -> Option<String> {
    let output = Command::new("cmd")
        .args(["/C", "where java"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.trim().to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn find_java_on_path() -> Option<String> {
    let output = Command::new("which").arg("java").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.trim().to_string())
}

fn java_home_candidate() -> Option<String> {
    let home = std::env::var("JAVA_HOME").ok()?;
    let bin_name = if cfg!(windows) { "java.exe" } else { "java" };
    let path = Path::new(&home).join("bin").join(bin_name);
    path.exists().then(|| path.display().to_string())
}

fn bundled_java_candidate(minecraft_dir: &Path, component: &str) -> Option<String> {
    let bin_name = if cfg!(windows) { "javaw.exe" } else { "java" };
    let path = minecraft_dir
        .join("runtime")
        .join(component)
        .join("bin")
        .join(bin_name);
    path.exists().then(|| path.display().to_string())
}

fn java_major_version(java_path: &str) -> Option<u32> {
    let output = Command::new(java_path).arg("-version").output().ok()?;
    parse_java_major_version(&String::from_utf8_lossy(&output.stderr))
}

fn parse_java_major_version(banner: &str) -> Option<u32> {
    let after = banner.split("version \"").nth(1)?;
    let version_str = after.split('"').next()?;
    let version_str = version_str.strip_prefix("1.").unwrap_or(version_str);
    let digits: String = version_str
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

pub async fn find_compatible_java(
    app: &AppHandle,
    minecraft_dir: &Path,
    component: &str,
    required_major: u32,
) -> Result<String, String> {
    let candidates = [
        bundled_java_candidate(minecraft_dir, component),
        java_home_candidate(),
        find_java_on_path(),
    ];

    for candidate in candidates.into_iter().flatten() {
        if java_major_version(&candidate) == Some(required_major) {
            return Ok(candidate);
        }
    }

    // Nothing already on this machine matches - fetch Mojang's own build for this
    // component (same as the official launcher does), so every Minecraft version
    // gets the Java release it actually needs without the user juggling installs.
    java_runtime::ensure_java_runtime(minecraft_dir, component, app)
        .await
        .map(|path| path.display().to_string())
        .map_err(|e| {
            format!(
                "This version needs Java {required_major}, and null-launcher couldn't fetch a \
                 matching runtime automatically ({e}). Install Java {required_major} yourself and \
                 make sure it's on PATH, or point JAVA_HOME at it."
            )
        })
}

fn open_terminal_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(TERMINAL_WINDOW_LABEL) {
        let _ = window.set_focus();
        let _ = app.emit_to(
            TERMINAL_WINDOW_LABEL,
            "mc-log",
            "\n── new launch ──\n".to_string(),
        );
        return;
    }

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
    app.once("terminal-ready", move |_event| {
        let _ = ready_tx.send(());
    });

    let built = WebviewWindowBuilder::new(
        app,
        TERMINAL_WINDOW_LABEL,
        WebviewUrl::App("index.html#terminal".into()),
    )
    .title("null-launcher — Console")
    .inner_size(760.0, 460.0)
    .build();

    if built.is_ok() {
        let _ = ready_rx.recv_timeout(Duration::from_secs(3));
    }
}

fn stream_to_terminal(
    app: AppHandle,
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
) {
    if let Some(stdout) = stdout {
        let app = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                let _ = app.emit_to(TERMINAL_WINDOW_LABEL, "mc-log", line);
            }
        });
    }
    if let Some(stderr) = stderr {
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().flatten() {
                let _ = app.emit_to(TERMINAL_WINDOW_LABEL, "mc-log", line);
            }
        });
    }
}

fn offline_uuid(username: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", username).as_bytes());
    let mut bytes: [u8; 16] = hasher.finalize().into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex = hex::encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn substitute(input: &str, values: &HashMap<&str, String>) -> String {
    let mut out = input.to_string();
    for (key, value) in values {
        out = out.replace(&format!("${{{}}}", key), value);
    }
    out
}

fn resolve_argument(
    arg: &Argument,
    features: &HashMap<String, bool>,
    values: &HashMap<&str, String>,
) -> Vec<String> {
    match arg {
        Argument::Simple(s) => vec![substitute(s, values)],
        Argument::Conditional { rules, value } => {
            if !is_library_allowed(&Some(rules.clone()), features) {
                return vec![];
            }
            match value {
                ArgumentValue::Single(s) => vec![substitute(s, values)],
                ArgumentValue::Multiple(list) => {
                    list.iter().map(|s| substitute(s, values)).collect()
                }
            }
        }
    }
}

fn extract_natives(
    client_json: &ClientJson,
    natives_dir: &Path,
    minecraft_dir: &Path,
    features: &HashMap<String, bool>,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(natives_dir)?;

    let os_key = match std::env::consts::OS {
        "macos" => "osx",
        _ => std::env::consts::OS,
    };

    for library in &client_json.libraries {
        if !is_library_allowed(&library.rules, features) {
            continue;
        }
        let Some(classifier_key) = library.natives.as_ref().and_then(|n| n.get(os_key)) else {
            continue;
        };
        let Some(artifact) = library
            .downloads
            .as_ref()
            .and_then(|d| d.classifiers.as_ref())
            .and_then(|c| c.get(classifier_key))
        else {
            continue;
        };

        let jar_path = minecraft_dir.join("libraries").join(&artifact.path);
        let file = File::open(&jar_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let exclude = library.extract.as_ref().and_then(|e| e.exclude.as_ref());

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();
            if entry.is_dir() {
                continue;
            }
            if let Some(exclude) = exclude {
                if exclude
                    .iter()
                    .any(|prefix| name.starts_with(prefix.as_str()))
                {
                    continue;
                }
            }
            let out_path = natives_dir.join(&name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
        }
    }

    Ok(())
}

pub async fn launch_instance(
    app: &AppHandle,
    version_id: &str,
    username: &str,
    terminal_mode: bool,
    min_memory_mb: Option<u32>,
    max_memory_mb: Option<u32>,
) -> Result<Child, Box<dyn Error>> {
    let minecraft_dir = get_minecraft_dir()?;
    // use vanilla jars for modded shit
    let base_id = read_client_json_from_disk(&minecraft_dir, version_id)?
        .inherits_from
        .unwrap_or_else(|| version_id.to_string());
    let client_json = resolve_client_json(&minecraft_dir, version_id).await?;

    let _ = app.emit("launch-status", "Checking game files…");
    verify_instance_files(app, &client_json, &base_id).await?;

    let required_major = client_json
        .java_version
        .as_ref()
        .map(|j| j.major_version)
        .unwrap_or(8);
    let component = client_json
        .java_version
        .as_ref()
        .map(|j| j.component.as_str())
        .unwrap_or("jre-legacy");
    let _ = app.emit("launch-status", "Preparing Java…");
    let java = find_compatible_java(app, &minecraft_dir, component, required_major).await?;

    let features: HashMap<String, bool> = HashMap::new();

    let natives_dir = minecraft_dir
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}-natives"));
    extract_natives(&client_json, &natives_dir, &minecraft_dir, &features)?;

    let mut classpath_entries: Vec<String> = client_json
        .libraries
        .iter()
        .filter(|l| is_library_allowed(&l.rules, &features))
        .filter_map(resolve_library_artifact)
        .map(|a| {
            minecraft_dir
                .join("libraries")
                .join(&a.path)
                .display()
                .to_string()
        })
        .collect();
    classpath_entries.push(
        minecraft_dir
            .join("versions")
            .join(&base_id)
            .join(format!("{base_id}.jar"))
            .display()
            .to_string(),
    );
    let classpath = classpath_entries.join(if cfg!(windows) { ";" } else { ":" });

    let asset_index_id = client_json
        .asset_index
        .as_ref()
        .map(|a| a.id.clone())
        .unwrap_or_else(|| version_id.to_string());
    let version_type = client_json
        .r#type
        .clone()
        .unwrap_or_else(|| "release".to_string());

    let mut values: HashMap<&str, String> = HashMap::new();
    values.insert("auth_player_name", username.to_string());
    values.insert("version_name", version_id.to_string());
    values.insert("game_directory", minecraft_dir.display().to_string());
    values.insert(
        "assets_root",
        minecraft_dir.join("assets").display().to_string(),
    );
    values.insert("assets_index_name", asset_index_id);
    values.insert("auth_uuid", offline_uuid(username));
    values.insert("auth_access_token", "0".to_string());
    values.insert("user_type", "legacy".to_string());
    values.insert("user_properties", "{}".to_string());
    values.insert("version_type", version_type);
    values.insert("natives_directory", natives_dir.display().to_string());
    values.insert("launcher_name", "null-launcher".to_string());
    values.insert("launcher_version", env!("CARGO_PKG_VERSION").to_string());
    values.insert("classpath", classpath.clone());

    let mut jvm_args = Vec::new();
    let mut game_args = Vec::new();

    if let Some(arguments) = &client_json.arguments {
        for arg in arguments.jvm.as_deref().unwrap_or_default() {
            jvm_args.extend(resolve_argument(arg, &features, &values));
        }
        for arg in arguments.game.as_deref().unwrap_or_default() {
            game_args.extend(resolve_argument(arg, &features, &values));
        }
    } else {
        jvm_args.push(format!("-Djava.library.path={}", natives_dir.display()));
        jvm_args.push("-cp".to_string());
        jvm_args.push(classpath);
        if let Some(legacy) = &client_json.minecraft_arguments {
            game_args.extend(
                legacy
                    .split_whitespace()
                    .map(|part| substitute(part, &values)),
            );
        }
    }

    if let Some(logging) = &client_json.logging {
        let relative_path = PathBuf::from("assets")
            .join("log_configs")
            .join(&logging.client.file.id);
        let download = DownloadObject {
            url: logging.client.file.url.clone(),
            size: Some(logging.client.file.size),
            sha1: Some(logging.client.file.sha1.clone()),
            file_path: relative_path.clone(),
        };
        download.download_file(|_| {}).await?;

        let mut log_values: HashMap<&str, String> = HashMap::new();
        log_values.insert(
            "path",
            minecraft_dir.join(&relative_path).display().to_string(),
        );
        jvm_args.push(substitute(&logging.client.argument, &log_values));
    }

    if let Some(min) = min_memory_mb {
        jvm_args.push(format!("-Xms{min}M"));
    }
    if let Some(max) = max_memory_mb {
        jvm_args.push(format!("-Xmx{max}M"));
    }

    let main_class = client_json
        .main_class
        .clone()
        .ok_or("client json has no mainClass")?;

    let mut command = Command::new(&java);
    command
        .current_dir(&minecraft_dir)
        .args(&jvm_args)
        .arg(&main_class)
        .args(&game_args);

    let _ = app.emit("launch-status", "Starting Minecraft…");

    let child = if terminal_mode {
        open_terminal_window(app);
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        stream_to_terminal(app.clone(), child.stdout.take(), child.stderr.take());
        child
    } else {
        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
    };

    Ok(child)
}
