use reqwest::Client;
use serde::{Deserialize, Serialize};

const MOD_LIST_URL: &str = "https://everestapi.github.io/modupdater.txt";

#[derive(Debug, Serialize, Deserialize)]
pub struct ModInfo {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ModCatalog {
    #[serde(flatten)]
    pub mods: std::collections::HashMap<String, ModInfo>,
}

impl ModCatalog {
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

    pub fn search(&self, query: &str) -> Vec<&ModInfo> {
        self.mods
            .values()
            .filter(|mod_info| mod_info.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn get_mod(&self, name: &str) -> Option<&ModInfo> {
        self.mods.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_update_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let catalog = ModCatalog::fetch_from_network().await?;

        // Test basic structure
        assert!(!catalog.mods.is_empty());

        // Test specific mod entry (CSRC Frog)
        let frog_mod = catalog
            .mods
            .get("CSRC Frog")
            .expect("CSRC Frog mod not found");
        assert_eq!(frog_mod.version, "1.0.1");
        assert_eq!(frog_mod.last_update, 1728796397);
        assert_eq!(frog_mod.gamebanana_type.as_deref(), Some("Tool"));
        assert_eq!(frog_mod.gamebanana_id, Some(15836));
        assert_eq!(frog_mod.hash, vec!["f437bf0515368130"]);
        assert_eq!(frog_mod.url, "https://gamebanana.com/mmdl/1298450");

        // Test mod with different version format
        let viewpoint_mod = catalog
            .mods
            .get("viewpoint-dreampoint-point")
            .expect("viewpoint-dreampoint-point mod not found");
        assert_eq!(viewpoint_mod.version, "1.0");

        // Test search functionality
        let results = catalog.search("CSRC Frog");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "CSRC Frog");

        // Test get_mod functionality
        let mod_info = catalog.get_mod("CSRC Frog");
        assert!(mod_info.is_some());
        assert_eq!(mod_info.unwrap().version, "1.0.1");

        Ok(())
    }

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
