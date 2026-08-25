use serde::{self, Deserialize, Serialize};
use std::fs;

use crate::launcher::minecraft_dir::get_minecraft_dir;

#[derive(Serialize, Deserialize)]
pub struct Screenshot {
    pub name: String,
    pub path: String,
}

pub fn list_screenshots() -> Option<Vec<Screenshot>> {
    let minecraft_dir = get_minecraft_dir().ok();
    let screenshots_dir = minecraft_dir?.join("screenshots");

    let paths = fs::read_dir(screenshots_dir).ok()?;

    let screenshots = paths
        .filter_map(Result::ok)
        .map(|path| Screenshot {
            name: path.file_name().to_string_lossy().into_owned(),
            path: path.path().to_string_lossy().into_owned(),
        })
        .collect();

    Some(screenshots)
}
