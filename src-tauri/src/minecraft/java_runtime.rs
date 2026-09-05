use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    fs
};

use reqwest::Client;
use serde::Deserialize;
use tauri::AppHandle;

use crate::launcher::download::DownloadObject;
use crate::launcher::progress::Progress;

const RUNTIME_INDEX_URL: &str =
    "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Deserialize)]
struct RuntimeManifestRef {
    url: String,
}

#[derive(Deserialize)]
struct RuntimeEntry {
    manifest: RuntimeManifestRef,
}

type RuntimeIndex = HashMap<String, HashMap<String, Vec<RuntimeEntry>>>;

#[derive(Deserialize)]
struct FilesManifest {
    files: HashMap<String, FileEntry>,
}

#[derive(Deserialize)]
struct FileEntry {
    r#type: String,
    #[serde(default)]
    executable: bool,
    downloads: Option<FileDownloads>,
}

#[derive(Deserialize)]
struct FileDownloads {
    raw: RawDownload,
}

#[derive(Deserialize)]
struct RawDownload {
    sha1: String,
    size: u64,
    url: String,
}

fn platform_key() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86") => Some("linux-i386"),
        ("linux", _) => Some("linux"),
        ("macos", "aarch64") => Some("mac-os-arm64"),
        ("macos", _) => Some("mac-os"),
        ("windows", "aarch64") => Some("windows-arm64"),
        ("windows", "x86") => Some("windows-x86"),
        ("windows", _) => Some("windows-x64"),
        _ => None,
    }
}

fn java_bin_path(minecraft_dir: &Path, component: &str) -> PathBuf {
    let bin_name = if cfg!(windows) { "javaw.exe" } else { "java" };
    minecraft_dir
        .join("runtime")
        .join(component)
        .join("bin")
        .join(bin_name)
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

pub async fn ensure_java_runtime(
    minecraft_dir: &Path,
    component: &str,
    app: &AppHandle,
) -> Result<PathBuf, Box<dyn Error>> {
    let java_path = java_bin_path(minecraft_dir, component);
    if java_path.exists() {
        return Ok(java_path);
    }

    let platform = platform_key().ok_or("no bundled Java build is published for this OS/CPU")?;

    let client = Client::new();
    let index: RuntimeIndex = client
        .get(RUNTIME_INDEX_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let manifest_url = index
        .get(platform)
        .and_then(|components| components.get(component))
        .and_then(|entries| entries.first())
        .map(|entry| entry.manifest.url.clone())
        .ok_or_else(|| format!("Mojang doesn't publish a {component} build for {platform}"))?;

    let manifest: FilesManifest = client
        .get(manifest_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let files: Vec<(&String, &FileEntry)> = manifest
        .files
        .iter()
        .filter(|(_, entry)| entry.r#type == "file")
        .collect();

    let total_bytes = files
        .iter()
        .filter_map(|(_, entry)| entry.downloads.as_ref())
        .map(|d| d.raw.size)
        .sum::<u64>();
    let mut progress = Progress::with_event(total_bytes, files.len() as u64, "launch-progress");

    let component_root = PathBuf::from("runtime").join(component);
    for (relative_path, entry) in files {
        let downloads = entry
            .downloads
            .as_ref()
            .ok_or("runtime file entry is missing its download info")?;
        let download = DownloadObject {
            url: downloads.raw.url.clone(),
            size: Some(downloads.raw.size),
            sha1: Some(downloads.raw.sha1.clone()),
            file_path: component_root.join(relative_path),
        };
        download
            .download_file(|n| progress.add_bytes(app, n))
            .await?;
        progress.finish_file(app);

        if entry.executable {
            mark_executable(&minecraft_dir.join(&component_root).join(relative_path))?;
        }
    }

    if !java_path.exists() {
        return Err(
            format!("{component} runtime downloaded but its java binary is missing").into(),
        );
    }

    Ok(java_path)
}
