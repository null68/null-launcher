mod launcher;
mod minecraft;

use tauri::AppHandle;

use crate::launcher::asset_orchestrator::install_version as install_version_impl;
use crate::minecraft::manifest::{get_versions as get_versions_impl, VersionManifest};

#[tauri::command]
async fn get_versions() -> Result<VersionManifest, String> {
    get_versions_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_version(app: AppHandle, version_id: String) -> Result<(), String> {
    let manifest = get_versions_impl().await.map_err(|e| e.to_string())?;

    let version = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version '{}' not found", version_id))?;

    install_version_impl(&app, version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_versions, install_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
