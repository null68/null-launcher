use std::{collections::HashMap, error::Error, path::PathBuf};

use tauri::AppHandle;

use crate::{
    launcher::progress::Progress,
    minecraft::{
        asset_index::{fetch_asset_index, install_assets},
        client_jar::install_client_jar,
        client_json::fetch_or_get_client_json,
        libraries::{install_libraries, is_library_allowed},
        manifest::Version,
    },
};

pub async fn install_version(app: &AppHandle, version: &Version) -> Result<(), Box<dyn Error>> {
    let client_json = fetch_or_get_client_json(version, &PathBuf::from("versions")).await?;
    let asset_index_json = fetch_asset_index(&client_json).await?;

    let features = HashMap::new();
    let allowed_libraries: Vec<_> = client_json
        .libraries
        .iter()
        .filter(|l| is_library_allowed(&l.rules, &features))
        .filter_map(|l| l.downloads.artifact.as_ref())
        .collect();

    let total_bytes = client_json.downloads.client.size
        + allowed_libraries.iter().map(|a| a.size).sum::<u64>()
        + asset_index_json
            .as_ref()
            .map(|i| i.objects.values().map(|o| o.size).sum())
            .unwrap_or(0);

    let files_total = 1
        + allowed_libraries.len() as u64
        + asset_index_json
            .as_ref()
            .map(|i| i.objects.len() as u64)
            .unwrap_or(0);

    let mut progress = Progress::new(total_bytes, files_total);

    install_client_jar(&client_json, version, &mut progress, app).await?;
    install_libraries(&client_json, &mut progress, app).await?;
    if let Some(index) = &asset_index_json {
        install_assets(index, &mut progress, app).await?;
    }

    Ok(())
}
