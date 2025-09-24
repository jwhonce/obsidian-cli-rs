//! Obsidian CLI Library
//!
//! This crate provides a command-line interface for interacting with Obsidian vaults.
//! It includes functionality for managing notes, frontmatter, and vault operations.

pub mod cli;
pub mod commands;
pub mod config;
pub mod errors;
pub mod frontmatter;
pub mod template;
pub mod types;
pub mod utils;

pub mod mcp_server;

// Re-export commonly used types
pub use cli::Cli;
pub use config::Config;
pub use errors::{ConfigError, ObsidianError, Result, TemplateError, VaultError};
// Re-export frontmatter functions for backward compatibility
pub use frontmatter::*;
pub use types::{
    BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, TemplateVars, Vault, VaultInfo,
};
