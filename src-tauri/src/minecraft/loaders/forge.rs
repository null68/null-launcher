use std::{error::Error, path::Path};

use tauri::AppHandle;

use crate::minecraft::loaders::installer_common;
use crate::minecraft::manifest::Version;

const METADATA_URL: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";

pub async fn list_versions(game_version: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let xml = reqwest::get(METADATA_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;

    let prefix = format!("{game_version}-");
    let mut versions: Vec<String> = xml
        .split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split("</version>").next())
        .filter(|v| v.starts_with(&prefix))
        .map(|v| v[prefix.len()..].to_string())
        .collect();
    versions.reverse(); // maven metadata lists oldest first
    Ok(versions)
}

pub async fn install(
    app: &AppHandle,
    minecraft_dir: &Path,
    base_version: &Version,
    forge_build: &str,
) -> Result<String, Box<dyn Error>> {
    let full = format!("{}-{}", base_version.id, forge_build);
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{full}/forge-{full}-installer.jar"
    );
    installer_common::run(app, minecraft_dir, &installer_url, base_version).await
}
