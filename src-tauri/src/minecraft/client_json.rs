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

#[derive(Serialize, Deserialize)]
pub struct ClientJson {
    pub id: Option<String>,
    pub r#type: Option<String>,

    pub arguments: Option<Arguments>,
    pub downloads: ClientDownloads,
    pub libraries: Vec<Library>,

    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,

    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>, // fuck you mojang

    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
}

#[derive(Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientDownloads {
    pub client: Download,
    //pub server: Download,
    pub client_mappings: Option<Download>, // if i ever get to mods in launcher
}

#[derive(Serialize, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extract>,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Serialize, Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,

    #[serde(rename = "totalSize")]
    pub total_size: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub action: Option<String>,
    pub features: Option<HashMap<String, bool>>,
    pub os: Option<OS>,
}

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

pub async fn fetch_or_get_client_json(version: &Version) -> Result<ClientJson, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("home dir doesn't exist")?;
    let path_to_versions_dir = home.join(".minecraft/versions").join(&version.id);
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
    } // todo: add retry now its just an err

    create_dir_all(&path_to_versions_dir)?;
    write(&path_to_version_json, &res)?;
    let json = from_slice::<ClientJson>(&res)?;

    Ok(json)
}
