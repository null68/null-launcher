mod launcher;
mod minecraft;

use tauri::AppHandle;

use crate::launcher::asset_orchestrator::install_version as install_version_impl;
use crate::launcher::instances::{list_instances as list_instances_impl, Instance};
use crate::launcher::runtime::launch_instance as launch_instance_impl;
use crate::launcher::screenshots::{list_screenshots as list_screenshots_impl, Screenshot};
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

#[tauri::command]
fn list_screenshots() -> Result<Vec<Screenshot>, String> {
    list_screenshots_impl().ok_or_else(|| "screenshots dir doesn't exist".to_string())
}

#[tauri::command]
fn list_instances() -> Result<Vec<Instance>, String> {
    list_instances_impl().map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_instance(version_id: String, username: String) -> Result<u32, String> {
    let manifest = get_versions_impl().await.map_err(|e| e.to_string())?;

    let version = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version '{}' not found", version_id))?;

    let child = launch_instance_impl(version, &username)
        .await
        .map_err(|e| e.to_string())?;

    Ok(child.id())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_versions,
            install_version,
            list_screenshots,
            list_instances,
            launch_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
