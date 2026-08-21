use std::{error::Error, path::Path};

use crate::minecraft::{
    asset_index::install_assets, client_jar::install_client_jar,
    client_json::fetch_or_get_client_json, libraries::install_libraries, manifest::Version,
};

pub async fn install_version(version: &Version) -> Result<(), Box<dyn Error>> {
    let minecraft_dir = dirs::home_dir()
        .ok_or("failed to get home directory")?
        .join(".minecraft");

    let versions_dir = minecraft_dir.join("versions");

    let client_json = fetch_or_get_client_json(version, &versions_dir).await?;

    // install_client_jar(&client_json, version).await?;

    // install_libraries(&client_json).await?;

    install_assets(&client_json).await?;

    Ok(())
}
