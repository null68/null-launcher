#[derive(Deserialize)]
pub struct VersionManifest {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
pub struct Version {
    id: String,
    url: String,
}

#[tauri::command]
pub async fn get_versions() -> Result<VersionManifest, Box<dyn std::error::Error>> {
    // todo
}
