//! Utility function tests - CI safe, no user input
//! Tests path operations, file matching, and template utilities

use chrono::Local;
use obsidian_cli::{utils::*, Vault};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod utils_tests {
    use super::*;

    fn create_test_vault(temp_dir: &TempDir) -> Vault {
        Vault {
            path: temp_dir.path().to_path_buf(),
            blacklist: vec![".obsidian".to_string(), "*.tmp".to_string()],
            editor: "echo".to_string(),
            ident_key: "uid".to_string(),
            journal_template: "# Daily {year}-{month:02}-{day:02}".to_string(),
            verbose: false,
        }
    }

    #[test]
    fn test_is_path_blacklisted_basic() {
        let blacklist = vec![
            ".obsidian".to_string(),
            "*.tmp".to_string(),
            "node_modules".to_string(),
        ];

        assert!(is_path_blacklisted(
            Path::new(".obsidian/config.json"),
            &blacklist
        ));
        assert!(is_path_blacklisted(Path::new("test.tmp"), &blacklist));
        assert!(is_path_blacklisted(
            Path::new("project/node_modules/package.json"),
            &blacklist
        ));
        assert!(!is_path_blacklisted(
            Path::new("normal-file.md"),
            &blacklist
        ));
        assert!(!is_path_blacklisted(Path::new("important.txt"), &blacklist));
    }

    #[test]
    fn test_is_path_blacklisted_patterns() {
        let blacklist = vec![
            "*.log".to_string(),
            "temp*".to_string(),
            "*cache*".to_string(),
        ];

        assert!(is_path_blacklisted(Path::new("error.log"), &blacklist));
        assert!(is_path_blacklisted(Path::new("temp_file.txt"), &blacklist));
        assert!(is_path_blacklisted(
            Path::new("my_cache_file.dat"),
            &blacklist
        ));
        assert!(is_path_blacklisted(Path::new("cache.txt"), &blacklist));
        assert!(!is_path_blacklisted(Path::new("normal.md"), &blacklist));
    }

    #[test]
    fn test_resolve_page_path_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test file
        let test_file = vault_path.join("test-note.md");
        fs::write(&test_file, "# Test Note").unwrap();

        let result = resolve_page_path(Path::new("test-note.md"), vault_path);
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

        let result = resolve_page_path(Path::new("test-note"), vault_path);
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

        let result = resolve_page_path(Path::new("folder/subfolder/nested"), vault_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), nested_file);
    }

    #[test]
    fn test_resolve_page_path_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        let result = resolve_page_path(Path::new("nonexistent-note"), vault_path);
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
            r#"---
title: Partial Match Test
tags: [partial, test]
---

