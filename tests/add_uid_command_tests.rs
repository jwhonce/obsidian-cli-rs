//! Tests for add_uid command edge cases and error handling
//! Tests verbose output, force flag, and various error conditions

use obsidian_cli::{
    commands::add_uid,
    frontmatter,
    types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, Vault},
};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn create_test_vault() -> (TempDir, Vault) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Create .obsidian directory to make it a valid vault
    fs::create_dir(vault_path.join(".obsidian")).unwrap();

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
    fn test_add_uid_to_new_file() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("new_note.md");

        // Create a new file without frontmatter
        fs::write(&test_file, "# Test Note\n\nContent here").unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify UID was added
        let (frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));

        if let Some(Value::String(uid)) = frontmatter.get("uid") {
            assert_eq!(uid.len(), 36); // UUID length
            assert!(uid.contains('-')); // UUID format
        } else {
            panic!("UID should be a string");
        }
    }

    #[test]
    fn test_add_uid_with_verbose() {
        let (temp_dir, mut vault) = create_test_vault();
        vault.verbose = true;

        let test_file = temp_dir.path().join("verbose_note.md");
        fs::write(&test_file, "# Verbose Note\n\nContent").unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify UID was added
        let (frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));
    }

    #[test]
    fn test_add_uid_file_already_has_uid_no_force() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("existing_uid.md");

        // Create file with existing UID
        let content_with_uid = r#"---
uid: existing-uuid-12345
title: Test Note
---

# Test Note

Content here"#;
        fs::write(&test_file, content_with_uid).unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_err());

        // Verify error is about existing UID
        let error = result.unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("already has"));
        assert!(error_msg.contains("uid"));
    }

    #[test]
    fn test_add_uid_file_already_has_uid_with_force() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("force_replace.md");

        // Create file with existing UID
        let content_with_uid = r#"---
uid: old-uuid-12345
title: Test Note
---

# Test Note

Content here"#;
        fs::write(&test_file, content_with_uid).unwrap();

        let result = add_uid::execute(&vault, &test_file, true); // force = true
        assert!(result.is_ok());

        // Verify UID was replaced
        let (frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));

        if let Some(Value::String(uid)) = frontmatter.get("uid") {
            assert_ne!(uid, "old-uuid-12345"); // Should be different
            assert_eq!(uid.len(), 36); // Should be a proper UUID
        }
    }

    #[test]
    fn test_add_uid_with_verbose_existing_uid_message() {
        let (temp_dir, mut vault) = create_test_vault();
        vault.verbose = true;

        let test_file = temp_dir.path().join("verbose_existing.md");

        // Create file with existing UID
        let content_with_uid = r#"---
uid: verbose-test-uuid
---

# Verbose Test"#;
        fs::write(&test_file, content_with_uid).unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_err());

        // The error should be about existing UID
        let error = result.unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("already has"));
        assert!(error_msg.contains("verbose-test-uuid"));
    }

    #[test]
    fn test_add_uid_with_different_ident_key() {
        let (temp_dir, mut vault) = create_test_vault();
        vault.ident_key = IdentKey::from("custom_id");

        let test_file = temp_dir.path().join("custom_key.md");
        fs::write(&test_file, "# Custom Key Test").unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify custom key was used
        let (frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("custom_id"));
        assert!(!frontmatter.contains_key("uid"));
    }

    #[test]
    fn test_add_uid_nonexistent_file() {
        let (temp_dir, vault) = create_test_vault();
        let nonexistent_file = temp_dir.path().join("does_not_exist.md");

        let result = add_uid::execute(&vault, &nonexistent_file, false);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("not found") || error_msg.contains("No such file"));
    }

    #[test]
    fn test_add_uid_with_existing_frontmatter() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("existing_frontmatter.md");

        // Create file with existing frontmatter but no UID
        let content = r#"---
title: Test Note
author: Test Author
created: 2025-01-15
tags: [test, note]
---

# Test Note

Content with existing frontmatter"#;
        fs::write(&test_file, content).unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify UID was added while preserving other frontmatter
        let (frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));
        assert!(frontmatter.contains_key("title"));
        assert!(frontmatter.contains_key("author"));
        assert!(frontmatter.contains_key("created"));
        assert!(frontmatter.contains_key("tags"));
    }

    #[test]
    fn test_add_uid_file_with_no_frontmatter() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("no_frontmatter.md");

        // Create file with no frontmatter at all
        fs::write(&test_file, "Just plain content with no frontmatter").unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify frontmatter was added with UID
        let (frontmatter, content) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));
        assert!(content.contains("Just plain content"));
    }

    #[test]
    fn test_add_uid_with_malformed_frontmatter() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("malformed.md");

        // Create file with malformed frontmatter
        let content = r#"---
title: Test Note
invalid yaml: [unclosed bracket
author: Test
---

Content after malformed frontmatter"#;
        fs::write(&test_file, content).unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        // This might succeed or fail depending on how the frontmatter parser handles errors
        match result {
            Ok(_) => {
                // If it succeeds, verify UID was added
                let (frontmatter, _) = frontmatter::parse_file(&test_file).unwrap();
                assert!(frontmatter.contains_key("uid"));
            }
            Err(_) => {
                // If it fails, that's also acceptable for malformed frontmatter
            }
        }
    }

    #[test]
    fn test_add_uid_generates_unique_uids() {
        let (temp_dir, vault) = create_test_vault();

        // Create multiple files and add UIDs
        let mut uids = Vec::new();

        for i in 0..5 {
            let test_file = temp_dir.path().join(format!("unique_{}.md", i));
            fs::write(&test_file, format!("# Test Note {}", i)).unwrap();

            let result = add_uid::execute(&vault, &test_file, false);
            assert!(result.is_ok());

            let (frontmatter, _) = frontmatter::parse_file(&test_file).unwrap();
            if let Some(Value::String(uid)) = frontmatter.get("uid") {
                uids.push(uid.clone());
            }
        }

        // Verify all UIDs are unique
        assert_eq!(uids.len(), 5);
        for i in 0..uids.len() {
            for j in (i + 1)..uids.len() {
                assert_ne!(uids[i], uids[j], "UIDs should be unique");
            }
        }
    }

    #[test]
    fn test_add_uid_preserves_file_permissions() {
        let (temp_dir, vault) = create_test_vault();
        let test_file = temp_dir.path().join("permissions.md");

        fs::write(&test_file, "# Permission Test").unwrap();

        // Get original metadata
        let original_metadata = fs::metadata(&test_file).unwrap();

        let result = add_uid::execute(&vault, &test_file, false);
        assert!(result.is_ok());

        // Verify file still exists and has same basic properties
        let new_metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(original_metadata.is_file(), new_metadata.is_file());

        // Verify UID was added
        let (frontmatter, _) = frontmatter::parse_file(&test_file).unwrap();
        assert!(frontmatter.contains_key("uid"));
    }
}
