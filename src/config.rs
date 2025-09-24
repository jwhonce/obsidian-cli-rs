use crate::errors::{ConfigError, Result, VaultError};
use crate::types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_ident_key() -> String {
    "uid".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub blacklist: Vec<String>,
    pub editor: Option<String>,
    #[serde(default = "default_ident_key")]
    pub ident_key: String,
    #[serde(default)]
    pub journal_template: String,
    pub vault: Option<PathBuf>,
    #[serde(default)]
    pub verbose: bool,
}

/// Configuration with type-safe wrappers
#[derive(Debug, Clone)]
pub struct TypedConfig {
    pub blacklist: Vec<BlacklistPattern>,
    pub editor: Option<EditorCommand>,
    pub ident_key: IdentKey,
    pub journal_template: JournalTemplate,
    pub vault: Option<PathBuf>,
    pub verbose: bool,
}

impl From<Config> for TypedConfig {
    fn from(config: Config) -> Self {
        Self {
            blacklist: config
                .blacklist
                .into_iter()
                .map(BlacklistPattern::from)
                .collect(),
            editor: config.editor.map(EditorCommand::from),
            ident_key: IdentKey::from(config.ident_key),
            journal_template: JournalTemplate::from(config.journal_template),
            vault: config.vault,
            verbose: config.verbose,
        }
    }
}

impl Config {
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory
        if let Ok(current) = std::env::current_dir() {
            paths.push(current.join("obsidian-cli.toml"));
        }

        // User config directory
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("obsidian-cli").join("config.toml"));
        }

        // Home directory fallback
        if let Some(home) = dirs::home_dir() {
            paths.push(
                home.join(".config")
                    .join("obsidian-cli")
                    .join("config.toml"),
            );
        }

        paths
    }

    #[must_use]
    pub fn get_editor(&self) -> String {
        if let Some(editor) = &self.editor {
            editor.clone()
        } else if let Ok(env_editor) = std::env::var("EDITOR") {
            env_editor
        } else {
            "vi".to_string()
        }
    }

    pub fn load() -> Result<Self> {
        let config_paths = Self::get_config_paths();

        for path in &config_paths {
            if path.exists() {
                return Self::load_from_path(path).map_err(|e| e.into());
            }
        }

        // No config file found, use defaults
        Ok(Self::default())
    }

    pub fn load_from_path(path: &Path) -> std::result::Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(ConfigError::IoError)?;

        let config: Self = toml::from_str(&contents).map_err(ConfigError::InvalidToml)?;

        Ok(config)
    }

    pub fn resolve_vault_path(&self, vault_arg: Option<&Path>) -> Result<PathBuf> {
        let vault_path = vault_arg
            .map(std::path::Path::to_path_buf)
            .or_else(|| self.vault.clone())
            .ok_or_else(|| ConfigError::MissingField {
                field: "vault".to_string(),
            })?;

        let expanded = shellexpand::full(&vault_path.to_string_lossy())
            .map_err(|_| ConfigError::PathExpansion {
                path: format!("{}", vault_path.display()),
            })?
            .into_owned();

        let expanded_path = PathBuf::from(&expanded);

        if !expanded_path.exists() {
            return Err(VaultError::NotFound { path: expanded }.into());
        }

        let vault = expanded_path
            .canonicalize()
            .map_err(|_| VaultError::AccessDenied { path: expanded })?;

        if !vault.is_dir() {
            return Err(VaultError::NotDirectory {
                path: format!("{}", vault.display()),
            }
            .into());
        }

        let obsidian_dir = vault.join(".obsidian");
        if !obsidian_dir.exists() {
            return Err(VaultError::InvalidVault {
                path: format!("{}", vault.display()),
            }
            .into());
        }

        Ok(vault)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blacklist: vec![
                "Assets/".to_string(),
                ".obsidian/".to_string(),
                ".git/".to_string(),
            ],
            editor: Some("vi".to_string()),
            ident_key: "uid".to_string(),
            journal_template: "Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}".to_string(),
            vault: None,
            verbose: false,
        }
    }
}

impl Default for TypedConfig {
    fn default() -> Self {
        Config::default().into()
    }
}
