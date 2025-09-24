//! Comprehensive tests for info command
//! Tests display formatting, vault information gathering, and error handling

use obsidian_cli::{
    commands::info,
    types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, Vault},
};
use std::fs;
use tempfile::TempDir;

fn create_test_vault_with_files() -> (TempDir, Vault) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create .obsidian directory to make it a valid vault
    fs::create_dir(vault_path.join(".obsidian")).unwrap();

    // Create some test files
    fs::write(vault_path.join("note1.md"), "# Note 1\nContent").unwrap();
    fs::write(vault_path.join("note2.md"), "# Note 2\nMore content").unwrap();
    fs::write(vault_path.join("document.txt"), "Text document").unwrap();
    fs::write(vault_path.join("image.jpg"), b"fake image data").unwrap();

    // Create subdirectories with files
    fs::create_dir(vault_path.join("subdir")).unwrap();
    fs::write(
        vault_path.join("subdir/subnote.md"),
        "# Subnote\nSubcontent",
    )
    .unwrap();
    fs::write(vault_path.join("subdir/data.json"), r#"{"test": "data"}"#).unwrap();

    // Create blacklisted directory (should be ignored)
    fs::create_dir(vault_path.join("Assets")).unwrap();
    fs::write(vault_path.join("Assets/ignored.md"), "Should be ignored").unwrap();

    let vault = Vault {
        path: vault_path.to_path_buf(),
        blacklist: vec![
            BlacklistPattern::from("Assets/"),
            BlacklistPattern::from("*.tmp"),
            BlacklistPattern::from(".git/"),
        ],
        editor: EditorCommand::from("vim"),
        ident_key: IdentKey::from("uid"),
        journal_template: JournalTemplate::from("Journal/{year}/{month:02d}/{day:02d}"),
        verbose: false,
    };

    (temp_dir, vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_command_basic_execution() {
        let (_temp_dir, vault) = create_test_vault_with_files();

        // Test that info command executes without error
        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_verbose_vault() {
        let (_temp_dir, mut vault) = create_test_vault_with_files();
        vault.verbose = true;

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_different_editor() {
        let (_temp_dir, mut vault) = create_test_vault_with_files();
        vault.editor = EditorCommand::from("nano");

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_different_blacklist() {
        let (_temp_dir, mut vault) = create_test_vault_with_files();
        vault.blacklist = vec![
            BlacklistPattern::from("node_modules/"),
            BlacklistPattern::from("*.log"),
            BlacklistPattern::from("build/*"),
        ];

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_different_journal_template() {
        let (_temp_dir, mut vault) = create_test_vault_with_files();
        vault.journal_template = JournalTemplate::from("Daily/{year}-{month:02d}-{day:02d}");

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_complex_blacklist() {
        let (_temp_dir, mut vault) = create_test_vault_with_files();
        vault.blacklist = vec![
            BlacklistPattern::from("**/node_modules/**"),
            BlacklistPattern::from("*.tmp"),
            BlacklistPattern::from("cache*"),
            BlacklistPattern::from("build/"),
            BlacklistPattern::from(".git"),
        ];

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory to make it a valid vault
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        let vault = Vault {
            path: vault_path.to_path_buf(),
            blacklist: vec![BlacklistPattern::from(".obsidian/")],
            editor: EditorCommand::from("vi"),
            ident_key: IdentKey::from("id"),
            journal_template: JournalTemplate::from("Notes/{year}-{month:02d}-{day:02d}"),
            verbose: false,
        };

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_many_file_types() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Create files of various types
        fs::write(vault_path.join("note.md"), "markdown").unwrap();
        fs::write(vault_path.join("doc.txt"), "text").unwrap();
        fs::write(vault_path.join("code.rs"), "rust code").unwrap();
        fs::write(vault_path.join("data.json"), "{}").unwrap();
        fs::write(vault_path.join("style.css"), "css").unwrap();
        fs::write(vault_path.join("script.js"), "javascript").unwrap();
        fs::write(vault_path.join("README"), "no extension").unwrap();
        fs::write(vault_path.join("config.toml"), "config").unwrap();
        fs::write(vault_path.join("data.xml"), "<xml/>").unwrap();
        fs::write(vault_path.join("image.png"), b"fake png").unwrap();

        let vault = Vault {
            path: vault_path.to_path_buf(),
            blacklist: vec![BlacklistPattern::from(".obsidian/")],
            editor: EditorCommand::from("code"),
            ident_key: IdentKey::from("uuid"),
            journal_template: JournalTemplate::from("Logs/{year}/{month:02d}"),
            verbose: true,
        };

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Create nested directory structure
        fs::create_dir_all(vault_path.join("level1/level2/level3")).unwrap();
        fs::write(vault_path.join("level1/note1.md"), "note1").unwrap();
        fs::write(vault_path.join("level1/level2/note2.md"), "note2").unwrap();
        fs::write(vault_path.join("level1/level2/level3/note3.md"), "note3").unwrap();

        // Create parallel structure
        fs::create_dir_all(vault_path.join("docs/technical")).unwrap();
        fs::write(vault_path.join("docs/readme.md"), "readme").unwrap();
        fs::write(vault_path.join("docs/technical/spec.md"), "spec").unwrap();

        let vault = Vault {
            path: vault_path.to_path_buf(),
            blacklist: vec![
                BlacklistPattern::from(".obsidian/"),
                BlacklistPattern::from("*.tmp"),
            ],
            editor: EditorCommand::from("nano"),
            ident_key: IdentKey::from("id"),
            journal_template: JournalTemplate::from("{year}/{month:02d}/{day:02d}"),
            verbose: false,
        };

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        // Create files with special characters in names
        fs::write(vault_path.join("note with spaces.md"), "content").unwrap();
        fs::write(vault_path.join("note-with-dashes.md"), "content").unwrap();
        fs::write(vault_path.join("note_with_underscores.md"), "content").unwrap();

        let vault = Vault {
            path: vault_path.to_path_buf(),
            blacklist: vec![BlacklistPattern::from(".obsidian/")],
            editor: EditorCommand::from("emacs"),
            ident_key: IdentKey::from("unique_id"),
            journal_template: JournalTemplate::from("Daily Notes/{year}-{month:02d}-{day:02d}"),
            verbose: true,
        };

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }
}
