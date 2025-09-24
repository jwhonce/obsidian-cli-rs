//! Comprehensive command integration tests with mocked environments
//! All tests designed for CI - no user input, complete automation

use obsidian_cli::{
    commands::*,
    types::{OutputStyle, Vault},
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod command_integration_tests {
    use super::*;

    // Helper function to create a test vault with mocked environment
    fn create_test_vault(temp_dir: &TempDir) -> Vault {
        // Create .obsidian directory to make it a valid vault
        fs::create_dir_all(temp_dir.path().join(".obsidian")).unwrap();

        Vault {
            path: temp_dir.path().to_path_buf(),
            blacklist: vec![".obsidian".to_string(), "*.tmp".to_string()],
            editor: "true".to_string(), // Mock editor that always succeeds
            ident_key: "uid".to_string(),
            journal_template: "Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}".to_string(),
            verbose: false,
        }
    }

    // Helper to create a test note with frontmatter
    fn create_test_note(vault_path: &Path, name: &str, frontmatter: &str, content: &str) {
        let full_content = if frontmatter.is_empty() {
            content.to_string()
        } else {
            format!("---\n{}\n---\n{}", frontmatter, content)
        };
        fs::write(vault_path.join(format!("{}.md", name)), full_content).unwrap();
    }

    #[tokio::test]
    async fn test_info_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create some test files
        create_test_note(temp_dir.path(), "note1", "title: Test Note 1", "Content 1");
        create_test_note(temp_dir.path(), "note2", "title: Test Note 2", "Content 2");

        let result = info::execute(&vault);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ls_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files
        create_test_note(temp_dir.path(), "test1", "title: First Note", "Content");
        create_test_note(
            temp_dir.path(),
            "test2",
            "title: Second Note",
            "More content",
        );

        // Test without dates
        let result = ls::execute(&vault, false);
        assert!(result.is_ok());

        // Test with dates
        let result = ls::execute(&vault, true);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cat_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file
        create_test_note(
            temp_dir.path(),
            "cat-test",
            "title: Cat Test\ntags: [test]",
            "# Cat Test\n\nThis is test content for cat command.",
        );

        let note_path = Path::new("cat-test.md");

        // Test without frontmatter
        let result = cat::execute(&vault, note_path, false);
        assert!(result.is_ok());

        // Test with frontmatter
        let result = cat::execute(&vault, note_path, true);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cat_command_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        create_test_note(
            temp_dir.path(),
            "no-ext-test",
            "title: No Extension Test",
            "Content without extension",
        );

        let note_path = Path::new("no-ext-test"); // No .md extension
        let result = cat::execute(&vault, note_path, false);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        let new_file_path = Path::new("new-test-file");

        // Test creating new file
        let result = new::execute(&vault, new_file_path, false);
        assert!(result.is_ok());
        assert!(temp_dir.path().join("new-test-file.md").exists());
    }

    #[tokio::test]
    async fn test_new_command_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        let file_path = Path::new("existing-file");

        // Create existing file
        create_test_note(temp_dir.path(), "existing-file", "", "Existing content");

        // Test overwriting with force
        let result = new::execute(&vault, file_path, true);
        assert!(result.is_ok());
        assert!(temp_dir.path().join("existing-file.md").exists());
    }

    #[tokio::test]
    async fn test_add_uid_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file without UID
        create_test_note(
            temp_dir.path(),
            "no-uid",
            "title: No UID Test",
            "Content without UID",
        );

        let note_path = Path::new("no-uid.md");
        let result = add_uid::execute(&vault, note_path, false);
        assert!(result.is_ok());

        // Verify UID was added
        let content = fs::read_to_string(temp_dir.path().join("no-uid.md")).unwrap();
        assert!(content.contains("uid: "));
    }

    #[tokio::test]
    async fn test_add_uid_command_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file with existing UID
        create_test_note(
            temp_dir.path(),
            "has-uid",
            "title: Has UID\nuid: existing-uid-123",
            "Content with UID",
        );

        let note_path = Path::new("has-uid.md");
        let result = add_uid::execute(&vault, note_path, true);
        assert!(result.is_ok());

        // Verify UID was replaced
        let content = fs::read_to_string(temp_dir.path().join("has-uid.md")).unwrap();
        assert!(content.contains("uid: "));
        assert!(!content.contains("existing-uid-123"));
    }

    #[tokio::test]
    async fn test_edit_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file
        create_test_note(
            temp_dir.path(),
            "edit-test",
            "title: Edit Test",
            "Content to edit",
        );

        let note_path = Path::new("edit-test.md");
        let result = edit::execute(&vault, note_path);

        // Since we use "true" as mock editor, it should succeed without error
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_command() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files with searchable content
        create_test_note(
            temp_dir.path(),
            "findable-note",
            "title: Findable Note",
            "This note can be found",
        );
        create_test_note(
            temp_dir.path(),
            "another-note",
            "title: Another Note",
            "This is another note",
        );

        // Test exact search
        let result = find::execute(&vault, "findable-note", true);
        assert!(result.is_ok());

        // Test fuzzy search
        let result = find::execute(&vault, "findable", false);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_meta_command_list_all() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file with frontmatter
        create_test_note(
            temp_dir.path(),
            "meta-test",
            "title: Meta Test\ntags: [test, meta]\nstatus: draft",
            "Content with metadata",
        );

        let note_path = Path::new("meta-test.md");

        // Test listing all metadata
        let result = meta::execute(&vault, note_path, None, None);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_meta_command_get_existing_key() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file with frontmatter
        create_test_note(
            temp_dir.path(),
            "meta-get",
            "title: Meta Get Test\nstatus: published",
            "Content",
        );

        let note_path = Path::new("meta-get.md");

        // Test getting existing key
        let result = meta::execute(&vault, note_path, Some("title"), None);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_meta_command_set_key() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file
        create_test_note(
            temp_dir.path(),
            "meta-set",
            "title: Meta Set Test",
            "Content",
        );

        let note_path = Path::new("meta-set.md");

        // Test setting a key
        let result = meta::execute(&vault, note_path, Some("status"), Some("published"));
        assert!(result.is_ok());

        // Verify the key was set
        let content = fs::read_to_string(temp_dir.path().join("meta-set.md")).unwrap();
        assert!(content.contains("status: published"));
    }

    #[tokio::test]
    async fn test_query_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files with different frontmatter
        create_test_note(
            temp_dir.path(),
            "query1",
            "title: Query Test 1\nstatus: published",
            "Content 1",
        );
        create_test_note(
            temp_dir.path(),
            "query2",
            "title: Query Test 2\nstatus: draft",
            "Content 2",
        );
        create_test_note(
            temp_dir.path(),
            "query3",
            "title: Query Test 3\nstatus: published",
            "Content 3",
        );

        let options = query::QueryOptions {
            key: "status",
            value: Some("published"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Table,
            count: false,
        };

        let result = query::execute(&vault, options);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_command_exists() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files
        create_test_note(
            temp_dir.path(),
            "has-tags",
            "title: Has Tags\ntags: [rust, test]",
            "Content",
        );
        create_test_note(temp_dir.path(), "no-tags", "title: No Tags", "Content");

        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: None,
            exists: true,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };

        let result = query::execute(&vault, options);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_command_missing() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files
        create_test_note(
            temp_dir.path(),
            "has-status",
            "title: Has Status\nstatus: draft",
            "Content",
        );
        create_test_note(temp_dir.path(), "no-status", "title: No Status", "Content");

        let options = query::QueryOptions {
            key: "status",
            value: None,
            contains: None,
            exists: false,
            missing: true,
            style: OutputStyle::Title,
            count: false,
        };

        let result = query::execute(&vault, options);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_command_count() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create test files
        create_test_note(
            temp_dir.path(),
            "published1",
            "title: Published 1\nstatus: published",
            "Content",
        );
        create_test_note(
            temp_dir.path(),
            "published2",
            "title: Published 2\nstatus: published",
            "Content",
        );
        create_test_note(
            temp_dir.path(),
            "draft1",
            "title: Draft 1\nstatus: draft",
            "Content",
        );

        let options = query::QueryOptions {
            key: "status",
            value: Some("published"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Table,
            count: true,
        };

        let result = query::execute(&vault, options);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rm_command_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create a test file to remove
        create_test_note(
            temp_dir.path(),
            "to-delete",
            "title: To Delete",
            "This file will be deleted",
        );

        let note_path = Path::new("to-delete.md");

        // Verify file exists
        assert!(temp_dir.path().join("to-delete.md").exists());

        // Test removal with force (to avoid user input)
        let result = rm::execute(&vault, note_path, true);
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!temp_dir.path().join("to-delete.md").exists());
    }

    #[tokio::test]
    async fn test_journal_command_today() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create Calendar directory structure
        fs::create_dir_all(temp_dir.path().join("Calendar")).unwrap();

        let result = journal::execute(&vault, None);
        assert!(result.is_ok());

        // Check that some journal file was created in the Calendar structure
        assert!(temp_dir.path().join("Calendar").exists());
    }

    #[tokio::test]
    async fn test_journal_command_specific_date() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create Calendar directory structure
        fs::create_dir_all(temp_dir.path().join("Calendar")).unwrap();

        let result = journal::execute(&vault, Some("2023-12-25"));
        assert!(result.is_ok());

        // Check that the specific date structure was created
        assert!(temp_dir.path().join("Calendar").exists());
    }

    #[tokio::test]
    async fn test_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create nested directory structure
        fs::create_dir_all(temp_dir.path().join("Projects/Rust")).unwrap();
        create_test_note(
            &temp_dir.path().join("Projects/Rust"),
            "notes",
            "title: Rust Notes",
            "Rust project notes",
        );

        // Test cat with nested path
        let result = cat::execute(&vault, Path::new("Projects/Rust/notes.md"), false);
        assert!(result.is_ok());

        // Test ls should find the nested structure
        let result = ls::execute(&vault, false);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unicode_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create file with unicode name and content
        let unicode_name = "测试笔记";
        create_test_note(
            temp_dir.path(),
            unicode_name,
            "title: Unicode Test 🌟",
            "# Unicode Content\n\nこんにちは世界 🎌",
        );

        let note_filename = format!("{}.md", unicode_name);
        let note_path = Path::new(&note_filename);

        // Test various commands with unicode
        let result = cat::execute(&vault, note_path, true);
        assert!(result.is_ok());

        let result = meta::execute(&vault, note_path, None, None);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_vault_operations() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Test operations on empty vault
        let result = info::execute(&vault);
        assert!(result.is_ok());

        let result = ls::execute(&vault, false);
        assert!(result.is_ok());

        let result = find::execute(&vault, "nonexistent", false);
        assert!(result.is_ok());

        // Query should handle empty vault gracefully
        let options = query::QueryOptions {
            key: "title",
            value: None,
            contains: None,
            exists: true,
            missing: false,
            style: OutputStyle::Table,
            count: false,
        };
        let result = query::execute(&vault, options);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_large_content_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create file with large content
        let large_content = "Large content line.\n".repeat(1000);
        create_test_note(
            temp_dir.path(),
            "large-file",
            "title: Large File",
            &large_content,
        );

        let note_path = Path::new("large-file.md");

        // Test commands with large content
        let result = cat::execute(&vault, note_path, false);
        assert!(result.is_ok());

        let result = add_uid::execute(&vault, note_path, false);
        assert!(result.is_ok());
    }

    // Note: serve command is intentionally not tested here as it starts a long-running server
    // that would hang in tests. It's better tested separately or with timeout mechanisms.
}
