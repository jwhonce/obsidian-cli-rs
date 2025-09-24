//! Simple utility function tests - CI safe, no user input
//! Basic tests for utility functions without complex pattern matching

use chrono::Local;
use obsidian_cli::{utils::*, Vault};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod simple_utils_tests {
    use super::*;

    fn create_test_vault(temp_dir: &TempDir) -> Vault {
        Vault {
            path: temp_dir.path().to_path_buf(),
            blacklist: vec![".obsidian".to_string().into()],
            editor: "echo".to_string().into(),
            ident_key: "uid".to_string().into(),
            journal_template: "# Daily".to_string().into(),
            verbose: false,
        }
    }

    #[test]
    fn test_resolve_page_path_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test file
        let test_file = vault_path.join("test-note.md");
        fs::write(&test_file, "# Test Note").unwrap();

        let result = resolve_page_path(std::path::Path::new("test-note.md"), vault_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_file);
    }

    #[test]
    fn test_resolve_page_path_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test file
        let test_file = vault_path.join("test-note.md");
        fs::write(&test_file, "# Test Note").unwrap();

        let result = resolve_page_path(std::path::Path::new("test-note"), vault_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_file);
    }

    #[test]
    fn test_resolve_page_path_nested() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create nested structure
        fs::create_dir_all(vault_path.join("folder/subfolder")).unwrap();
        let nested_file = vault_path.join("folder/subfolder/nested.md");
        fs::write(&nested_file, "# Nested Note").unwrap();

        let result = resolve_page_path(std::path::Path::new("folder/subfolder/nested"), vault_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), nested_file);
    }

    #[test]
    fn test_resolve_page_path_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        let result = resolve_page_path(std::path::Path::new("nonexistent-note"), vault_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_matching_files_exact_match() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test files
        fs::write(
            vault_path.join("exact-match.md"),
            "# Exact Match\nContent here",
        )
        .unwrap();
        fs::write(
            vault_path.join("other-file.md"),
            "# Other File\nDifferent content",
        )
        .unwrap();

        let results = find_matching_files(vault_path, "exact-match", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(!files.is_empty());
        assert!(files
            .iter()
            .any(|path| path.file_name().unwrap() == "exact-match.md"));
    }

    #[test]
    fn test_find_matching_files_partial_match() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test files
        fs::write(
            vault_path.join("partial-match-test.md"),
            "# Partial Match Test\nThis should match partial searches.",
        )
        .unwrap();

        let results = find_matching_files(vault_path, "partial", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_find_matching_files_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        fs::write(
            vault_path.join("CaSeTest.md"),
            "# Case Test\nMixed case content",
        )
        .unwrap();

        let results = find_matching_files(vault_path, "casetest", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_find_matching_files_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        let results = find_matching_files(vault_path, "anything", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_get_template_vars() {
        let date = Local::now();
        let vars = get_template_vars(date);

        assert!(vars.year >= 2020);
        assert!(vars.month >= 1 && vars.month <= 12);
        assert!(vars.day >= 1 && vars.day <= 31);
        assert!(!vars.month_name.is_empty());
        assert!(!vars.weekday.is_empty());
        assert!(!vars.month_abbr.is_empty());
        assert!(!vars.weekday_abbr.is_empty());
    }

    #[test]
    fn test_get_template_vars_specific_date() {
        use chrono::TimeZone;
        let date = Local.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let vars = get_template_vars(date);

        assert_eq!(vars.year, 2023);
        assert_eq!(vars.month, 6);
        assert_eq!(vars.day, 15);
        assert_eq!(vars.month_name, "June");
        assert_eq!(vars.weekday, "Thursday");
        assert_eq!(vars.month_abbr, "Jun");
        assert_eq!(vars.weekday_abbr, "Thu");
    }

    #[test]
    fn test_format_journal_template() {
        use chrono::TimeZone;
        let date = Local.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let vars = get_template_vars(date);

        let template = "# Daily Note";
        let result = format_journal_template(template, &vars);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("# Daily Note"));
    }

    #[test]
    fn test_get_vault_info() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create some test files
        fs::write(temp_dir.path().join("note1.md"), "# Note 1\nContent").unwrap();
        fs::write(temp_dir.path().join("note2.md"), "# Note 2\nMore content").unwrap();

        // Create nested structure
        fs::create_dir_all(temp_dir.path().join("subfolder")).unwrap();
        fs::write(
            temp_dir.path().join("subfolder/note3.md"),
            "# Note 3\nNested content",
        )
        .unwrap();

        let result = get_vault_info(&vault);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.total_files >= 3);
        assert!(info.markdown_files >= 3);
        assert!(info.vault_path == temp_dir.path());
        assert!(!info.blacklist.is_empty());
    }

    #[test]
    fn test_get_vault_info_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        let result = get_vault_info(&vault);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.total_files, 0);
        assert_eq!(info.markdown_files, 0);
    }

    #[test]
    fn test_launch_editor_success() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();

        // Use "true" command as a safe mock editor that always succeeds
        let result = launch_editor("true", &test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_launch_editor_command_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();

        // Use a command that doesn't exist
        let result = launch_editor("nonexistent_editor_xyz", &test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_launch_editor_command_fails() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();

        // Use "false" command which always exits with error
        let result = launch_editor("false", &test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_paths() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create files with unusual names
        let unusual_names = vec![
            "file with spaces.md",
            "file_with_underscores.md",
            "file-with-dashes.md",
        ];

        for name in &unusual_names {
            fs::write(vault_path.join(name), format!("Content of {}", name)).unwrap();
        }

        // Test vault info with unusual files
        let vault = create_test_vault(&temp_dir);
        let result = get_vault_info(&vault);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.total_files, unusual_names.len());

        // Test find functionality
        for name in &unusual_names {
            let results = find_matching_files(vault_path, &name[0..4], false); // Search first 4 chars
            assert!(results.is_ok());
            let files = results.unwrap();
            assert!(!files.is_empty(), "Should find file with name: {}", name);
        }
    }
}
