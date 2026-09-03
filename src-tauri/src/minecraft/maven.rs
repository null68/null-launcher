use std::{error::Error, path::Path};

use tauri::AppHandle;

use crate::launcher::download::DownloadObject;
use crate::launcher::progress::Progress;
use crate::minecraft::client_json::{Artifact, Library};

// group:artifact:version[:classifier][@ext]
pub fn coords_to_path(coords: &str) -> Option<String> {
    let (coords, extension) = match coords.split_once('@') {
        Some((c, ext)) => (c, ext),
        None => (coords, "jar"),
    };
    let parts: Vec<&str> = coords.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let group_path = group.replace('.', "/");
    let file_name = match parts.get(3) {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.{extension}"),
        None => format!("{artifact}-{version}.{extension}"),
    };
    Some(format!("{group_path}/{artifact}/{version}/{file_name}"))
}

pub fn coords_to_url(coords: &str, base_url: &str) -> Option<String> {
    let path = coords_to_path(coords)?;
    Some(format!("{}/{path}", base_url.trim_end_matches('/')))
}

pub fn resolve_library_artifact(library: &Library) -> Option<Artifact> {
    if let Some(artifact) = library.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
        return Some(artifact.clone());
    }
    let base = library
        .url
        .as_deref()
        .unwrap_or("https://libraries.minecraft.net/");
    Some(Artifact {
        path: coords_to_path(&library.name)?,
        url: coords_to_url(&library.name, base)?,
        sha1: None,
        size: None,
    })
}

pub async fn download_libraries(
    libraries: &[Library],
    progress: &mut Progress,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    for library in libraries {
        let Some(artifact) = resolve_library_artifact(library) else {
            continue;
        };
        let download = DownloadObject {
            url: artifact.url.clone(),
            size: artifact.size,
            sha1: artifact.sha1.clone(),
            file_path: Path::new("libraries").join(&artifact.path),
        };
        download.download_file(|_| {}).await?;
        progress.add_file(app, artifact.size.unwrap_or(0));
    }
    Ok(())
}
