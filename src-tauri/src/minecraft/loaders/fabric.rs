use std::{error::Error, path::Path};

use serde::Deserialize;
use tauri::AppHandle;

use crate::launcher::progress::Progress;
use crate::minecraft::client_json::{write_client_json, ClientJson};
use crate::minecraft::maven::{download_libraries, resolve_library_artifact};

const META_BASE: &str = "https://meta.fabricmc.net/v2";
const MAVEN_BASE: &str = "https://maven.fabricmc.net/";

fn url_part(s: &str) -> String {
    s.replace(' ', "%20")
}

#[derive(Deserialize)]
struct LoaderEntry {
    loader: LoaderInfo,
}

#[derive(Deserialize)]
struct LoaderInfo {
    version: String,
    stable: bool,
}

pub async fn list_versions(game_version: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!("{META_BASE}/versions/loader/{}", url_part(game_version));
    let entries: Vec<LoaderEntry> = reqwest::get(&url).await?.error_for_status()?.json().await?;

    let mut stable: Vec<String> = Vec::new();
    let mut unstable: Vec<String> = Vec::new();
    for entry in entries {
        if entry.loader.stable {
            stable.push(entry.loader.version);
        } else {
            unstable.push(entry.loader.version);
        }
    }
    stable.extend(unstable);
    Ok(stable)
}

pub async fn install(
    app: &AppHandle,
    minecraft_dir: &Path,
    game_version_id: &str,
    loader_version: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "{META_BASE}/versions/loader/{}/{}/profile/json",
        url_part(game_version_id),
        url_part(loader_version)
    );
    let mut child_json: ClientJson = reqwest::get(&url).await?.error_for_status()?.json().await?;

    for library in &mut child_json.libraries {
        if library.downloads.is_none() && library.url.is_none() {
            library.url = Some(MAVEN_BASE.to_string());
        }
    }

    let instance_id = child_json
        .id
        .clone()
        .ok_or("fabric profile response has no id")?;
    write_client_json(minecraft_dir, &instance_id, &child_json)?;

    let total_bytes: u64 = child_json
        .libraries
        .iter()
        .filter_map(resolve_library_artifact)
        .filter_map(|a| a.size)
        .sum();
    let mut progress = Progress::new(total_bytes, child_json.libraries.len() as u64);
    download_libraries(&child_json.libraries, &mut progress, app).await?;

    Ok(instance_id)
}
