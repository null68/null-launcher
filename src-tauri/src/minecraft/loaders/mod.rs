pub mod fabric;
pub mod forge;
pub mod installer_common;
pub mod neoforge;
pub mod quilt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl std::str::FromStr for Loader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fabric" => Ok(Loader::Fabric),
            "quilt" => Ok(Loader::Quilt),
            "forge" => Ok(Loader::Forge),
            "neoforge" => Ok(Loader::NeoForge),
            other => Err(format!("unknown loader '{other}'")),
        }
    }
}
