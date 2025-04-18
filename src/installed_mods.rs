use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::PathBuf};

use crate::error::Error;

/// Represents the `everest.yaml` manifest file that defines a mod
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModManifest {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "DLL")]
    pub dll: Option<String>,
    #[serde(rename = "Dependencies")]
    pub dependencies: Option<Vec<Dependency>>,
    #[serde(rename = "OptionalDependencies")]
    pub optional_dependencies: Option<Vec<Dependency>>,
}

/// Dependency specification for required or optional mod dependencies
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Option<String>,
}

impl ModManifest {
    /// Parses the mod manifest YAML content into a structured `ModManifest` object.
    pub fn parse_mod_manifest_from_yaml(yaml_content: &str) -> Result<Self, Error> {
        let mut manifest_entries = serde_yaml_ng::from_str::<VecDeque<ModManifest>>(yaml_content)?;

        // Attempt to retrieve the first entry without unnecessary cloning.
        manifest_entries
            .pop_front()
            .ok_or_else(|| Error::NoEntriesInModManifest(manifest_entries))
    }
}

/// Collection of all installed mods and their metadata
pub type InstalledModList = Vec<LocalModInfo>;

/// Information about a locally installed mod
#[derive(Debug, Deserialize, Serialize)]
pub struct LocalModInfo {
    /// Path to the zip file which contains the mod's assets and manifest
    #[serde(rename = "Filename")]
    pub archive_path: PathBuf,
    /// Name of the mod as defined in its manifest
    #[serde(rename = "Mod Name")]
    pub mod_name: String,
    /// Version string of the mod
    #[serde(rename = "Version")]
    pub version: String,
    /// Computed XXH64 hash of the mod archive for update verification
    #[serde(rename = "xxHash")]
    pub checksum: String,
}

impl LocalModInfo {
    /// Creates a new LocalModInfo from a mod's archive path, manifest, and computed checksum
    pub fn new(archive_path: PathBuf, manifest: ModManifest, checksum: String) -> Self {
        let mod_name = manifest.name;
        let version = manifest.version;

        // Extract basename from full path.
        let archive_filename = archive_path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| archive_path.clone());

        Self {
            archive_path: archive_filename,
            mod_name,
            version,
            checksum,
        }
    }
}
