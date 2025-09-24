use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_ident_key() -> String {
    "uid".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub blacklist: Vec<String>,
    pub editor: Option<String>,
    #[serde(default = "default_ident_key")]
    pub ident_key: String,
    pub journal_template: String,
    pub vault: Option<PathBuf>,
    pub verbose: bool,
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
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vi".to_string())
    }

    pub fn load() -> Result<Self> {
        let config_paths = Self::get_config_paths();

        for path in &config_paths {
            if path.exists() {
                return Self::load_from_path(path)
                    .with_context(|| format!("Failed to parse config file: {}", path.display()));
            }
        }

        // No config file found, use defaults
        Ok(Self::default())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file {}", path.display()))?;

        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML in config file {}", path.display()))?;

        Ok(config)
    }

    pub fn resolve_vault_path(&self, vault_arg: Option<&Path>) -> Result<PathBuf> {
        let vault_path = vault_arg
            .map(std::path::Path::to_path_buf)
            .or_else(|| self.vault.clone())
            .context("Vault path is required. Use --vault option, OBSIDIAN_VAULT environment variable, or specify 'vault' in configuration file.")?;

        let expanded = shellexpand::full(&vault_path.to_string_lossy())
            .context("Failed to expand vault path")?
            .into_owned();

        let expanded_path = PathBuf::from(&expanded);

        if !expanded_path.exists() {
            anyhow::bail!("Vault directory does not exist: {}\nMake sure the vault path in your configuration is correct.", expanded);
        }

        let vault = expanded_path
            .canonicalize()
            .with_context(|| format!("Cannot access vault directory: {expanded}"))?;

        if !vault.is_dir() {
            anyhow::bail!("Vault path must be a directory: {}", vault.display());
        }

        let obsidian_dir = vault.join(".obsidian");
        if !obsidian_dir.exists() {
            anyhow::bail!(
                "Invalid Obsidian vault: missing .obsidian directory in {}\nMake sure this is a valid Obsidian vault.", 
                vault.display()
            );
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
