use std::{error::Error, path::Path};

use serde::Deserialize;
use tauri::AppHandle;

use crate::launcher::progress::Progress;
use crate::minecraft::client_json::{write_client_json, ClientJson};
use crate::minecraft::maven::{download_libraries, resolve_library_artifact};

const META_BASE: &str = "https://meta.quiltmc.org/v3";
const MAVEN_BASE: &str = "https://maven.quiltmc.org/repository/release/";

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
}

pub async fn list_versions(game_version: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!("{META_BASE}/versions/loader/{}", url_part(game_version));
    let entries: Vec<LoaderEntry> = reqwest::get(&url).await?.error_for_status()?.json().await?;
    Ok(entries.into_iter().map(|e| e.loader.version).collect())
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
        .ok_or("quilt profile response has no id")?;
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
