use md5::Md5;
use sha1::Digest;
use std::fs::{self, File};
use std::process::{Child, Command, Stdio};
use std::{collections::HashMap, error::Error, io, path::Path};

use crate::launcher::minecraft_dir::get_minecraft_dir;
use crate::minecraft::client_json::fetch_or_get_client_json;
use crate::minecraft::manifest::Version;
use crate::minecraft::{
    client_json::{Argument, ArgumentValue, ClientJson},
    libraries::is_library_allowed,
};

#[cfg(target_os = "windows")]
pub fn find_java() -> Option<String> {
    use std::process::Command;

    let output = Command::new("cmd")
        .args(["/C", "where java"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();

    Some(path)
}

#[cfg(target_os = "linux")]
pub fn find_java() -> Option<String> {
    use std::process::Command;

    let output = Command::new("which").arg("java").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();

    Some(path)
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
            .classifiers
            .as_ref()
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

pub async fn launch_instance(version: &Version, username: &str) -> Result<Child, Box<dyn Error>> {
    let minecraft_dir = get_minecraft_dir()?;
    let client_json = fetch_or_get_client_json(version).await?;
    let java = find_java().ok_or("java not found on PATH")?;
    let features: HashMap<String, bool> = HashMap::new();

    let natives_dir = minecraft_dir
        .join("versions")
        .join(&version.id)
        .join(format!("{}-natives", version.id));
    extract_natives(&client_json, &natives_dir, &minecraft_dir, &features)?;

    let mut classpath_entries: Vec<String> = client_json
        .libraries
        .iter()
        .filter(|l| is_library_allowed(&l.rules, &features))
        .filter_map(|l| l.downloads.artifact.as_ref())
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
            .join(&version.id)
            .join(format!("{}.jar", version.id))
            .display()
            .to_string(),
    );
    let classpath = classpath_entries.join(if cfg!(windows) { ";" } else { ":" });

    let asset_index_id = client_json
        .asset_index
        .as_ref()
        .map(|a| a.id.clone())
        .unwrap_or_else(|| version.id.clone());

    let mut values: HashMap<&str, String> = HashMap::new();
    values.insert("auth_player_name", username.to_string());
    values.insert("version_name", version.id.clone());
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
    values.insert("version_type", version.r#type.clone());
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

    let main_class = client_json
        .main_class
        .clone()
        .ok_or("client json has no mainClass")?;

    let child = Command::new(&java)
        .current_dir(&minecraft_dir)
        .args(&jvm_args)
        .arg(&main_class)
        .args(&game_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}
