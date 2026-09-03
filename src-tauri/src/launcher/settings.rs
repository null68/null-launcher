use std::{error::Error, fs};

use serde::{Deserialize, Serialize};

use crate::launcher::minecraft_dir::get_minecraft_dir;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSettings {
    pub username: String,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub terminal_mode: bool,
}

fn settings_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    Ok(get_minecraft_dir()?.join("launcher_settings.json"))
}

pub fn get_settings() -> Result<Option<LauncherSettings>, Box<dyn Error>> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

pub fn save_settings(settings: &LauncherSettings) -> Result<(), Box<dyn Error>> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(settings)?)?;
    Ok(())
}
