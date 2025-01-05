use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModMetadata {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Version")]
    pub version: String,
    #[serde(default)]
    pub dll: Option<String>,
    #[serde(default)]
    pub dependencies: Option<Vec<Dependency>>,
    #[serde(rename = "OptionalDependencies")]
    #[serde(default)]
    pub optional_dependencies: Option<Vec<Dependency>>,
}

#[derive(Debug, Clone)]
pub struct ModMetadataList(pub Vec<ModMetadata>);

impl ModMetadataList {
    pub fn from_zip(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Look for everest.yaml in the zip
        let mut everest_yaml = None;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_lowercase();
            if name.ends_with("everest.yaml") || name.ends_with("everest.yml") {
                everest_yaml = Some(i);
                break;
            }
        }

        // Read and parse everest.yaml if found
        if let Some(index) = everest_yaml {
            let mut file = archive.by_index(index)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Remove BOM if present
            if contents.starts_with('\u{feff}') {
                contents = contents[3..].to_string(); // Skip the BOM (3 bytes)
            }

            let metadata: Vec<ModMetadata> = serde_yaml::from_str(&contents)?;
            Ok(ModMetadataList(metadata))
        } else {
            Err("No everest.yaml found in zip file".into())
        }
    }

    pub fn get_main_mod(&self) -> Option<&ModMetadata> {
        self.0.first()
    }

    pub fn get_dependencies(&self) -> Vec<&Dependency> {
        self.get_main_mod()
            .and_then(|mod_meta| mod_meta.dependencies.as_ref())
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_optional_dependencies(&self) -> Vec<&Dependency> {
        self.get_main_mod()
            .and_then(|mod_meta| mod_meta.optional_dependencies.as_ref())
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }
}

// Function to compare version strings
pub fn compare_versions(ver1: Option<&str>, ver2: Option<&str>) -> std::cmp::Ordering {
    match (ver1, ver2) {
        (None, None) => std::cmp::Ordering::Equal,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(v1), Some(v2)) => {
            let v1_parts: Vec<&str> = v1.split('.').collect();
            let v2_parts: Vec<&str> = v2.split('.').collect();

            for i in 0..std::cmp::max(v1_parts.len(), v2_parts.len()) {
                let n1 = v1_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                let n2 = v2_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                
                match n1.cmp(&n2) {
                    std::cmp::Ordering::Equal => continue,
                    other => return other,
                }
            }
            std::cmp::Ordering::Equal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_yaml_with_bom() -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("test/test_with_bom.zip");
        let metadata_list = ModMetadataList::from_zip(&path)?;
        
        assert_eq!(metadata_list.0.len(), 1);
        let metadata = &metadata_list.0[0];
        assert_eq!(metadata.name, "TestMod");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dll, Some("TestMod.dll".to_string()));
        
        let dependencies = metadata.dependencies.as_ref().unwrap();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "Celeste");
        assert_eq!(dependencies[0].version, Some("1.4.0.0".to_string()));
        
        Ok(())
    }
}
