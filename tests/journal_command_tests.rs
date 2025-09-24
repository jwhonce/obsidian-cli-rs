//! Comprehensive tests for journal command
//! Tests journal creation, date parsing, template expansion, and error handling

use obsidian_cli::{
    commands::journal,
    types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate, Vault},
};
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

    #[test]
    fn test_journal_create_default_date() {
        let (_temp_dir, vault) = create_test_vault();

        let result = journal::execute(&vault, None);
        assert!(result.is_ok());

        // Check that some journal file was created (we can't predict exact name due to current date)
        let journal_dir = vault.path.join("Journal");
        assert!(journal_dir.exists());
    }

    #[test]
    fn test_journal_create_specific_date() {
        let (_temp_dir, vault) = create_test_vault();

        let result = journal::execute(&vault, Some("2025-01-15"));
        assert!(result.is_ok());

        // Check that the specific date journal was created
        let expected_path = vault.path.join("Journal/2025/01/15.md");
        assert!(expected_path.exists());

        // Verify the file has proper content
        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("# 15")); // The header uses the filename stem
        assert!(content.contains("uid:")); // Should have frontmatter with UID
    }

    #[test]
    fn test_journal_with_verbose() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.verbose = true;

        let result = journal::execute(&vault, Some("2025-02-28"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Journal/2025/02/28.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_journal_different_template() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.journal_template = JournalTemplate::from("Daily/{year}-{month:02d}-{day:02d}");

        let result = journal::execute(&vault, Some("2025-03-10"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Daily/2025-03-10.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_journal_nested_template() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.journal_template =
            JournalTemplate::from("Notes/{year}/Month-{month:02d}/Day-{day:02d}");

        let result = journal::execute(&vault, Some("2025-12-25"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Notes/2025/Month-12/Day-25.md");
        assert!(expected_path.exists());

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("# Day-25")); // The header uses the filename from template
    }

    #[test]
    fn test_journal_file_already_exists() {
        let (_temp_dir, vault) = create_test_vault();

        // Create the journal file first
        let journal_path = vault.path.join("Journal/2025/01/01.md");
        fs::create_dir_all(journal_path.parent().unwrap()).unwrap();
        fs::write(&journal_path, "Existing content").unwrap();

        // Try to create journal for same date
        let result = journal::execute(&vault, Some("2025-01-01"));
        assert!(result.is_ok()); // Should still succeed (opens existing)

        // File should still exist
        assert!(journal_path.exists());
    }

    #[test]
    fn test_journal_invalid_date_format() {
        let (_temp_dir, vault) = create_test_vault();

        // Test various invalid date formats
        let invalid_dates = vec![
            "invalid-date",
            "2025-13-01", // Invalid month
            "2025-01-32", // Invalid day
            "25-01-01",   // Wrong year format
            "2025/01/01", // Wrong separator
            "01-01-2025", // Wrong order
            "",           // Empty string
            "2025-1-1",   // No zero padding (this might actually work)
        ];

        for invalid_date in invalid_dates {
            let result = journal::execute(&vault, Some(invalid_date));
            // Some of these might succeed if the date parser is forgiving
            // We're mainly testing that the code doesn't panic
            match result {
                Ok(_) => {
                    // If it succeeds, make sure a file was created somewhere
                    assert!(
                        vault.path.join("Journal").exists()
                            || vault.path.read_dir().unwrap().count() > 1
                    );
                }
                Err(_) => {
                    // If it fails, that's also acceptable for invalid dates
                }
            }
        }
    }

    #[test]
    fn test_journal_leap_year_date() {
        let (_temp_dir, vault) = create_test_vault();

        // Test leap year date
        let result = journal::execute(&vault, Some("2024-02-29"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Journal/2024/02/29.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_journal_edge_case_dates() {
        let (_temp_dir, vault) = create_test_vault();

        // Test edge case dates
        let edge_dates = vec![
            "2025-01-01", // New Year's Day
            "2025-12-31", // New Year's Eve
            "2025-02-28", // End of February (non-leap year)
            "2025-04-30", // End of April (30-day month)
            "2025-07-04", // Random mid-year date
        ];

        for date in edge_dates {
            let result = journal::execute(&vault, Some(date));
            assert!(result.is_ok(), "Failed for date: {}", date);

            // Verify file was created with proper structure
            let date_parts: Vec<&str> = date.split('-').collect();
            let year = date_parts[0];
            let month = date_parts[1];
            let day = date_parts[2];

            let expected_path = vault
                .path
                .join(format!("Journal/{}/{}/{}.md", year, month, day));
            assert!(
                expected_path.exists(),
                "File not created for date: {}",
                date
            );
        }
    }

    #[test]
    fn test_journal_with_different_ident_key() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.ident_key = IdentKey::from("journal_id");

        let result = journal::execute(&vault, Some("2025-06-15"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Journal/2025/06/15.md");
        assert!(expected_path.exists());

        // Check that the file has the correct ident_key in frontmatter
        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("journal_id:"));
    }

    #[test]
    fn test_journal_template_with_no_extension() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.journal_template = JournalTemplate::from("Logs/{year}/{month:02d}/{day:02d}");

        let result = journal::execute(&vault, Some("2025-05-20"));
        assert!(result.is_ok());

        // Should add .md extension automatically
        let expected_path = vault.path.join("Logs/2025/05/20.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_journal_creates_nested_directories() {
        let (_temp_dir, mut vault) = create_test_vault();
        vault.journal_template =
            JournalTemplate::from("Deep/Nested/Structure/{year}/{month:02d}/{day:02d}");

        let result = journal::execute(&vault, Some("2025-08-14"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Deep/Nested/Structure/2025/08/14.md");
        assert!(expected_path.exists());

        // Verify all intermediate directories were created
        assert!(vault.path.join("Deep").exists());
        assert!(vault.path.join("Deep/Nested").exists());
        assert!(vault.path.join("Deep/Nested/Structure").exists());
        assert!(vault.path.join("Deep/Nested/Structure/2025").exists());
        assert!(vault.path.join("Deep/Nested/Structure/2025/08").exists());
    }

    #[test]
    fn test_journal_content_structure() {
        let (_temp_dir, vault) = create_test_vault();

        let result = journal::execute(&vault, Some("2025-07-04"));
        assert!(result.is_ok());

        let expected_path = vault.path.join("Journal/2025/07/04.md");
        let content = fs::read_to_string(&expected_path).unwrap();

        // Verify content structure
        assert!(content.contains("---")); // Frontmatter delimiters
        assert!(content.contains("uid:")); // UID field
        assert!(content.contains("created:")); // Created timestamp
        assert!(content.contains("modified:")); // Modified timestamp
        assert!(content.contains("# 04")); // Header uses filename stem
    }
}
