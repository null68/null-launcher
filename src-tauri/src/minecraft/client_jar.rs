use std::{error::Error, path::PathBuf};

use crate::{
    launcher::download::DownloadObject,
    minecraft::{client_json::ClientJson, manifest::Version},
};

pub async fn install_client_jar(
    client_json: &ClientJson,
    version: &Version,
) -> Result<(), Box<dyn Error>> {
    let client = &client_json.downloads.client;

    let download_obj = DownloadObject {
        url: client.url.clone(),
        sha1: client.sha1.clone().into(),
        size: Some(client.size),
        file_path: PathBuf::from(format!("versions/{}/{}.jar", version.id, version.id)),
    };

    download_obj.download_file().await?;

    Ok(())
}
