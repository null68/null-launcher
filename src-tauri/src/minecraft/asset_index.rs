use crate::launcher::minecraft_dir::get_minecraft_dir;
use crate::launcher::progress::Progress;
use crate::{launcher::download::DownloadObject, minecraft::client_json::ClientJson};
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::{error::Error, fs, path::PathBuf};
use tauri::AppHandle;

#[derive(Serialize, Deserialize)]
pub struct AssetIndexJson {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub async fn fetch_asset_index(
    client_json: &ClientJson,
) -> Result<Option<AssetIndexJson>, Box<dyn Error>> {
    let Some(asset_index) = &client_json.asset_index else {
        return Ok(None);
    };
    let index_path = PathBuf::from(format!("assets/indexes/{}.json", asset_index.id));
    let download_object = DownloadObject {
        url: asset_index.url.clone(),
        size: Some(asset_index.size),
        sha1: Some(asset_index.sha1.clone()),
        file_path: index_path.clone(),
    };

    download_object.download_file(|_| {}).await?;

    let bytes = fs::read(get_minecraft_dir()?.join(&index_path))?;
    Ok(Some(serde_json::from_slice::<AssetIndexJson>(&bytes)?))
}

pub async fn install_assets(
    asset_index_json: &AssetIndexJson,
    progress: &mut Progress,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    for (_, object) in &asset_index_json.objects {
        let hash = &object.hash;
        let size = object.size;
        let hash_prefix = &hash[0..2];

        let asset_download_object = DownloadObject {
            url: format!(
                "https://resources.download.minecraft.net/{}/{}",
                hash_prefix, hash
            ),
            size: Some(size),
            sha1: Some(hash.clone()),
            file_path: PathBuf::from(format!("assets/objects/{}/{}", hash_prefix, hash)),
        };

        asset_download_object.download_file(|_| {}).await?;
        progress.add_file(app, size);
    }
    Ok(())
}
