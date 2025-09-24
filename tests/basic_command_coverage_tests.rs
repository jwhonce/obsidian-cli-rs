//! Tests for basic command coverage - cat, edit, find, ls commands
//! Tests error paths and edge cases for better coverage

use obsidian_cli::{
    commands::{cat, edit, find, ls},
    types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, Vault},
};
use std::fs;
use tempfile::TempDir;

fn create_test_vault_with_content() -> (TempDir, Vault) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create .obsidian directory to make it a valid vault
    fs::create_dir(vault_path.join(".obsidian")).unwrap();

    // Create test files
    fs::write(
        vault_path.join("note1.md"),
        r#"---
title: Test Note 1
author: Test Author
tags: [test, markdown]
---

# Test Note 1

This is test content for note 1."#,
    )
    .unwrap();

    fs::write(
        vault_path.join("note2.md"),
        r#"---
title: Test Note 2  
created: 2025-01-15
---

# Test Note 2

Another test note with different frontmatter."#,
    )
    .unwrap();

    // File without frontmatter
    fs::write(
        vault_path.join("simple.md"),
        "# Simple Note\n\nNo frontmatter here.",
    )
    .unwrap();

    // Create subdirectory with files
    fs::create_dir(vault_path.join("subdir")).unwrap();
    fs::write(
        vault_path.join("subdir/nested.md"),
        "# Nested Note\n\nIn subdirectory.",
    )
    .unwrap();

    // Create non-markdown files
    fs::write(vault_path.join("readme.txt"), "This is a text file.").unwrap();
    fs::write(vault_path.join("data.json"), r#"{"key": "value"}"#).unwrap();

    let vault = Vault {
        path: vault_path.to_path_buf(),
        blacklist: vec![
            BlacklistPattern::from(".obsidian/"),
            BlacklistPattern::from("*.tmp"),
        ],
        editor: EditorCommand::from("true"), // Mock editor that always succeeds
        ident_key: IdentKey::from("uid"),
        journal_template: JournalTemplate::from("Journal/{year}/{month:02d}/{day:02d}"),
        verbose: false,
    };

    (temp_dir, vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cat command tests
    #[test]
    fn test_cat_file_with_frontmatter() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let note_path = vault.path.join("note1.md");

        let result = cat::execute(&vault, &note_path, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cat_file_without_frontmatter() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let note_path = vault.path.join("simple.md");

        let result = cat::execute(&vault, &note_path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cat_nonexistent_file() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let nonexistent = vault.path.join("does_not_exist.md");

        let result = cat::execute(&vault, &nonexistent, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_cat_file_in_subdirectory() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let nested_path = vault.path.join("subdir/nested.md");

        let result = cat::execute(&vault, &nested_path, true);
        assert!(result.is_ok());
    }

    // Edit command tests
    #[test]
    fn test_edit_existing_file() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let note_path = vault.path.join("note1.md");

        let result = edit::execute(&vault, &note_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edit_nonexistent_file() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let nonexistent = vault.path.join("new_file.md");

        let result = edit::execute(&vault, &nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_edit_with_different_editor() {
        let (_temp_dir, mut vault) = create_test_vault_with_content();
        vault.editor = EditorCommand::from("true"); // Use 'true' command which always succeeds
        let note_path = vault.path.join("simple.md");

        let result = edit::execute(&vault, &note_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edit_file_in_subdirectory() {
        let (_temp_dir, vault) = create_test_vault_with_content();
        let nested_path = vault.path.join("subdir/nested.md");

        let result = edit::execute(&vault, &nested_path);
        assert!(result.is_ok());
    }

    // Find command tests
    #[test]
    fn test_find_exact_match() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "note1", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_fuzzy_match() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "test", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_no_matches() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "nonexistent_pattern", true);
        assert!(result.is_ok()); // Should succeed but find nothing
    }

    #[test]
    fn test_find_partial_match() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "note", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_case_insensitive() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "NOTE", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_with_special_characters() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Create a file with special characters
        fs::write(
            vault.path.join("special-file_with.chars.md"),
            "# Special File",
        )
        .unwrap();

        let result = find::execute(&vault, "special-file", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_nested_files() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "nested", false);
        assert!(result.is_ok());
    }

    // LS command tests
    #[test]
    fn test_ls_without_date() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = ls::execute(&vault, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_with_date() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = ls::execute(&vault, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create .obsidian directory to make it a valid vault
        fs::create_dir(vault_path.join(".obsidian")).unwrap();

        let vault = Vault {
            path: vault_path.to_path_buf(),
            blacklist: vec![BlacklistPattern::from(".obsidian/")],
            editor: EditorCommand::from("vi"),
            ident_key: IdentKey::from("uid"),
            journal_template: JournalTemplate::from("test"),
            verbose: false,
        };

        let result = ls::execute(&vault, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_with_blacklisted_files() {
        let (_temp_dir, mut vault) = create_test_vault_with_content();

        // Create some files that should be blacklisted
        fs::create_dir(vault.path.join("node_modules")).unwrap();
        fs::write(vault.path.join("node_modules/package.md"), "# Package").unwrap();
        fs::write(vault.path.join("temp.tmp"), "temporary file").unwrap();

        vault
            .blacklist
            .push(BlacklistPattern::from("node_modules/"));
        vault.blacklist.push(BlacklistPattern::from("*.tmp"));

        let result = ls::execute(&vault, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_with_many_files() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Create many files to test table formatting
        for i in 1..=20 {
            fs::write(
                vault.path.join(format!("bulk_note_{:02}.md", i)),
                format!("# Bulk Note {}\n\nContent for note {}", i, i),
            )
            .unwrap();
        }

        let result = ls::execute(&vault, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_with_long_filenames() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Create files with very long names to test wrapping
        let long_name = "this_is_a_very_long_filename_that_should_trigger_the_wrapping_functionality_in_the_table_display";
        fs::write(
            vault.path.join(format!("{}.md", long_name)),
            "# Long filename test",
        )
        .unwrap();

        let result = ls::execute(&vault, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ls_mixed_file_types() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // The vault already has mixed file types, test that ls handles them
        let result = ls::execute(&vault, false);
        assert!(result.is_ok());

        // Test with date display too
        let result = ls::execute(&vault, true);
        assert!(result.is_ok());
    }

    // Error handling tests
    #[test]
    fn test_commands_with_invalid_paths() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Test with path containing invalid characters
        let invalid_path = vault.path.join("invalid\0path.md");

        let cat_result = cat::execute(&vault, &invalid_path, true);
        assert!(cat_result.is_err());

        let edit_result = edit::execute(&vault, &invalid_path);
        assert!(edit_result.is_err());
    }

    #[test]
    fn test_commands_with_directory_instead_of_file() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Try to cat a directory
        let dir_path = vault.path.join("subdir");

        let result = cat::execute(&vault, &dir_path, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_empty_search_term() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        let result = find::execute(&vault, "", false);
        assert!(result.is_ok()); // Should handle empty search gracefully
    }

    #[test]
    fn test_find_with_regex_special_characters() {
        let (_temp_dir, vault) = create_test_vault_with_content();

        // Test search terms with regex special characters
        let special_chars = vec!["[", "]", "(", ")", "*", "+", "?", ".", "^", "$"];

        for special_char in special_chars {
            let result = find::execute(&vault, special_char, false);
            assert!(
                result.is_ok(),
                "Failed with special character: {}",
                special_char
            );
        }
    }
}
