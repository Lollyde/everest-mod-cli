use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModMetadata {
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
    #[serde(rename = "LastUpdate")]
    pub last_update: i64,
    #[serde(rename = "xxHash", alias = "MD5")]
    pub hash: Vec<String>,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "GameBananaType")]
    pub gamebanana_type: Option<String>,
    #[serde(rename = "GameBananaId")]
    pub gamebanana_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModMetadataList(pub Vec<ModMetadata>);

impl ModMetadataList {
    pub fn from_zip(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with("everest.yaml") || file.name().ends_with("everest.yml") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                
                // Remove BOM if present
                if contents.starts_with('\u{feff}') {
                    contents = contents[3..].to_string();
                }
                
                let metadata: Vec<ModMetadata> = serde_yaml::from_str(&contents)?;
                return Ok(ModMetadataList(metadata));
            }
        }
        Err("No everest.yaml found in zip file".into())
    }

    pub fn get_main_mod(&self) -> Option<&ModMetadata> {
        self.0.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    #[test]
    fn test_load_update_yaml() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory
        let dir = tempdir()?;
        let zip_path = dir.path().join("test.zip");
        let mut zip = zip::ZipWriter::new(File::create(&zip_path)?);

        // Write test YAML content to a file in the ZIP
        let yaml_content = r#"
- Name: FrogMod
  Version: 1.0.0
  LastUpdate: 1728796397
  GameBananaType: Tool
  GameBananaId: 15836
  xxHash: ["f437bf0515368130"]
  URL: https://gamebanana.com/mmdl/1298450
"#;
        zip.start_file::<_, ()>("everest.yaml", FileOptions::default())?;
        zip.write_all(yaml_content.as_bytes())?;
        zip.finish()?;

        // Test loading the YAML from the ZIP
        let metadata_list = ModMetadataList::from_zip(&zip_path)?;
        let frog_mod = metadata_list.get_main_mod().unwrap();

        // Verify the parsed content
        assert_eq!(frog_mod.name, "FrogMod");
        assert_eq!(frog_mod.version, "1.0.0");
        assert_eq!(frog_mod.last_update, 1728796397);
        assert_eq!(frog_mod.gamebanana_type.as_deref(), Some("Tool"));
        assert_eq!(frog_mod.gamebanana_id, Some(15836));
        assert_eq!(frog_mod.hash.join(", "), "f437bf0515368130");
        assert_eq!(frog_mod.url, "https://gamebanana.com/mmdl/1298450");

        Ok(())
    }

    #[test]
    fn test_invalid_yaml() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("invalid.zip");
        let mut zip = zip::ZipWriter::new(File::create(&zip_path).unwrap());

        let invalid_yaml = "invalid: [yaml: content";
        zip.start_file::<_, ()>("everest.yaml", FileOptions::default()).unwrap();
        zip.write_all(invalid_yaml.as_bytes()).unwrap();
        zip.finish().unwrap();

        assert!(ModMetadataList::from_zip(&zip_path).is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("missing_fields.zip");
        let mut zip = zip::ZipWriter::new(File::create(&zip_path).unwrap());

        let yaml_content = "- Version: 1.0.0"; // Missing required fields
        zip.start_file::<_, ()>("everest.yaml", FileOptions::default()).unwrap();
        zip.write_all(yaml_content.as_bytes()).unwrap();
        zip.finish().unwrap();

        assert!(ModMetadataList::from_zip(&zip_path).is_err());
    }
}
