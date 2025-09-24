//! Tests for CLI integration with builder patterns
//! Tests the CLI usage of VaultBuilder and QueryOptions builder patterns

use obsidian_cli::config::Config;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod cli_builder_integration_tests {
    use super::*;

    fn create_test_environment() -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault");
        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        let config = Config {
            vault: Some(vault_path),
            blacklist: vec!["*.tmp".to_string(), "cache/".to_string()],
            editor: Some("test_editor".to_string()),
            ident_key: "test_uid".to_string(),
            journal_template: "Notes/{year}/{month:02d}/{day:02d}".to_string(),
            verbose: false,
        };

        (temp_dir, config)
    }

    #[test]
    fn test_vault_builder_with_config_blacklist() {
        let (_temp_dir, config) = create_test_environment();
        let vault_path = config.vault.as_ref().unwrap().clone();

        // Simulate CLI vault construction with config blacklist
        let mut vault_builder = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .editor(config.get_editor())
            .ident_key(config.ident_key.clone())
            .journal_template(config.journal_template.clone())
            .verbose(false);

        vault_builder =
            vault_builder.blacklist_patterns(config.blacklist.iter().map(|s| s.as_str()));

        let vault = vault_builder.build().unwrap();

        assert_eq!(vault.path, vault_path);
        assert_eq!(vault.editor.as_str(), "test_editor");
        assert_eq!(vault.ident_key.as_str(), "test_uid");
        assert_eq!(
            vault.journal_template.as_str(),
            "Notes/{year}/{month:02d}/{day:02d}"
        );
        assert_eq!(vault.blacklist.len(), 2);
        assert_eq!(vault.blacklist[0].as_str(), "*.tmp");
        assert_eq!(vault.blacklist[1].as_str(), "cache/");
    }

    #[test]
    fn test_vault_builder_with_cli_override_blacklist() {
        let (_temp_dir, config) = create_test_environment();
        let vault_path = config.vault.as_ref().unwrap().clone();
        let cli_blacklist = vec!["override1.txt".to_string(), "override2/*".to_string()];

        // Simulate CLI vault construction with CLI-provided blacklist
        let mut vault_builder = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .editor(config.get_editor())
            .ident_key(config.ident_key.clone())
            .journal_template(config.journal_template.clone())
            .verbose(true);

        vault_builder = vault_builder.blacklist_patterns(cli_blacklist.iter().map(|s| s.as_str()));

        let vault = vault_builder.build().unwrap();

        assert_eq!(vault.blacklist.len(), 2);
        assert_eq!(vault.blacklist[0].as_str(), "override1.txt");
        assert_eq!(vault.blacklist[1].as_str(), "override2/*");
        assert!(vault.verbose); // CLI override
    }

    #[test]
    fn test_vault_builder_with_empty_blacklist() {
        let (_temp_dir, config) = create_test_environment();
        let vault_path = config.vault.as_ref().unwrap().clone();
        let empty_blacklist: Vec<String> = vec![];

        // Test with empty blacklist
        let mut vault_builder = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .editor(config.get_editor())
            .ident_key(config.ident_key.clone())
            .journal_template(config.journal_template.clone())
            .verbose(false);

        vault_builder =
            vault_builder.blacklist_patterns(empty_blacklist.iter().map(|s| s.as_str()));

        let vault = vault_builder.build().unwrap();

        assert!(vault.blacklist.is_empty());
    }

    #[test]
    fn test_vault_builder_error_handling() {
        let _config = Config::default();

        // Test vault construction failure when path is missing from config
        let vault_builder = obsidian_cli::types::Vault::builder()
            .editor("vim")
            .ident_key("uid")
            .journal_template("test");

        let result = vault_builder.build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vault path is required");
    }

    #[test]
    fn test_query_options_builder_cli_style() {
        // Test QueryOptions construction as would be done in CLI
        let result = obsidian_cli::commands::query::QueryOptions::builder()
            .key("status")
            .value("published")
            .exists(false)
            .missing(false)
            .style(obsidian_cli::types::OutputStyle::Json)
            .count(false)
            .build();

        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.key, "status");
        assert_eq!(options.value, Some("published"));
        assert!(matches!(
            options.style,
            obsidian_cli::types::OutputStyle::Json
        ));
    }

    #[test]
    fn test_query_options_builder_cli_validation() {
        // Test the validation that would catch CLI argument conflicts
        let result = obsidian_cli::commands::query::QueryOptions::builder()
            .key("tags")
            .value("rust")
            .contains("programming") // This should conflict with value
            .build();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot specify both value and contains options"
        );
    }

    #[test]
    fn test_vault_builder_with_all_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("minimal_vault");
        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Test minimal vault construction (all defaults)
        let vault = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .build()
            .unwrap();

        assert_eq!(vault.path, vault_path);
        assert_eq!(vault.editor.as_str(), "vi"); // Default
        assert_eq!(vault.ident_key.as_str(), "uid"); // Default
        assert!(vault.journal_template.as_str().contains("Calendar")); // Default template
        assert!(!vault.verbose); // Default
        assert!(vault.blacklist.is_empty()); // Default empty
    }

    #[test]
    fn test_vault_builder_mixed_string_types() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("mixed_vault");
        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Test with mixed string types (String vs &str)
        let editor_string = "nvim".to_string();
        let ident_key_str = "custom_id";
        let template_string = "Journal/{year}".to_string();

        let vault = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .editor(editor_string) // String
            .ident_key(ident_key_str) // &str
            .journal_template(template_string) // String
            .build()
            .unwrap();

        assert_eq!(vault.editor.as_str(), "nvim");
        assert_eq!(vault.ident_key.as_str(), "custom_id");
        assert_eq!(vault.journal_template.as_str(), "Journal/{year}");
    }

    #[test]
    fn test_vault_builder_pathbuf_vs_path() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("pathbuf_vault");
        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Test with PathBuf
        let vault1 = obsidian_cli::types::Vault::builder()
            .path(vault_path.clone()) // PathBuf
            .build()
            .unwrap();

        // Test with &Path
        let vault2 = obsidian_cli::types::Vault::builder()
            .path(&vault_path) // &Path
            .build()
            .unwrap();

        assert_eq!(vault1.path, vault_path);
        assert_eq!(vault2.path, vault_path);
        assert_eq!(vault1.path, vault2.path);
    }

    #[test]
    fn test_builder_pattern_performance() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("perf_vault");
        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        let start = std::time::Instant::now();

        // Build vault with many blacklist patterns
        let patterns: Vec<String> = (0..100).map(|i| format!("pattern_{}.tmp", i)).collect();

        let mut vault_builder = obsidian_cli::types::Vault::builder()
            .path(&vault_path)
            .editor("vim")
            .ident_key("uid")
            .journal_template("test/{year}")
            .verbose(false);

        vault_builder = vault_builder.blacklist_patterns(patterns.iter().map(|s| s.as_str()));

        let vault = vault_builder.build().unwrap();
        let duration = start.elapsed();

        // Should complete quickly even with many patterns
        assert!(
            duration.as_millis() < 50,
            "Builder took too long: {:?}",
            duration
        );
        assert_eq!(vault.blacklist.len(), 100);
    }

    #[test]
    fn test_builder_error_propagation() {
        // Test that builder errors can be properly converted to CLI errors
        let vault_builder = obsidian_cli::types::Vault::builder().editor("vim");

        let result = vault_builder.build().map_err(|e| {
            obsidian_cli::errors::ObsidianError::Config(
                obsidian_cli::errors::ConfigError::InvalidValue {
                    field: "vault_construction".to_string(),
                    value: e.to_string(),
                },
            )
        });

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(format!("{}", error).contains("Configuration error"));
        assert!(format!("{}", error).contains("Vault path is required"));
    }

    #[test]
    fn test_query_builder_with_output_styles() {
        use obsidian_cli::types::OutputStyle;

        let styles = [
            OutputStyle::Path,
            OutputStyle::Title,
            OutputStyle::Table,
            OutputStyle::Json,
        ];

        for style in styles {
            let options = obsidian_cli::commands::query::QueryOptions::builder()
                .key("test_key")
                .style(style)
                .build()
                .unwrap();

            assert_eq!(options.key, "test_key");
            // Each style should build successfully
        }
    }

    #[test]
    fn test_builders_with_clone_vs_reference_patterns() {
        let (_temp_dir, config) = create_test_environment();
        let vault_path = config.vault.as_ref().unwrap();

        // Test that we can build without cloning config values
        let vault = obsidian_cli::types::Vault::builder()
            .path(vault_path) // Reference, not clone
            .editor(config.get_editor()) // Owned String
            .ident_key(config.ident_key.as_str()) // Reference to String content
            .journal_template(config.journal_template.as_str()) // Reference to String content
            .blacklist_patterns(config.blacklist.iter().map(|s| s.as_str()))
            .build()
            .unwrap();

        assert_eq!(vault.path, *vault_path);
        assert_eq!(vault.editor.as_str(), config.get_editor());
        assert_eq!(vault.ident_key.as_str(), config.ident_key);
        assert_eq!(vault.journal_template.as_str(), config.journal_template);
    }
}
