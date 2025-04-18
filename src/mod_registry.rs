use std::collections::HashMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Each entry in `everest_update.yaml` containing information about a mod
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteModInfo {
    /// Actual mod name (not filename)
    #[serde(skip)]
    pub name: String,
    /// Version string
    #[serde(rename = "Version")]
    pub version: String,
    /// File size
    #[serde(rename = "Size")]
    pub file_size: u64,
    /// Timestamp of last update
    #[serde(rename = "LastUpdate")]
    pub updated_at: u64,
    /// Download link
    #[serde(rename = "URL")]
    pub download_url: String,
    /// Checksums
    #[serde(rename = "xxHash")]
    pub checksums: Vec<String>,
    /// Category for a mod
    #[serde(rename = "GameBananaType")]
    pub gamebanana_type: String,
    /// Reference ID of gamebanana page
    #[serde(rename = "GameBananaId")]
    pub gamebanana_id: u32,
}

/// Mod Registry: represents the complete `everest_update.yaml` containing all available remote mods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRegistry {
    #[serde(flatten)]
    pub entries: HashMap<String, RemoteModInfo>,
}

impl ModRegistry {
    /// Initialize ModRegistry instance from raw data
    pub async fn from(data: Bytes) -> Result<Self, serde_yaml_ng::Error> {
        let mut mod_registry: Self = serde_yaml_ng::from_slice(&data)?;

        // Set the name field for each ModInfo
        for (key, mod_info) in mod_registry.entries.iter_mut() {
            mod_info.name = key.clone();
        }

        Ok(mod_registry)
    }

    /// Search for mods
    pub fn search(&self, query: &str) -> Vec<&RemoteModInfo> {
        self.entries
            .values()
            .filter(|mod_info| mod_info.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    /// Get mod information
    pub fn get_mod_info(&self, name: &str) -> Option<&RemoteModInfo> {
        self.entries.get(name)
    }
}
