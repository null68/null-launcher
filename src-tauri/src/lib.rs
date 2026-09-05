mod launcher;
mod minecraft;

use std::str::FromStr;
use std::sync::Mutex;

use tauri::{AppHandle, Listener, Manager, State};

use crate::launcher::asset_orchestrator::install_version as install_version_impl;
use crate::launcher::instances::{list_instances as list_instances_impl, Instance};
use crate::launcher::runtime::launch_instance as launch_instance_impl;
use crate::launcher::screenshots::{list_screenshots as list_screenshots_impl, Screenshot};
use crate::launcher::settings::{
    get_settings as get_settings_impl, save_settings as save_settings_impl, LauncherSettings,
};
use crate::minecraft::loaders::{fabric, forge, neoforge, quilt, Loader};
use crate::minecraft::manifest::{get_versions as get_versions_impl, VersionManifest};

struct InstallLock(Mutex<Option<String>>);

fn try_start_install(state: &State<'_, InstallLock>, id: &str) -> Result<(), String> {
    let mut current = state.0.lock().unwrap();
    if let Some(running) = current.as_ref() {
        return Err(format!(
            "Already installing {running} - wait for it to finish before starting another install."
        ));
    }
    *current = Some(id.to_string());
    Ok(())
}

fn finish_install(state: &State<'_, InstallLock>) {
    *state.0.lock().unwrap() = None;
}

#[tauri::command]
async fn get_versions() -> Result<VersionManifest, String> {
    get_versions_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_version(
    app: AppHandle,
    state: State<'_, InstallLock>,
    version_id: String,
) -> Result<(), String> {
    try_start_install(&state, &version_id)?;

    let result = async {
        let manifest = get_versions_impl().await.map_err(|e| e.to_string())?;

        let version = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| format!("Version '{}' not found", version_id))?;

        install_version_impl(&app, version)
            .await
            .map_err(|e| e.to_string())
    }
    .await;

    finish_install(&state);
    result
}

#[tauri::command]
fn list_screenshots() -> Result<Vec<Screenshot>, String> {
    list_screenshots_impl().ok_or_else(|| "screenshots dir doesn't exist".to_string())
}

#[tauri::command]
async fn list_instances() -> Result<Vec<Instance>, String> {
    list_instances_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_instance(
    app: AppHandle,
    version_id: String,
    username: Option<String>,
    terminal_mode: Option<bool>,
    min_memory_mb: Option<u32>,
    max_memory_mb: Option<u32>,
) -> Result<u32, String> {
    let username = username
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "Player".to_string());

    let child = launch_instance_impl(
        &app,
        &version_id,
        &username,
        terminal_mode.unwrap_or(false),
        min_memory_mb,
        max_memory_mb,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(child.id())
}

#[tauri::command]
fn get_settings() -> Result<Option<LauncherSettings>, String> {
    get_settings_impl().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(settings: LauncherSettings) -> Result<(), String> {
    save_settings_impl(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_loader_versions(loader: String, game_version: String) -> Result<Vec<String>, String> {
    let loader = Loader::from_str(&loader)?;
    let result = match loader {
        Loader::Fabric => fabric::list_versions(&game_version).await,
        Loader::Quilt => quilt::list_versions(&game_version).await,
        Loader::Forge => forge::list_versions(&game_version).await,
        Loader::NeoForge => neoforge::list_versions(&game_version).await,
    };
    result.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_modded_instance(
    app: AppHandle,
    state: State<'_, InstallLock>,
    game_version_id: String,
    loader: String,
    loader_version: String,
) -> Result<String, String> {
    let loader = Loader::from_str(&loader)?;
    try_start_install(&state, &game_version_id)?;

    let result = async {
        let manifest = get_versions_impl().await.map_err(|e| e.to_string())?;
        let base_version = manifest
            .versions
            .iter()
            .find(|v| v.id == game_version_id)
            .ok_or_else(|| format!("Version '{}' not found", game_version_id))?;

        install_version_impl(&app, base_version)
            .await
            .map_err(|e| e.to_string())?;

        let minecraft_dir =
            crate::launcher::minecraft_dir::get_minecraft_dir().map_err(|e| e.to_string())?;

        match loader {
            Loader::Fabric => {
                fabric::install(&app, &minecraft_dir, &base_version.id, &loader_version).await
            }
            Loader::Quilt => {
                quilt::install(&app, &minecraft_dir, &base_version.id, &loader_version).await
            }
            Loader::Forge => {
                forge::install(&app, &minecraft_dir, base_version, &loader_version).await
            }
            Loader::NeoForge => {
                neoforge::install(&app, &minecraft_dir, base_version, &loader_version).await
            }
        }
        .map_err(|e| e.to_string())
    }
    .await;

    finish_install(&state);
    result
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(InstallLock(Mutex::new(None)))
        .setup(|app| {
            let handle = app.handle().clone();
            if let Some(window) = handle.get_webview_window("main") {
                let ready_window = window.clone();
                handle.once("app-ready", move |_event| {
                    let _ = ready_window.show();
                });

                // safety net
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let _ = window.show();
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_versions,
            install_version,
            list_screenshots,
            list_instances,
            launch_instance,
            list_loader_versions,
            install_modded_instance,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
