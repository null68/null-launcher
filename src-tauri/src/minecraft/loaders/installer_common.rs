use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{Read, Seek},
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use tauri::AppHandle;

use crate::launcher::progress::Progress;
use crate::launcher::runtime::find_compatible_java;
use crate::minecraft::client_json::{
    fetch_or_get_client_json, write_client_json, ClientJson, Library,
};
use crate::minecraft::manifest::Version;
use crate::minecraft::maven::{coords_to_path, download_libraries, resolve_library_artifact};

#[derive(Deserialize, Debug)]
struct InstallProfile {
    version: String,
    json: String,
    #[serde(default)]
    libraries: Vec<Library>,
    #[serde(default)]
    processors: Vec<Processor>,
    #[serde(default)]
    data: HashMap<String, DataEntry>,
}

#[derive(Deserialize, Debug)]
struct DataEntry {
    client: String,
}

#[derive(Deserialize, Debug)]
struct Processor {
    sides: Option<Vec<String>>,
    jar: String,
    #[serde(default)]
    classpath: Vec<String>,
    #[serde(default)]
    args: Vec<String>,
}

fn read_zip_entry<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| format!("'{name}' not found in installer jar: {e}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn read_json_entry<T: serde::de::DeserializeOwned, R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<T, Box<dyn Error>> {
    let bytes = read_zip_entry(archive, name)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn read_main_class(jar_path: &Path) -> Result<String, Box<dyn Error>> {
    let file =
        fs::File::open(jar_path).map_err(|e| format!("can't open {}: {e}", jar_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut manifest = String::new();
    archive
        .by_name("META-INF/MANIFEST.MF")?
        .read_to_string(&mut manifest)?;
    for line in manifest.lines() {
        if let Some(value) = line.strip_prefix("Main-Class:") {
            return Ok(value.trim().to_string());
        }
    }
    Err(format!("no Main-Class attribute in {}", jar_path.display()).into())
}

fn substitute_arg(arg: &str, datamap: &HashMap<String, String>, minecraft_dir: &Path) -> String {
    if let Some(key) = arg.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        if let Some(value) = datamap.get(key) {
            return value.clone();
        }
    }
    if let Some(coords) = arg.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        if let Some(path) = coords_to_path(coords) {
            return minecraft_dir
                .join("libraries")
                .join(path)
                .display()
                .to_string();
        }
    }
    arg.to_string()
}

// base version has to already be installed before this
pub async fn run(
    app: &AppHandle,
    minecraft_dir: &Path,
    installer_url: &str,
    base_version: &Version,
) -> Result<String, Box<dyn Error>> {
    let installer_bytes = reqwest::get(installer_url)
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("null-launcher-installer-{unique}"));
    fs::create_dir_all(&temp_dir)?;
    let installer_jar_path = temp_dir.join("installer.jar");
    fs::write(&installer_jar_path, &installer_bytes)?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(installer_bytes))?;

    let profile: InstallProfile = read_json_entry(&mut archive, "install_profile.json")?;
    let child_json: ClientJson =
        read_json_entry(&mut archive, profile.json.trim_start_matches('/'))?;

    let instance_id = child_json
        .id
        .clone()
        .unwrap_or_else(|| profile.version.clone());
    write_client_json(minecraft_dir, &instance_id, &child_json)?;

    let all_libraries: Vec<Library> = profile
        .libraries
        .iter()
        .chain(child_json.libraries.iter())
        .cloned()
        .collect();
    let total_bytes: u64 = all_libraries
        .iter()
        .filter_map(resolve_library_artifact)
        .filter_map(|a| a.size)
        .sum();
    let mut progress = Progress::new(total_bytes, all_libraries.len() as u64);
    download_libraries(&all_libraries, &mut progress, app).await?;

    let mut datamap: HashMap<String, String> = HashMap::new();
    for (key, entry) in &profile.data {
        let raw = &entry.client;
        let resolved = if let Some(coords) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
        {
            let path = coords_to_path(coords)
                .ok_or_else(|| format!("bad maven coordinate in data.{key}: {coords}"))?;
            minecraft_dir
                .join("libraries")
                .join(path)
                .display()
                .to_string()
        } else if let Some(quoted) = raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            quoted.to_string()
        } else {
            let inner = raw.trim_start_matches('/');
            let out_path = temp_dir.join(inner);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let bytes = read_zip_entry(&mut archive, inner)?;
            fs::write(&out_path, bytes)?;
            out_path.display().to_string()
        };
        datamap.insert(key.clone(), resolved);
    }

    let vanilla_jar = minecraft_dir
        .join("versions")
        .join(&base_version.id)
        .join(format!("{}.jar", base_version.id));

    datamap.insert("SIDE".into(), "client".into());
    datamap.insert("MINECRAFT_JAR".into(), vanilla_jar.display().to_string());
    datamap.insert("MINECRAFT_VERSION".into(), base_version.id.clone());
    datamap.insert("ROOT".into(), minecraft_dir.display().to_string());
    datamap.insert(
        "LIBRARY_DIR".into(),
        minecraft_dir.join("libraries").display().to_string(),
    );
    datamap.insert("INSTALLER".into(), installer_jar_path.display().to_string());

    let base_client_json = fetch_or_get_client_json(base_version).await?;
    let required_major = base_client_json
        .java_version
        .as_ref()
        .map(|j| j.major_version)
        .unwrap_or(8);
    let component = base_client_json
        .java_version
        .as_ref()
        .map(|j| j.component.as_str())
        .unwrap_or("jre-legacy");
    let java = find_compatible_java(app, minecraft_dir, component, required_major)
        .await
        .map_err(|e| format!("can't run the {instance_id} installer: {e}"))?;

    let classpath_sep = if cfg!(windows) { ";" } else { ":" };

    for processor in &profile.processors {
        if let Some(sides) = &processor.sides {
            if !sides.iter().any(|s| s == "client") {
                continue;
            }
        }

        let classpath = processor
            .classpath
            .iter()
            .filter_map(|c| coords_to_path(c))
            .map(|p| {
                minecraft_dir
                    .join("libraries")
                    .join(p)
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(classpath_sep);

        let jar_path = minecraft_dir.join("libraries").join(
            coords_to_path(&processor.jar)
                .ok_or_else(|| format!("bad processor jar coordinate: {}", processor.jar))?,
        );
        let main_class = read_main_class(&jar_path)?;

        let args: Vec<String> = processor
            .args
            .iter()
            .map(|a| substitute_arg(a, &datamap, minecraft_dir))
            .collect();

        let status = Command::new(&java)
            .arg("-cp")
            .arg(&classpath)
            .arg(&main_class)
            .args(&args)
            .status()
            .map_err(|e| format!("failed to run installer step {main_class}: {e}"))?;

        if !status.success() {
            return Err(format!(
                "{instance_id} installer step {main_class} failed (exit {status})"
            )
            .into());
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(instance_id)
}
