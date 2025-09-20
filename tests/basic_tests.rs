//! Basic tests that work with CI
//! Completely rewritten for automated testing without user input

use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create .obsidian directory
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

    // Create a test note
    let note_content = r#"---
title: Test Note
tags: [test]
---

# Test Note
This is test content.
"#;
    fs::write(vault_path.join("test-note.md"), note_content).unwrap();

    // Verify file was created
    assert!(vault_path.join("test-note.md").exists());

    // Verify content
    let content = fs::read_to_string(vault_path.join("test-note.md")).unwrap();
    assert!(content.contains("Test Note"));
    assert!(content.contains("test content"));
}

#[test]
fn test_frontmatter_parsing() {
    let content = r#"---
title: Test
tags: [example]
---

# Content here
"#;

    let result = obsidian_cli::frontmatter::parse_string(content);
    assert!(result.is_ok());

    let (fm, body) = result.unwrap();
    assert!(fm.contains_key("title"));
    assert!(body.contains("Content here"));
}

#[test]
fn test_config_defaults() {
    let config = obsidian_cli::Config::default();
    assert!(!config.journal_template.is_empty());
    assert!(!config.ident_key.is_empty());
    assert!(!config.blacklist.is_empty());
}

#[test]
fn test_template_vars() {
    let vars = obsidian_cli::utils::get_template_vars(chrono::Local::now());
    // TemplateVars is a struct, not a HashMap
    assert!(vars.year >= 2020);
    assert!(vars.month >= 1 && vars.month <= 12);
    assert!(vars.day >= 1 && vars.day <= 31);
    assert!(!vars.month_name.is_empty());
    assert!(!vars.weekday.is_empty());
}

#[test]
fn test_path_operations() {
    let temp_dir = TempDir::new().unwrap();
    assert!(temp_dir.path().exists());
    assert!(temp_dir.path().is_dir());
}

#[test]
fn test_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    fs::write(&test_file, "test content").unwrap();
    assert!(test_file.exists());

    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_vault_structure() {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create typical vault structure
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();
    fs::create_dir_all(vault_path.join("Daily Notes")).unwrap();
    fs::create_dir_all(vault_path.join("Templates")).unwrap();

    // Create some files
    fs::write(vault_path.join("README.md"), "# My Vault").unwrap();
    fs::write(vault_path.join("Daily Notes/2023-01-01.md"), "# Daily Note").unwrap();

    // Verify structure
    assert!(vault_path.join(".obsidian").is_dir());
    assert!(vault_path.join("Daily Notes").is_dir());
    assert!(vault_path.join("Templates").is_dir());
    assert!(vault_path.join("README.md").exists());
    assert!(vault_path.join("Daily Notes/2023-01-01.md").exists());
}

#[test]
fn test_unicode_content() {
    let temp_dir = TempDir::new().unwrap();
    let unicode_file = temp_dir.path().join("unicode.md");

    let unicode_content = "# Unicode Test 🌟\n\n这是中文 • Français • العربية";
    fs::write(&unicode_file, unicode_content).unwrap();

    let content = fs::read_to_string(&unicode_file).unwrap();
    assert!(content.contains("🌟"));
    assert!(content.contains("这是中文"));
    assert!(content.contains("Français"));
    assert!(content.contains("العربية"));
}

#[test]
fn test_empty_frontmatter() {
    let content = "---\n---\n\n# Just content";
    let result = obsidian_cli::frontmatter::parse_string(content);
    assert!(result.is_ok());

    let (fm, body) = result.unwrap();
    assert!(fm.is_empty());
    assert!(body.contains("Just content"));
}

#[test]
fn test_no_frontmatter() {
    let content = "# Plain markdown\n\nNo frontmatter here.";
    let result = obsidian_cli::frontmatter::parse_string(content);
    assert!(result.is_ok());

    let (fm, body) = result.unwrap();
    assert!(fm.is_empty());
    assert!(body.contains("Plain markdown"));
}
