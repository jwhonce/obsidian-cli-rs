//! Tests for the info and ls command rendering helpers to improve coverage.

use colored::control::set_override;
use obsidian_cli::commands::info::render_info_output;
use obsidian_cli::commands::ls::render_ls_output;
use obsidian_cli::types::{
    BlacklistPattern, EditorCommand, FileTypeStat, IdentKey, JournalTemplate, Vault, VaultInfo,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn build_test_vault(path: PathBuf) -> Vault {
    Vault {
        path,
        blacklist: Vec::new(),
        editor: EditorCommand::from("true"),
        ident_key: IdentKey::from("uid"),
        journal_template: JournalTemplate::from("Journal/{year}/{month:02}/{day:02}"),
        verbose: false,
    }
}

fn build_vault_info(file_stats: HashMap<String, FileTypeStat>, total_files: usize) -> VaultInfo {
    VaultInfo {
        vault_path: PathBuf::from("/tmp/test-vault"),
        total_files,
        total_directories: 3,
        usage_files: 1024,
        usage_directories: 2048,
        file_type_stats: file_stats,
        markdown_files: total_files,
        blacklist: vec![BlacklistPattern::from("test_blacklist")],
        editor: EditorCommand::from("nvim"),
        journal_template: JournalTemplate::from("Journal/{year}/{month}/{day}"),
        journal_path: "Journal/2025/09/24".to_string(),
        verbose: true,
        version: "0.1.1".to_string(),
    }
}

#[test]
fn test_render_info_output_with_file_stats() {
    set_override(false);

    let mut stats = HashMap::new();
    stats.insert(
        "md".to_string(),
        FileTypeStat {
            count: 3,
            total_size: 3000,
        },
    );
    stats.insert(
        "(no extension)".to_string(),
        FileTypeStat {
            count: 1,
            total_size: 42,
        },
    );

    let output = render_info_output(&build_vault_info(stats, 3));

    assert!(output.contains("OBSIDIAN VAULT INFORMATION"));
    assert!(output.contains("File Types by Extension"));
    assert!(output.contains("md"));
    assert!(output.contains("TOTAL"));
    assert!(output.contains("Vault Blacklist"));
    assert!(output.contains("test_blacklist"));
    assert!(output.contains("Journal/2025/09/24"));
}

#[test]
fn test_render_info_output_no_files() {
    set_override(false);

    let output = render_info_output(&build_vault_info(HashMap::new(), 0));

    assert!(output.contains("No files found in vault"));
    assert!(output.contains("Vault Blacklist"));
}

#[test]
fn test_render_ls_output_with_dates() {
    set_override(false);

    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().to_path_buf();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();
    fs::write(vault_path.join("first.md"), "# First note").unwrap();
    fs::write(vault_path.join("second.md"), "# Second note").unwrap();
    fs::write(vault_path.join("ignore.txt"), "not markdown").unwrap();

    let vault = build_test_vault(vault_path.clone());
    let output = render_ls_output(&vault, true);

    assert!(output.contains("Vault Files with Dates"));
    assert!(output.contains("first.md"));
    assert!(output.contains("second.md"));
}

#[test]
fn test_render_ls_output_without_dates() {
    set_override(false);

    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().to_path_buf();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();
    fs::write(vault_path.join("note.md"), "# Note").unwrap();

    let vault = build_test_vault(vault_path.clone());
    let output = render_ls_output(&vault, false);

    assert!(output.contains("note.md"));
    assert!(!output.contains("Vault Files with Dates"));
}

#[test]
fn test_render_ls_output_no_markdown_files() {
    set_override(false);

    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().to_path_buf();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();
    fs::write(vault_path.join("readme.txt"), "not markdown").unwrap();

    let vault = build_test_vault(vault_path.clone());
    let output = render_ls_output(&vault, true);

    assert!(output.contains("No markdown files found in vault"));
}
