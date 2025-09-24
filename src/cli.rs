use crate::commands::*;
use crate::config::Config;
use crate::errors::Result;
use crate::types::Vault;
use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Macro to resolve page_or_path to a file_path for commands that operate on existing files
///
/// Simplifies the common pattern in commands that operate on existing files by
/// automatically calling resolve_page_path(page_or_path, &vault.path).
///
/// Usage: `let file_path = crate::resolve_page_or_path!(vault, page_or_path)?;`
#[macro_export]
macro_rules! resolve_page_or_path {
    ($vault:expr, $page_or_path:expr) => {
        $crate::utils::resolve_page_path($page_or_path, &$vault.path)
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "obsidian-cli")]
pub struct Cli {
    /// Path to the Obsidian vault
    #[arg(short, long, env = "OBSIDIAN_VAULT")]
    vault: Option<PathBuf>,

    /// Configuration file to read configuration from
    #[arg(short, long, env = "OBSIDIAN_CONFIG")]
    config: Option<PathBuf>,

    /// Colon-separated list of directories to ignore
    #[arg(short, long, env = "OBSIDIAN_BLACKLIST", value_delimiter = ':')]
    blacklist: Option<Vec<String>>,

    /// Path for editor to use for editing journal entries
    #[arg(short, long, env = "EDITOR")]
    editor: Option<String>,

    /// Enable verbose output
    #[arg(long, env = "OBSIDIAN_VERBOSE")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a unique ID to a page's frontmatter
    AddUid {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
        /// If set, overwrite existing uid
        #[arg(short, long)]
        force: bool,
    },
    /// Display the contents of a file
    Cat {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
        /// If set, show frontmatter in addition to file content
        #[arg(short, long)]
        show_frontmatter: bool,
    },
    /// Edit any file with the configured editor
    Edit {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
    },
    /// Find files by name or title with exact/fuzzy matching
    Find {
        /// Obsidian Page to use in search
        page_name: String,
        /// Require exact match on page name
        #[arg(short, long)]
        exact: bool,
    },
    /// Display vault and configuration information
    Info,
    /// Open a journal entry (optionally for a specific --date)
    Journal {
        /// Date to open in YYYY-MM-DD format; defaults to today if omitted
        #[arg(short, long)]
        date: Option<String>,
    },
    /// List markdown files in the vault, respecting the blacklist
    Ls {
        /// Display created and modified dates for each file
        #[arg(long)]
        date: bool,
    },
    /// View or update frontmatter metadata
    #[command(visible_alias = "frontmatter")]
    Meta {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
        /// Key of the frontmatter metadata to view or update
        #[arg(short, long)]
        key: Option<String>,
        /// New metadata for given key. If unset, list current metadata of key
        #[arg(short, long)]
        value: Option<String>,
    },
    /// Create a new file in the vault
    New {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
        /// Overwrite existing file with new contents
        #[arg(short, long)]
        force: bool,
    },
    /// Query frontmatter across all files
    Query {
        /// Frontmatter key to query across Vault
        key: String,
        /// Find pages where the key's metadata exactly matches this string
        #[arg(short, long)]
        value: Option<String>,
        /// Find pages where the key's metadata contains this substring
        #[arg(long)]
        contains: Option<String>,
        /// Find pages where the key exists
        #[arg(long)]
        exists: bool,
        /// Find pages where the key is missing
        #[arg(long)]
        missing: bool,
        /// Output format style
        #[arg(short, long, value_enum, default_value = "path")]
        style: OutputStyleArg,
        /// Only show count of matching pages
        #[arg(long)]
        count: bool,
    },
    /// Rename a file and optionally update wiki links
    Rename {
        /// Obsidian page name or Path to file to rename
        page_or_path: PathBuf,
        /// New name for the file
        new_name: String,
        /// Search and update wiki links to the renamed file
        #[arg(short, long)]
        link: bool,
    },
    /// Remove a file from the vault
    Rm {
        /// Obsidian page name or Path to file
        page_or_path: PathBuf,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Start an MCP (Model Context Protocol) server
    Serve,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputStyleArg {
    Path,
    Title,
    Table,
    Json,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let config = if let Some(config_path) = &self.config {
            Config::load_from_path(config_path)
                .with_context(|| format!("Failed to load config from {}", config_path.display()))?
        } else {
            Config::load().context("Failed to load configuration")?
        };

        let vault = config
            .resolve_vault_path(self.vault.as_deref())
            .context("Failed to resolve vault path")?;

        let blacklist = self.blacklist.unwrap_or_else(|| config.blacklist.clone());
        let editor = self.editor.unwrap_or_else(|| config.get_editor());

        let vault = Vault {
            path: vault,
            blacklist,
            editor,
            ident_key: config.ident_key,
            journal_template: config.journal_template,
            verbose: self.verbose || config.verbose,
        };

        match self.command {
            Commands::AddUid {
                page_or_path,
                force,
            } => add_uid::execute(&vault, &page_or_path, force).await,
            Commands::Cat {
                page_or_path,
                show_frontmatter,
            } => cat::execute(&vault, &page_or_path, show_frontmatter).await,
            Commands::Edit { page_or_path } => edit::execute(&vault, &page_or_path).await,
            Commands::Find { page_name, exact } => find::execute(&vault, &page_name, exact).await,
            Commands::Info => info::execute(&vault).await,
            Commands::Journal { date } => journal::execute(&vault, date.as_deref()).await,
            Commands::Ls { date } => ls::execute(&vault, date).await,
            Commands::Meta {
                page_or_path,
                key,
                value,
            } => meta::execute(&vault, &page_or_path, key.as_deref(), value.as_deref()).await,
            Commands::New {
                page_or_path,
                force,
            } => new::execute(&vault, &page_or_path, force).await,
            Commands::Query {
                key,
                value,
                contains,
                exists,
                missing,
                style,
                count,
            } => {
                let options = query::QueryOptions {
                    key: &key,
                    value: value.as_deref(),
                    contains: contains.as_deref(),
                    exists,
                    missing,
                    style: style.into(),
                    count,
                };
                query::execute(&vault, options).await
            }
            Commands::Rename {
                page_or_path,
                new_name,
                link,
            } => rename::execute(&vault, &page_or_path, &new_name, link).await,
            Commands::Rm {
                page_or_path,
                force,
            } => rm::execute(&vault, &page_or_path, force).await,
            Commands::Serve => serve::execute(&vault).await,
        }
    }
}

impl From<OutputStyleArg> for crate::types::OutputStyle {
    fn from(style: OutputStyleArg) -> Self {
        match style {
            OutputStyleArg::Json => crate::types::OutputStyle::Json,
            OutputStyleArg::Path => crate::types::OutputStyle::Path,
            OutputStyleArg::Table => crate::types::OutputStyle::Table,
            OutputStyleArg::Title => crate::types::OutputStyle::Title,
        }
    }
}
