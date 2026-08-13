use crate::{launcher::download::DownloadObject, minecraft::client_json::ClientJson};
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::{error::Error, fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct AssetIndexJson {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}
pub async fn install_assets(client_json: &ClientJson) -> Result<(), Box<dyn Error>> {
    let Some(asset_index) = &client_json.asset_index else {
        return Ok(());
    };
    let index_path = PathBuf::from(format!("assets/indexes/{}.json", asset_index.id));
    let download_object = DownloadObject {
        url: asset_index.url.clone(),
        size: Some(asset_index.size),
        sha1: Some(asset_index.sha1.clone()),
        file_path: index_path.clone(),
    };

    download_object.download_file().await?;

    let bytes = fs::read(&index_path)?;

    let asset_index_json = serde_json::from_slice::<AssetIndexJson>(&bytes)?;
    for (_, object) in asset_index_json.objects {
        let hash = object.hash;
        let size = object.size;

        let hash_prefix = &hash[0..2];
        let asset_path = PathBuf::from(format!("assets/objects/{}/{}", hash_prefix, hash));

        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            hash_prefix, hash
        );

        let asset_download_object = DownloadObject {
            url: url,
            size: Some(size),
            sha1: Some(hash),
            file_path: asset_path,
        };

        asset_download_object.download_file().await?;
    }
    Ok(())
}
