use reqwest::{self, Client};
use serde::{self, Deserialize, Serialize};

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionManifest {
    versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    id: String,
    url: String,
    sha1: String,
    r#type: String,
}

pub static VERSIONS: Lazy<Mutex<Option<VersionManifest>>> = Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub fn get_versions() -> Option<VersionManifest> {
    let versions = VERSIONS.lock().unwrap();

    versions.clone()
}

pub async fn fetch_versions() -> Result<VersionManifest, Box<dyn std::error::Error>> {
    {
        let versions = VERSIONS.lock().unwrap();

        if let Some(manifest) = versions.as_ref() {
            return Ok(manifest.clone());
        }
    }
    let client = Client::new();
    let res = client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await?
        .error_for_status()?;

    let version_manifest: VersionManifest = res.json::<VersionManifest>().await?;
    {
        let mut versions = VERSIONS.lock().unwrap();

        *versions = Some(version_manifest.clone());
    }

    Ok(version_manifest)
}
