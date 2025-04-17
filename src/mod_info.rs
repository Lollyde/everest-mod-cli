use bytes::Bytes;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const MOD_LIST_URL: &str = "https://everestapi.github.io/modupdater.txt";

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

impl RemoteModInfo {
    /// Checks if the provided hash matches any of the expected checksums.
    ///
    /// # Arguments
    ///
    /// * `computed_hash` - The hash to check against the mod's checksums.
    ///
    /// # Returns
    ///
    /// Returns `true` if the hash matches any of the checksums, otherwise `false`.
    pub fn has_matching_hash(&self, computed_hash: &String) -> bool {
        // Check if the computed hash exists in the list of expected checksums
        self.hash.contains(computed_hash)
    }
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

    pub async fn fetch_from_network() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();

        // First, fetch the URL of the YAML file
        let yaml_url = client
            .get(MOD_LIST_URL)
            .send()
            .await?
            .text()
            .await?
            .trim()
            .to_string();

        println!("Fetching mod list from: {}", yaml_url);

        // Then fetch the actual YAML content
        let yaml_content = client.get(&yaml_url).send().await?.text().await?;

        let mut catalog: ModCatalog = serde_yaml_ng::from_str(&yaml_content)?;

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
