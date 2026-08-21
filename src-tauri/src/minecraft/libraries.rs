use tauri::AppHandle;

use crate::launcher::download::DownloadObject;
use crate::launcher::progress::Progress;
use crate::minecraft::client_json::ClientJson;
use crate::minecraft::client_json::Rule;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

pub async fn install_libraries(
    client_json: &ClientJson,
    progress: &mut Progress,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    let features = HashMap::new(); // idk will it support quick play ill see
    for library in &client_json.libraries {
        if !is_library_allowed(&library.rules, &features) {
            continue;
        }

        if let Some(artifact) = &library.downloads.artifact {
            let download_obj = DownloadObject {
                url: artifact.url.clone(),
                size: Some(artifact.size),
                sha1: Some(artifact.sha1.clone()),
                file_path: PathBuf::from("libraries").join(&artifact.path),
            };

            download_obj.download_file().await?;
            progress.add_file(app, artifact.size);
        }
    }

    Ok(())
}

pub fn is_library_allowed(rules: &Option<Vec<Rule>>, features: &HashMap<String, bool>) -> bool {
    let Some(rules) = rules else {
        return true;
    };

    let current_os_name = std::env::consts::OS;
    let current_arch_name = std::env::consts::ARCH;
    let mut allowed = false;

    for rule in rules {
        if let Some(os) = &rule.os {
            if let Some(name) = &os.name {
                if name != current_os_name {
                    continue;
                }
                if let Some(arch) = &os.arch {
                    if arch != current_arch_name {
                        continue;
                    }
                }
            }
            // todo: os version
        }

        if let Some(rule_features) = &rule.features {
            let mut features_match = true;
            for (feature, expected) in rule_features {
                let actual = features.get(feature).copied().unwrap_or(false);
                if actual != *expected {
                    features_match = false;
                    break;
                }
            }
            if !features_match {
                continue;
            }
        }

        if let Some(action) = &rule.action {
            if action == "allow" {
                allowed = true;
            } else {
                allowed = false;
            }
        }
    }

    allowed
}
