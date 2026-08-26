use serde::{Deserialize, Serialize};

use crate::launcher::minecraft_dir::get_minecraft_dir;

#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub r#type: String,
    pub installed_at: Option<String>,
}

pub fn list_instances() -> Result<Vec<Instance>, String> {
    let minecraft_dir = get_minecraft_dir().map_err(|e| e.to_string())?;
    let versions_dir = minecraft_dir.join("versions");

    if !versions_dir.exists() {
        return Ok(vec![]);
    }

    let mut instances = Vec::new();

    for entry in std::fs::read_dir(versions_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            let version_id = entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid UTF-8 in version id".to_string())?;
            // todo: type should be based on version type
            let instance = Instance {
                id: version_id.clone(),
                r#type: "release".to_string(),
                installed_at: None,
            };
            instances.push(instance);
        }
    }

    Ok(instances)
}
