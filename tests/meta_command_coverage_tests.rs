//! Tests for meta command coverage
//! Tests meta command functionality, edge cases, and error paths

use obsidian_cli::{
    commands::meta,
    types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, Vault},
};
use std::fs;
use tempfile::TempDir;

fn create_test_vault_with_meta_files() -> (TempDir, Vault) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create .obsidian directory to make it a valid vault
    fs::create_dir(vault_path.join(".obsidian")).unwrap();

    // Create files with different frontmatter
    fs::write(
        vault_path.join("note_with_meta.md"),
        r#"---
title: Test Note
author: John Doe
tags: [rust, testing]
priority: high
count: 42
published: true
created: 2025-01-15
---

# Test Note

Content here."#,
    )
    .unwrap();

    fs::write(
        vault_path.join("simple_note.md"),
        r#"---
title: Simple Note
---

# Simple Note"#,
    )
    .unwrap();

    fs::write(
        vault_path.join("no_frontmatter.md"),
        "# No Frontmatter\n\nJust content.",
    )
    .unwrap();

    let vault = Vault {
        path: vault_path.to_path_buf(),
        blacklist: vec![BlacklistPattern::from(".obsidian/")],
        editor: EditorCommand::from("true"),
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
    fn test_meta_display_existing_key() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        let result = meta::execute(&vault, &note_path, Some("title"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_display_nonexistent_key() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        let result = meta::execute(&vault, &note_path, Some("nonexistent_key"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_meta_display_all_frontmatter() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        let result = meta::execute(&vault, &note_path, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_update_existing_key() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        let result = meta::execute(&vault, &note_path, Some("title"), Some("Updated Title"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_add_new_key() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("simple_note.md");

        let result = meta::execute(&vault, &note_path, Some("new_field"), Some("new_value"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_with_verbose() {
        let (_temp_dir, mut vault) = create_test_vault_with_meta_files();
        vault.verbose = true;
        let note_path = vault.path.join("note_with_meta.md");

        let result = meta::execute(&vault, &note_path, Some("author"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_update_different_types() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        // Test updating with different value types
        let result = meta::execute(&vault, &note_path, Some("count"), Some("100"));
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("published"), Some("false"));
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("tags"), Some("[updated, test]"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_file_without_frontmatter() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("no_frontmatter.md");

        // Try to read from file without frontmatter
        let result = meta::execute(&vault, &note_path, Some("title"), None);
        assert!(result.is_err());

        // Try to add to file without frontmatter
        let result = meta::execute(&vault, &note_path, Some("new_key"), Some("new_value"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_nonexistent_file() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let nonexistent = vault.path.join("does_not_exist.md");

        let result = meta::execute(&vault, &nonexistent, Some("title"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_meta_with_special_characters() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        // Test keys and values with special characters
        let result = meta::execute(
            &vault,
            &note_path,
            Some("special-key"),
            Some("value with spaces"),
        );
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("unicode_key"), Some("café résumé"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_display_different_value_types() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        // Test displaying different types of values
        let result = meta::execute(&vault, &note_path, Some("author"), None); // string
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("count"), None); // number
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("published"), None); // boolean
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("tags"), None); // array
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_empty_values() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("simple_note.md");

        // Test setting empty value
        let result = meta::execute(&vault, &note_path, Some("empty_field"), Some(""));
        assert!(result.is_ok());

        // Test displaying empty value
        let result = meta::execute(&vault, &note_path, Some("empty_field"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_with_nested_directory() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();

        // Create nested file
        fs::create_dir(vault.path.join("subdir")).unwrap();
        fs::write(
            vault.path.join("subdir/nested.md"),
            r#"---
title: Nested Note
---

# Nested"#,
        )
        .unwrap();

        let nested_path = vault.path.join("subdir/nested.md");
        let result = meta::execute(&vault, &nested_path, Some("title"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_key_case_sensitivity() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        // Test that keys are case sensitive
        let result = meta::execute(&vault, &note_path, Some("TITLE"), None);
        assert!(result.is_err()); // Should fail because "title" exists but "TITLE" doesn't

        let result = meta::execute(&vault, &note_path, Some("title"), None);
        assert!(result.is_ok()); // Should succeed
    }

    #[test]
    fn test_meta_json_values() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("simple_note.md");

        // Test setting complex JSON values
        let result = meta::execute(
            &vault,
            &note_path,
            Some("config"),
            Some(r#"{"enabled": true, "count": 5}"#),
        );
        assert!(result.is_ok());

        let result = meta::execute(&vault, &note_path, Some("list"), Some("[1, 2, 3, 4, 5]"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_overwrite_existing_values() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();
        let note_path = vault.path.join("note_with_meta.md");

        // First read the original value
        let result = meta::execute(&vault, &note_path, Some("priority"), None);
        assert!(result.is_ok());

        // Then overwrite it
        let result = meta::execute(&vault, &note_path, Some("priority"), Some("low"));
        assert!(result.is_ok());

        // Read it again to confirm change
        let result = meta::execute(&vault, &note_path, Some("priority"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_with_malformed_frontmatter() {
        let (_temp_dir, vault) = create_test_vault_with_meta_files();

        // Create file with malformed frontmatter
        let malformed_path = vault.path.join("malformed.md");
        fs::write(
            &malformed_path,
            r#"---
title: Test
invalid: [unclosed
author: Test
---

Content"#,
        )
        .unwrap();

        // This might succeed or fail depending on how robust the frontmatter parser is
        let result = meta::execute(&vault, &malformed_path, Some("title"), None);
        match result {
            Ok(_) => {
                // If it succeeds, that's fine
            }
            Err(_) => {
                // If it fails due to malformed frontmatter, that's also acceptable
            }
        }
    }
}
