use serde::{Deserialize, Serialize};

use crate::launcher::minecraft_dir::get_minecraft_dir;
use crate::minecraft::client_json::resolve_client_json;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub r#type: String,
    pub installed_at: Option<String>,
    pub loader: Option<String>,
}

pub async fn list_instances() -> Result<Vec<Instance>, String> {
    let minecraft_dir = get_minecraft_dir().map_err(|e| e.to_string())?;
    let versions_dir = minecraft_dir.join("versions");

    if !versions_dir.exists() {
        return Ok(vec![]);
    }

    let mut instances = Vec::new();

    for entry in std::fs::read_dir(versions_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let version_id = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid UTF-8 in version id".to_string())?;

        // if its mid install or somethings off just skip it, dont fuck up the whole list
        let Ok(client_json) = resolve_client_json(&minecraft_dir, &version_id).await else {
            continue;
        };

        instances.push(Instance {
            id: version_id,
            r#type: client_json
                .r#type
                .clone()
                .unwrap_or_else(|| "release".to_string()),
            installed_at: None,
            loader: crate::minecraft::client_json::detect_loader(&client_json)
                .map(|l| l.to_string()),
        });
    }

    Ok(instances)
}
