use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteModInfo {
    #[serde(skip)]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "LastUpdate")]
    pub last_update: i64,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "xxHash", alias = "MD5")]
    pub hash: Vec<String>,
    #[serde(rename = "GameBananaType")]
    pub gamebanana_type: Option<String>,
    #[serde(rename = "GameBananaId")]
    pub gamebanana_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModCatalog {
    #[serde(flatten)]
    pub mods: std::collections::HashMap<String, RemoteModInfo>,
}

impl ModCatalog {
    pub async fn new(data: Bytes) -> Result<Self, serde_yaml_ng::Error> {
        let mut catalog: Self = serde_yaml_ng::from_slice(&data)?;

        // Set the name field for each ModInfo
        for (key, mod_info) in catalog.mods.iter_mut() {
            mod_info.name = key.clone();
        }

        Ok(catalog)
    }

    pub fn search(&self, query: &str) -> Vec<&RemoteModInfo> {
        self.mods
            .values()
            .filter(|mod_info| mod_info.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn get_mod(&self, name: &str) -> Option<&RemoteModInfo> {
        self.mods.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_yaml() {
        let invalid_yaml = r#"
            invalid:
              - missing: colon
                broken structure
        "#;

        let result: Result<ModCatalog, _> = serde_yaml_ng::from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let incomplete_yaml = r#"
            TestMod:
              GameBananaType: Tool
              # Missing required fields like version, URL, etc.
        "#;

        let result: Result<ModCatalog, _> = serde_yaml_ng::from_str(incomplete_yaml);
        assert!(result.is_err());
    }
}
