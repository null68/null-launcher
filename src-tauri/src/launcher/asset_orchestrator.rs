use std::{collections::HashMap, error::Error};

use tauri::AppHandle;

use crate::{
    launcher::progress::Progress,
    minecraft::{
        asset_index::{fetch_asset_index, install_assets},
        client_jar::install_client_jar,
        client_json::{fetch_or_get_client_json, ClientJson},
        libraries::{install_libraries, is_library_allowed},
        manifest::Version,
        maven::resolve_library_artifact,
    },
};

async fn download_instance_files(
    app: &AppHandle,
    client_json: &ClientJson,
    version_id: &str,
    event: &'static str,
) -> Result<(), Box<dyn Error>> {
    let downloads = client_json
        .downloads
        .as_ref()
        .ok_or("client json has no downloads block")?;
    let asset_index_json = fetch_asset_index(client_json).await?;

    let features = HashMap::new();
    let allowed_libraries: Vec<_> = client_json
        .libraries
        .iter()
        .filter(|l| is_library_allowed(&l.rules, &features))
        .filter_map(resolve_library_artifact)
        .collect();

    let total_bytes = downloads.client.size
        + allowed_libraries.iter().filter_map(|a| a.size).sum::<u64>()
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

    let mut progress = Progress::with_event(total_bytes, files_total, event);
    install_client_jar(client_json, version_id, &mut progress, app).await?;
    install_libraries(client_json, &mut progress, app).await?;
    if let Some(index) = &asset_index_json {
        install_assets(index, &mut progress, app).await?;
    }

    Ok(())
}

pub async fn install_version(app: &AppHandle, version: &Version) -> Result<(), Box<dyn Error>> {
    let client_json = fetch_or_get_client_json(version).await?;
    download_instance_files(app, &client_json, &version.id, "install-progress").await
}

pub async fn verify_instance_files(
    app: &AppHandle,
    client_json: &ClientJson,
    base_version_id: &str,
) -> Result<(), Box<dyn Error>> {
    download_instance_files(app, client_json, base_version_id, "launch-progress").await
}
