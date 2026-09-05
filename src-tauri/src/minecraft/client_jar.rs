use std::{error::Error, path::PathBuf};

use tauri::AppHandle;

use crate::{
    launcher::{download::DownloadObject, progress::Progress},
    minecraft::client_json::ClientJson,
};

pub async fn install_client_jar(
    client_json: &ClientJson,
    version_id: &str,
    progress: &mut Progress,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    let client = &client_json
        .downloads
        .as_ref()
        .ok_or("client json has no downloads block")?
        .client;

    let download_obj = DownloadObject {
        url: client.url.clone(),
        sha1: client.sha1.clone().into(),
        size: Some(client.size),
        file_path: PathBuf::from(format!("versions/{version_id}/{version_id}.jar")),
    };

    download_obj
        .download_file(|n| progress.add_bytes(app, n))
        .await?;
    progress.finish_file(app);

    Ok(())
}
