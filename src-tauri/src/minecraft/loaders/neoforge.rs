use std::{error::Error, path::Path};

use tauri::AppHandle;

use crate::minecraft::loaders::installer_common;
use crate::minecraft::manifest::Version;

const METADATA_URL: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

// <mc_minor>.<mc_patch>.<build>
fn minor_patch_prefix(game_version: &str) -> Option<String> {
    let rest = game_version.strip_prefix("1.")?;
    let mut parts = rest.splitn(2, '.');
    let minor = parts.next()?;
    let patch = parts.next().unwrap_or("0");
    Some(format!("{minor}.{patch}."))
}

pub async fn list_versions(game_version: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let Some(prefix) = minor_patch_prefix(game_version) else {
        return Ok(vec![]);
    };
    let xml = reqwest::get(METADATA_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;

    let mut versions: Vec<String> = xml
        .split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split("</version>").next())
        .filter(|v| v.starts_with(&prefix))
        .map(|v| v.to_string())
        .collect();
    versions.reverse();
    Ok(versions)
}

pub async fn install(
    app: &AppHandle,
    minecraft_dir: &Path,
    base_version: &Version,
    neoforge_version: &str,
) -> Result<String, Box<dyn Error>> {
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{neoforge_version}/neoforge-{neoforge_version}-installer.jar"
    );
    installer_common::run(app, minecraft_dir, &installer_url, base_version).await
}
