use crate::launcher::minecraft_dir::get_minecraft_dir;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, create_dir_all, remove_file, write},
    path::Path,
};

use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use serde_json::from_slice;
use sha1::{Digest, Sha1};

use crate::minecraft::manifest::Version;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientJson {
    pub id: Option<String>,
    pub r#type: Option<String>,

    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,

    pub arguments: Option<Arguments>,
    pub downloads: Option<ClientDownloads>,
    #[serde(default)]
    pub libraries: Vec<Library>,

    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,

    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>, // fuck you mojang

    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,

    pub logging: Option<Logging>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientDownloads {
    pub client: Download,
    //pub server: Download,
    pub client_mappings: Option<Download>, // if i ever get to mods in launcher, update: guess what u piece of shit
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    pub url: Option<String>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    pub path: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,

    #[serde(rename = "totalSize")]
    pub total_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: Option<String>,
    pub features: Option<HashMap<String, bool>>,
    pub os: Option<OS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OS {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

pub async fn fetch_or_get_client_json(version: &Version) -> Result<ClientJson, Box<dyn Error>> {
    let minecraft_dir = get_minecraft_dir()?;
    let path_to_versions_dir = minecraft_dir.join("versions").join(&version.id);
    let path_to_version_json = path_to_versions_dir.join(format!("{}.json", version.id));

    if path_to_version_json.exists() {
        let bytes = fs::read(&path_to_version_json)?;
        let mut hasher = Sha1::new();

        hasher.update(&bytes);
        let hash = hasher.finalize();
        let hash = hex::encode(hash);
        if version.sha1 == hash {
            let json = from_slice::<ClientJson>(&bytes)?;

            return Ok(json);
        }

        remove_file(&path_to_version_json)?;
    }
    let client = Client::new();

    let res = client
        .get(&version.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let mut hasher = Sha1::new();

    hasher.update(&res);
    let hash = hasher.finalize();
    let hash = hex::encode(hash);
    if version.sha1 != hash {
        return Err(String::from("sha1 mismatch").into());
    }

    create_dir_all(&path_to_versions_dir)?;
    write(&path_to_version_json, &res)?;
    let json = from_slice::<ClientJson>(&res)?;
    Ok(json)
}

pub fn read_client_json_from_disk(
    minecraft_dir: &Path,
    id: &str,
) -> Result<ClientJson, Box<dyn Error>> {
    let path = minecraft_dir
        .join("versions")
        .join(id)
        .join(format!("{id}.json"));
    let bytes = fs::read(&path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    Ok(from_slice::<ClientJson>(&bytes)?)
}

pub fn write_client_json(
    minecraft_dir: &Path,
    id: &str,
    json: &ClientJson,
) -> Result<(), Box<dyn Error>> {
    let dir = minecraft_dir.join("versions").join(id);
    create_dir_all(&dir)?;
    let bytes = serde_json::to_vec_pretty(json)?;
    write(dir.join(format!("{id}.json")), bytes)?;
    Ok(())
}

pub async fn resolve_client_json(
    minecraft_dir: &Path,
    id: &str,
) -> Result<ClientJson, Box<dyn Error>> {
    let json = read_client_json_from_disk(minecraft_dir, id)?;
    match &json.inherits_from {
        Some(parent_id) => {
            let parent = read_client_json_from_disk(minecraft_dir, parent_id).map_err(|e| {
                format!("{id} inherits from {parent_id}, which isn't installed ({e})")
            })?;
            Ok(merge_client_json(parent, json))
        }
        None => Ok(json),
    }
}

fn library_key(name: &str) -> String {
    name.splitn(3, ':').take(2).collect::<Vec<_>>().join(":")
}

pub fn merge_client_json(parent: ClientJson, child: ClientJson) -> ClientJson {
    let child_coords: std::collections::HashSet<String> = child
        .libraries
        .iter()
        .map(|l| library_key(&l.name))
        .collect();

    let mut libraries: Vec<Library> = parent
        .libraries
        .into_iter()
        .filter(|l| !child_coords.contains(&library_key(&l.name)))
        .collect();
    libraries.extend(child.libraries);

    let parent_args = match parent.arguments {
        Some(args) => Some(args),
        None => parent.minecraft_arguments.map(|legacy| Arguments {
            jvm: Some(vec![
                Argument::Simple("-Djava.library.path=${natives_directory}".to_string()),
                Argument::Simple("-cp".to_string()),
                Argument::Simple("${classpath}".to_string()),
            ]),
            game: Some(
                legacy
                    .split_whitespace()
                    .map(|s| Argument::Simple(s.to_string()))
                    .collect(),
            ),
        }),
    };

    let arguments = match (parent_args, child.arguments) {
        (Some(p), Some(c)) => Some(Arguments {
            jvm: Some(
                p.jvm
                    .unwrap_or_default()
                    .into_iter()
                    .chain(c.jvm.unwrap_or_default())
                    .collect(),
            ),
            game: Some(
                p.game
                    .unwrap_or_default()
                    .into_iter()
                    .chain(c.game.unwrap_or_default())
                    .collect(),
            ),
        }),
        (Some(p), None) => Some(p),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    ClientJson {
        id: child.id.or(parent.id),
        r#type: child.r#type.or(parent.r#type),
        inherits_from: None,
        arguments,
        downloads: child.downloads.or(parent.downloads),
        libraries,
        main_class: child.main_class.or(parent.main_class),
        asset_index: child.asset_index.or(parent.asset_index),
        minecraft_arguments: None,
        java_version: child.java_version.or(parent.java_version),
        logging: child.logging.or(parent.logging),
    }
}

pub fn detect_loader(client_json: &ClientJson) -> Option<&'static str> {
    let has = |prefix: &str| {
        client_json
            .libraries
            .iter()
            .any(|l| l.name.starts_with(prefix))
    };
    if has("net.neoforged:") {
        Some("neoforge")
    } else if has("net.minecraftforge:") {
        Some("forge")
    } else if has("net.fabricmc:fabric-loader") {
        Some("fabric")
    } else if has("org.quiltmc:quilt-loader") {
        Some("quilt")
    } else if has("optifine:OptiFine") {
        Some("optifine")
    } else {
        None
    }
}
