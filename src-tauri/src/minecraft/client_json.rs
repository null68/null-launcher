use std::collections::HashMap;

use serde::{self, Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Artifact,
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
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
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
