mod minecraft;

use crate::minecraft::manifest::{fetch_versions, get_versions};

#[tauri::command]
async fn initialize_launcher() -> Result<(), String> {
    fetch_versions().await.map_err(|e| e.to_string())?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(async {
                initialize_launcher().await.unwrap();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
