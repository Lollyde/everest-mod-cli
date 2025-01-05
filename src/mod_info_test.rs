#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_load_update_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let catalog = ModCatalog::load_from_file("test/everest_update.yaml").await?;
        
        // Test basic structure
        assert!(!catalog.mods.is_empty());

        // Test specific mod entry (CSRC Frog)
        let frog_mod = catalog.mods.get("CSRC Frog").expect("CSRC Frog mod not found");
        assert_eq!(frog_mod.version, "1.0.1");
        assert_eq!(frog_mod.last_update, 1728796397);
        assert_eq!(frog_mod.gamebanana_type.as_deref(), Some("Tool"));
        assert_eq!(frog_mod.gamebanana_id, Some(15836));
        assert_eq!(frog_mod.xx_hash, vec!["f437bf0515368130"]);
        assert_eq!(frog_mod.url, "https://gamebanana.com/mmdl/1298450");

        // Test mod with different version format
        let viewpoint_mod = catalog.mods.get("viewpoint-dreampoint-point")
            .expect("viewpoint-dreampoint-point mod not found");
        assert_eq!(viewpoint_mod.version, "1.0");

        // Test search functionality
        let results = catalog.search("CSRC");
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
        
        let result: Result<ModCatalog, _> = serde_yaml::from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let incomplete_yaml = r#"
            TestMod:
              GameBananaType: Tool
              # Missing required fields like version, URL, etc.
        "#;
        
        let result: Result<ModCatalog, _> = serde_yaml::from_str(incomplete_yaml);
        assert!(result.is_err());
    }
}