# Partial Match Test
This should match partial searches.
"#,
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

        let template = "# Daily Note {year}-{month:02}-{day:02}\n\n## {weekday}";
        let result = format_journal_template(template, &vars);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("# Daily Note 2023-06-15"));
        assert!(formatted.contains("## Thursday"));
    }

    #[test]
    fn test_format_journal_template_complex() {
        use chrono::TimeZone;
        let date = Local.with_ymd_and_hms(2023, 12, 25, 15, 45, 0).unwrap();
        let vars = get_template_vars(date);

        let template = r#"---
title: "Journal {year}-{month:02}-{day:02}"
date: {year}-{month:02}-{day:02}
weekday: {weekday}
type: journal
---

# Journal Entry - {month_name} {day}, {year}

## Weather
- Temperature: 
- Conditions: 

## Tasks
- [ ] 

## Notes


## Reflection on {weekday}

"#;

        let result = format_journal_template(template, &vars);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(formatted.contains("title: \"Journal 2023-12-25\""));
        assert!(formatted.contains("date: 2023-12-25"));
        assert!(formatted.contains("weekday: Monday"));
        assert!(formatted.contains("# Journal Entry - December 25, 2023"));
        assert!(formatted.contains("## Reflection on Monday"));
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
    fn test_get_vault_info_with_blacklisted_files() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault(&temp_dir);

        // Create normal files
        fs::write(temp_dir.path().join("note.md"), "# Note\nContent").unwrap();

        // Create blacklisted files
        fs::create_dir_all(temp_dir.path().join(".obsidian")).unwrap();
        fs::write(temp_dir.path().join(".obsidian/config.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("temp.tmp"), "Temporary file").unwrap();

        let result = get_vault_info(&vault);
        assert!(result.is_ok());

        let info = result.unwrap();
        // Should only count non-blacklisted files
        assert_eq!(info.total_files, 1);
        assert_eq!(info.markdown_files, 1);
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
    fn test_complex_blacklist_patterns() {
        let patterns = vec![
            "*.log".to_string(),
            "**/node_modules/**".to_string(),
            ".git".to_string(),
            "build/*".to_string(),
            "*.tmp".to_string(),
            "cache*".to_string(),
        ];

        // Should be blacklisted
        assert!(is_path_blacklisted(Path::new("error.log"), &patterns));
        assert!(is_path_blacklisted(
            Path::new("project/node_modules/package.json"),
            &patterns
        ));
        assert!(is_path_blacklisted(Path::new(".git/config"), &patterns));
        assert!(is_path_blacklisted(
            Path::new("build/output.exe"),
            &patterns
        ));
        assert!(is_path_blacklisted(Path::new("temp.tmp"), &patterns));
        assert!(is_path_blacklisted(Path::new("cache_file.dat"), &patterns));

        // Should NOT be blacklisted
        assert!(!is_path_blacklisted(Path::new("normal.md"), &patterns));
        assert!(!is_path_blacklisted(Path::new("src/main.rs"), &patterns));
        assert!(!is_path_blacklisted(Path::new("README.txt"), &patterns));
    }

    #[test]
    fn test_find_matching_files_with_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create file with frontmatter title
        fs::write(
            vault_path.join("title-test.md"),
            r#"---
title: "My Special Title"
tags: [special, test]
---

# Title Test
Content here.
"#,
        )
        .unwrap();

        let results = find_matching_files(vault_path, "Special Title", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_find_matching_files_no_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create file without frontmatter - use filename that matches search term
        fs::write(
            vault_path.join("frontmatter-test.md"),
            "# No Frontmatter\nJust content",
        )
        .unwrap();

        let results = find_matching_files(vault_path, "frontmatter", false);
        assert!(results.is_ok());
        let files = results.unwrap();
        assert!(!files.is_empty());
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
            "file.with.many.dots.md",
            "file(with)parens.md",
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

    #[test]
    fn test_wrap_filename_short() {
        use obsidian_cli::utils::wrap_filename;
        let result = wrap_filename("short.md", 40);
        assert_eq!(result, "short.md");
    }

    #[test]
    fn test_wrap_filename_exact_length() {
        use obsidian_cli::utils::wrap_filename;
        let filename = "a".repeat(40);
        let result = wrap_filename(&filename, 40);
        assert_eq!(result, filename);
    }

    #[test]
    fn test_wrap_filename_long_with_path() {
        use obsidian_cli::utils::wrap_filename;
        let filename = "very/long/directory/structure/with/deep/nested/files/example.md";
        let result = wrap_filename(filename, 40);
        let lines: Vec<&str> = result.split('\n').collect();
        
        // Should wrap into multiple lines
        assert!(lines.len() > 1);
        
        // Each line should be <= 40 characters (except for parts that can't be broken)
        for line in &lines {
            assert!(line.len() <= 40 || !line.contains('/'));
        }
    }

    #[test]
    fn test_wrap_filename_breaks_at_separator() {
        use obsidian_cli::utils::wrap_filename;
        let filename = "documents/projects/obsidian-cli/src/commands/ls.rs";
        let result = wrap_filename(filename, 25);
        let lines: Vec<&str> = result.split('\n').collect();
        
        // Should break at path separators when possible
        assert!(lines.len() > 1);
        
        // Verify it breaks intelligently
        for line in &lines {
            assert!(line.len() <= 25 || !line.contains('/'));
        }
    }

    #[test]
    fn test_wrap_filename_very_long_part() {
        use obsidian_cli::utils::wrap_filename;
        let filename = "normal/averyverylongfilenamethatcannotbebrokenatpathseparators.md";
        let result = wrap_filename(filename, 20);
        let lines: Vec<&str> = result.split('\n').collect();
        
        // Should still wrap even when individual parts are very long
        assert!(lines.len() > 1);
    }
}
