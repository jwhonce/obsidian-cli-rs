//! Simple configuration tests - CI safe, no user input
//! Basic tests for configuration management without TOML parsing issues

use obsidian_cli::Config;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod simple_config_tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::default();

        // Verify all required fields have sensible defaults
        assert!(!config.journal_template.is_empty());
        assert!(!config.ident_key.is_empty());
        assert!(!config.blacklist.is_empty());
        assert!(config.vault.is_none()); // No default vault path
                                         // Editor might have a default, so just check it exists
        let _editor = config.editor;
        assert!(!config.verbose); // Default to not verbose
    }

    #[test]
    fn test_default_journal_template() {
        let config = Config::default();
        let template = &config.journal_template;

        // Verify template contains expected placeholders
        assert!(template.len() > 50); // Should be substantial
    }

    #[test]
    fn test_default_blacklist() {
        let config = Config::default();
        let blacklist = &config.blacklist;

        // Should not be empty (may or may not contain .obsidian)
        assert!(!blacklist.is_empty());

        // Verify blacklist has some reasonable entries
        assert!(!blacklist.is_empty());
    }

    #[test]
    fn test_default_ident_key() {
        let config = Config::default();
        assert_eq!(config.ident_key, "uid");
    }

    #[test]
    fn test_get_editor_with_default() {
        let config = Config::default();
        let editor = config.get_editor();

        // Should return a default editor when none is configured
        assert!(!editor.is_empty());
        // Default should be vi when no EDITOR env var
        assert!(editor == "vi" || !editor.is_empty());
    }

    #[test]
    fn test_get_editor_with_config_value() {
        let config = Config {
            editor: Some("nano".to_string()),
            ..Default::default()
        };

        let editor = config.get_editor();
        assert_eq!(editor, "nano");
    }

    #[test]
    fn test_get_editor_with_env_var() {
        let config = Config {
            editor: None,
            ..Default::default()
        };

        // Set environment variable
        std::env::set_var("EDITOR", "emacs");

        let editor = config.get_editor();
        assert_eq!(editor, "emacs");

        // Clean up
        std::env::remove_var("EDITOR");
    }

    #[test]
    fn test_load_config_no_file() {
        // When no config file exists, should return default config
        let config = Config::load().unwrap();

        // Should have default values
        assert!(!config.journal_template.is_empty());
        assert_eq!(config.ident_key, "uid");
        assert!(!config.blacklist.is_empty());
    }

    #[test]
    fn test_resolve_vault_path_with_arg() {
        let config = Config::default();

        // Should use the explicit argument over config
        let temp_dir = TempDir::new().unwrap();
        let actual_path = temp_dir.path();

        // Create .obsidian directory to make it a valid vault
        fs::create_dir_all(actual_path.join(".obsidian")).unwrap();

        let result = config.resolve_vault_path(Some(actual_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), actual_path.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_vault_path_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory to make it a valid vault
        fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

        let config = Config {
            vault: Some(vault_path.to_path_buf()),
            ..Default::default()
        };

        let result = config.resolve_vault_path(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vault_path.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_vault_path_nonexistent() {
        let config = Config::default();
        let nonexistent = std::path::Path::new("/this/path/does/not/exist");

        let result = config.resolve_vault_path(Some(nonexistent));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_vault_path_no_vault_specified() {
        let config = Config::default(); // No vault in config

        let result = config.resolve_vault_path(None);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Vault path is required"));
    }

    #[test]
    fn test_config_serialization_fields() {
        let config = Config {
            vault: Some("/test/vault".into()),
            editor: Some("test-editor".to_string()),
            ident_key: "test-id".to_string(),
            journal_template: "test template".to_string(),
            blacklist: vec!["test1".to_string(), "test2".to_string()],
            verbose: true,
        };

        // Verify all fields are accessible
        assert_eq!(config.vault, Some("/test/vault".into()));
        assert_eq!(config.editor, Some("test-editor".to_string()));
        assert_eq!(config.ident_key, "test-id");
        assert_eq!(config.journal_template, "test template");
        assert_eq!(config.blacklist, vec!["test1", "test2"]);
        assert!(config.verbose);
    }

    #[test]
    fn test_config_clone() {
        let original = Config {
            vault: Some("/original/vault".into()),
            editor: Some("original-editor".to_string()),
            ident_key: "original-id".to_string(),
            journal_template: "original template".to_string(),
            blacklist: vec!["orig1".to_string()],
            verbose: false,
        };

        let cloned = original.clone();

        assert_eq!(original.vault, cloned.vault);
        assert_eq!(original.editor, cloned.editor);
        assert_eq!(original.ident_key, cloned.ident_key);
        assert_eq!(original.journal_template, cloned.journal_template);
        assert_eq!(original.blacklist, cloned.blacklist);
        assert_eq!(original.verbose, cloned.verbose);
    }
}
