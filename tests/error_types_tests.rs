//! Tests for granular error types
//! Tests for ConfigError, TemplateError, VaultError and their display/conversion

use obsidian_cli::errors::{ConfigError, ObsidianError, TemplateError, VaultError};
use std::io;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_variants() {
        let not_found = ConfigError::NotFound {
            path: "/path/to/config".to_string(),
        };
        assert_eq!(
            format!("{}", not_found),
            "Config file not found at /path/to/config"
        );

        let missing_field = ConfigError::MissingField {
            field: "vault".to_string(),
        };
        assert_eq!(
            format!("{}", missing_field),
            "Missing required configuration field: vault"
        );

        let invalid_value = ConfigError::InvalidValue {
            field: "editor".to_string(),
            value: "invalid_editor".to_string(),
        };
        assert_eq!(
            format!("{}", invalid_value),
            "Invalid configuration value for editor: invalid_editor"
        );

        let path_expansion = ConfigError::PathExpansion {
            path: "~/invalid/path".to_string(),
        };
        assert_eq!(
            format!("{}", path_expansion),
            "Failed to expand path: ~/invalid/path"
        );
    }

    #[test]
    fn test_config_error_from_toml() {
        // Create a TOML parsing error by trying to parse invalid TOML
        let invalid_toml = "invalid toml [[[";
        let toml_error = toml::from_str::<serde_json::Value>(invalid_toml).unwrap_err();
        let config_error = ConfigError::InvalidToml(toml_error);

        let error_string = format!("{}", config_error);
        assert!(error_string.contains("Invalid TOML syntax"));
    }

    #[test]
    fn test_config_error_from_io() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let config_error = ConfigError::IoError(io_error);

        let error_string = format!("{}", config_error);
        assert!(error_string.contains("Failed to read config file"));
    }

    #[test]
    fn test_vault_error_variants() {
        let not_found = VaultError::NotFound {
            path: "/missing/vault".to_string(),
        };
        assert_eq!(
            format!("{}", not_found),
            "Vault directory does not exist: /missing/vault"
        );

        let invalid_vault = VaultError::InvalidVault {
            path: "/not/a/vault".to_string(),
        };
        assert_eq!(
            format!("{}", invalid_vault),
            "Invalid Obsidian vault: missing .obsidian directory in /not/a/vault"
        );

        let not_directory = VaultError::NotDirectory {
            path: "/file/not/dir".to_string(),
        };
        assert_eq!(
            format!("{}", not_directory),
            "Vault path must be a directory: /file/not/dir"
        );

        let access_denied = VaultError::AccessDenied {
            path: "/no/access".to_string(),
        };
        assert_eq!(
            format!("{}", access_denied),
            "Cannot access vault directory: /no/access"
        );
    }

    #[test]
    fn test_template_error_variants() {
        let invalid_format = TemplateError::InvalidFormatSpecifier {
            spec: "invalid".to_string(),
        };
        assert_eq!(
            format!("{}", invalid_format),
            "Invalid format specifier: invalid"
        );

        let var_not_found = TemplateError::VariableNotFound {
            var: "missing_var".to_string(),
        };
        assert_eq!(
            format!("{}", var_not_found),
            "Variable not found: missing_var"
        );

        let invalid_syntax = TemplateError::InvalidSyntax {
            message: "bad template".to_string(),
        };
        assert_eq!(
            format!("{}", invalid_syntax),
            "Invalid template syntax: bad template"
        );

        let datetime_conversion = TemplateError::DateTimeConversion {
            message: "invalid date".to_string(),
        };
        assert_eq!(
            format!("{}", datetime_conversion),
            "Date/time conversion error: invalid date"
        );
    }

    #[test]
    fn test_obsidian_error_from_config_error() {
        let config_error = ConfigError::MissingField {
            field: "test".to_string(),
        };
        let obsidian_error: ObsidianError = config_error.into();

        let error_string = format!("{}", obsidian_error);
        assert!(error_string.contains("Configuration error"));
        assert!(error_string.contains("Missing required configuration field: test"));
    }

    #[test]
    fn test_obsidian_error_from_vault_error() {
        let vault_error = VaultError::NotFound {
            path: "/test/path".to_string(),
        };
        let obsidian_error: ObsidianError = vault_error.into();

        let error_string = format!("{}", obsidian_error);
        assert!(error_string.contains("Vault error"));
        assert!(error_string.contains("Vault directory does not exist: /test/path"));
    }

    #[test]
    fn test_obsidian_error_from_template_error() {
        let template_error = TemplateError::VariableNotFound {
            var: "test_var".to_string(),
        };
        let obsidian_error: ObsidianError = template_error.into();

        let error_string = format!("{}", obsidian_error);
        assert!(error_string.contains("Template error"));
        assert!(error_string.contains("Variable not found: test_var"));
    }

    #[test]
    fn test_obsidian_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let obsidian_error: ObsidianError = io_error.into();

        let error_string = format!("{}", obsidian_error);
        assert!(error_string.contains("IO error"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let config_error = ConfigError::InvalidValue {
            field: "test".to_string(),
            value: "bad".to_string(),
        };
        let debug_string = format!("{:?}", config_error);
        assert!(debug_string.contains("InvalidValue"));
        assert!(debug_string.contains("test"));
        assert!(debug_string.contains("bad"));
    }

    #[test]
    fn test_error_equality() {
        let error1 = ConfigError::MissingField {
            field: "vault".to_string(),
        };
        let error2 = ConfigError::MissingField {
            field: "vault".to_string(),
        };
        let error3 = ConfigError::MissingField {
            field: "editor".to_string(),
        };

        // Note: These errors don't implement PartialEq, so we test their string representations
        assert_eq!(format!("{}", error1), format!("{}", error2));
        assert_ne!(format!("{}", error1), format!("{}", error3));
    }
}
