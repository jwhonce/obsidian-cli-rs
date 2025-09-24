//! Comprehensive tests for builder patterns
//! Tests VaultBuilder, QueryOptionsBuilder, and TemplateVarsBuilder

use obsidian_cli::{
    commands::query::QueryOptions,
    types::{TemplateVars, Vault},
};
use tempfile::TempDir;

#[cfg(test)]
mod vault_builder_tests {
    use super::*;

    fn create_test_vault_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();
        std::fs::create_dir(vault_path.join(".obsidian")).unwrap();
        temp_dir
    }

    #[test]
    fn test_vault_builder_minimal() {
        let temp_dir = create_test_vault_dir();

        let vault = Vault::builder().path(temp_dir.path()).build().unwrap();

        assert_eq!(vault.path, temp_dir.path());
        assert_eq!(vault.editor.as_str(), "vi"); // Default editor
        assert_eq!(vault.ident_key.as_str(), "uid"); // Default ident_key
        assert!(!vault.verbose); // Default verbose
        assert!(vault.blacklist.is_empty()); // Default empty blacklist
    }

    #[test]
    fn test_vault_builder_full_configuration() {
        let temp_dir = create_test_vault_dir();

        let vault = Vault::builder()
            .path(temp_dir.path())
            .editor("nvim")
            .ident_key("custom_id")
            .journal_template("Notes/{year}/{month:02d}/{day:02d}")
            .blacklist_pattern("*.tmp")
            .blacklist_pattern(".git/")
            .verbose(true)
            .build()
            .unwrap();

        assert_eq!(vault.path, temp_dir.path());
        assert_eq!(vault.editor.as_str(), "nvim");
        assert_eq!(vault.ident_key.as_str(), "custom_id");
        assert_eq!(
            vault.journal_template.as_str(),
            "Notes/{year}/{month:02d}/{day:02d}"
        );
        assert!(vault.verbose);
        assert_eq!(vault.blacklist.len(), 2);
        assert_eq!(vault.blacklist[0].as_str(), "*.tmp");
        assert_eq!(vault.blacklist[1].as_str(), ".git/");
    }

    #[test]
    fn test_vault_builder_blacklist_patterns_multiple() {
        let temp_dir = create_test_vault_dir();
        let patterns = vec!["*.log", "temp/", "cache/*"];

        let vault = Vault::builder()
            .path(temp_dir.path())
            .blacklist_patterns(patterns.iter().map(|&s| s))
            .build()
            .unwrap();

        assert_eq!(vault.blacklist.len(), 3);
        assert_eq!(vault.blacklist[0].as_str(), "*.log");
        assert_eq!(vault.blacklist[1].as_str(), "temp/");
        assert_eq!(vault.blacklist[2].as_str(), "cache/*");
    }

    #[test]
    fn test_vault_builder_mixed_blacklist_methods() {
        let temp_dir = create_test_vault_dir();
        let patterns = vec!["*.log", "temp/"];

        let vault = Vault::builder()
            .path(temp_dir.path())
            .blacklist_patterns(patterns.iter().map(|&s| s))
            .blacklist_pattern("additional.txt")
            .build()
            .unwrap();

        assert_eq!(vault.blacklist.len(), 3);
        assert_eq!(vault.blacklist[0].as_str(), "*.log");
        assert_eq!(vault.blacklist[1].as_str(), "temp/");
        assert_eq!(vault.blacklist[2].as_str(), "additional.txt");
    }

    #[test]
    fn test_vault_builder_missing_path() {
        let result = Vault::builder().editor("vim").build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vault path is required");
    }

    #[test]
    fn test_vault_builder_default_constructor() {
        let _builder = Vault::builder();
        // Should not panic and should be in default state
        assert!(true);
    }

    #[test]
    fn test_vault_builder_pathbuf_conversion() {
        let temp_dir = create_test_vault_dir();
        let path_buf = temp_dir.path().to_path_buf();

        let vault = Vault::builder().path(path_buf.clone()).build().unwrap();

        assert_eq!(vault.path, path_buf);
    }

    #[test]
    fn test_vault_builder_string_conversions() {
        let temp_dir = create_test_vault_dir();

        let vault = Vault::builder()
            .path(temp_dir.path())
            .editor("custom_editor".to_string())
            .ident_key("custom_key".to_string())
            .journal_template("custom/{year}".to_string())
            .build()
            .unwrap();

        assert_eq!(vault.editor.as_str(), "custom_editor");
        assert_eq!(vault.ident_key.as_str(), "custom_key");
        assert_eq!(vault.journal_template.as_str(), "custom/{year}");
    }

    #[test]
    fn test_vault_builder_fluent_chaining() {
        let temp_dir = create_test_vault_dir();

        // Test that all methods return Self and can be chained
        let vault = Vault::builder()
            .path(temp_dir.path())
            .verbose(true)
            .verbose(false) // Should override previous
            .editor("vim")
            .editor("nvim") // Should override previous
            .build()
            .unwrap();

        assert!(!vault.verbose); // Last value wins
        assert_eq!(vault.editor.as_str(), "nvim"); // Last value wins
    }
}

#[cfg(test)]
mod query_options_builder_tests {
    use super::*;
    use obsidian_cli::types::OutputStyle;

    #[test]
    fn test_query_options_builder_minimal() {
        let options = QueryOptions::builder().key("test_key").build().unwrap();

        assert_eq!(options.key, "test_key");
        assert!(options.value.is_none());
        assert!(options.contains.is_none());
        assert!(!options.exists);
        assert!(!options.missing);
        assert!(matches!(options.style, OutputStyle::Path));
        assert!(!options.count);
    }

    #[test]
    fn test_query_options_builder_full_configuration() {
        let options = QueryOptions::builder()
            .key("status")
            .value("published")
            .exists(true)
            .style(OutputStyle::Json)
            .count(true)
            .build()
            .unwrap();

        assert_eq!(options.key, "status");
        assert_eq!(options.value, Some("published"));
        assert!(options.exists);
        assert!(matches!(options.style, OutputStyle::Json));
        assert!(options.count);
    }

    #[test]
    fn test_query_options_builder_contains_mode() {
        let options = QueryOptions::builder()
            .key("tags")
            .contains("rust")
            .missing(true)
            .build()
            .unwrap();

        assert_eq!(options.key, "tags");
        assert_eq!(options.contains, Some("rust"));
        assert!(options.missing);
        assert!(options.value.is_none());
    }

    #[test]
    fn test_query_options_builder_all_output_styles() {
        let styles = [
            OutputStyle::Path,
            OutputStyle::Title,
            OutputStyle::Table,
            OutputStyle::Json,
        ];

        for style in styles {
            let options = QueryOptions::builder()
                .key("test")
                .style(style)
                .build()
                .unwrap();

            // Just verify it builds successfully with each style
            assert_eq!(options.key, "test");
        }
    }

    #[test]
    fn test_query_options_builder_missing_key() {
        let result = QueryOptions::builder().value("test_value").build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Key is required for query");
    }

    #[test]
    fn test_query_options_builder_conflicting_value_and_contains() {
        let result = QueryOptions::builder()
            .key("test_key")
            .value("exact_value")
            .contains("substring")
            .build();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot specify both value and contains options"
        );
    }

    #[test]
    fn test_query_options_builder_default_constructor() {
        let _builder = QueryOptions::builder();
        // Should not panic and should be in default state
        assert!(true);
    }

    #[test]
    fn test_query_options_builder_fluent_chaining() {
        let options = QueryOptions::builder()
            .key("test")
            .exists(true)
            .exists(false) // Should override
            .count(false)
            .count(true) // Should override
            .build()
            .unwrap();

        assert!(!options.exists); // Last value wins
        assert!(options.count); // Last value wins
    }

    #[test]
    fn test_query_options_builder_boolean_combinations() {
        // Test various boolean flag combinations
        let combinations = [
            (true, false, true, false), // exists, missing, count, verbose
            (false, true, false, true),
            (true, true, false, false),
            (false, false, true, true),
        ];

        for (exists, missing, count, _verbose) in combinations {
            let options = QueryOptions::builder()
                .key("test")
                .exists(exists)
                .missing(missing)
                .count(count)
                .build()
                .unwrap();

            assert_eq!(options.exists, exists);
            assert_eq!(options.missing, missing);
            assert_eq!(options.count, count);
        }
    }
}

#[cfg(test)]
mod template_vars_builder_tests {
    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn test_template_vars_builder_manual_construction() {
        let vars = TemplateVars::builder()
            .year(2025)
            .month(1)
            .day(15)
            .month_name("January")
            .month_abbr("Jan")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build()
            .unwrap();

        assert_eq!(vars.year, 2025);
        assert_eq!(vars.month, 1);
        assert_eq!(vars.day, 15);
        assert_eq!(vars.month_name, "January");
        assert_eq!(vars.month_abbr, "Jan");
        assert_eq!(vars.weekday, "Wednesday");
        assert_eq!(vars.weekday_abbr, "Wed");
    }

    #[test]
    fn test_template_vars_builder_from_datetime() {
        let dt = Local.with_ymd_and_hms(2025, 6, 15, 12, 30, 45).unwrap();

        let vars = TemplateVars::builder()
            .from_chrono_datetime(&dt)
            .build()
            .unwrap();

        assert_eq!(vars.year, 2025);
        assert_eq!(vars.month, 6);
        assert_eq!(vars.day, 15);
        assert_eq!(vars.month_name, "June");
        assert_eq!(vars.month_abbr, "Jun");
        assert!(vars.weekday.len() > 0); // Weekday name exists
        assert!(vars.weekday_abbr.len() > 0); // Weekday abbr exists
    }

    #[test]
    fn test_template_vars_builder_missing_year() {
        let result = TemplateVars::builder()
            .month(1)
            .day(15)
            .month_name("January")
            .month_abbr("Jan")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Year is required");
    }

    #[test]
    fn test_template_vars_builder_missing_month() {
        let result = TemplateVars::builder()
            .year(2025)
            .day(15)
            .month_name("January")
            .month_abbr("Jan")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Month is required");
    }

    #[test]
    fn test_template_vars_builder_missing_day() {
        let result = TemplateVars::builder()
            .year(2025)
            .month(1)
            .month_name("January")
            .month_abbr("Jan")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Day is required");
    }

    #[test]
    fn test_template_vars_builder_missing_month_name() {
        let result = TemplateVars::builder()
            .year(2025)
            .month(1)
            .day(15)
            .month_abbr("Jan")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Month name is required");
    }

    #[test]
    fn test_template_vars_builder_missing_month_abbr() {
        let result = TemplateVars::builder()
            .year(2025)
            .month(1)
            .day(15)
            .month_name("January")
            .weekday("Wednesday")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Month abbreviation is required");
    }

    #[test]
    fn test_template_vars_builder_missing_weekday() {
        let result = TemplateVars::builder()
            .year(2025)
            .month(1)
            .day(15)
            .month_name("January")
            .month_abbr("Jan")
            .weekday_abbr("Wed")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Weekday is required");
    }

    #[test]
    fn test_template_vars_builder_missing_weekday_abbr() {
        let result = TemplateVars::builder()
            .year(2025)
            .month(1)
            .day(15)
            .month_name("January")
            .month_abbr("Jan")
            .weekday("Wednesday")
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Weekday abbreviation is required");
    }

    #[test]
    fn test_template_vars_builder_default_constructor() {
        let _builder = TemplateVars::builder();
        // Should not panic and should be in default state
        assert!(true);
    }

    #[test]
    fn test_template_vars_builder_string_conversions() {
        let vars = TemplateVars::builder()
            .year(2025)
            .month(12)
            .day(31)
            .month_name("December".to_string())
            .month_abbr("Dec".to_string())
            .weekday("Tuesday".to_string())
            .weekday_abbr("Tue".to_string())
            .build()
            .unwrap();

        assert_eq!(vars.month_name, "December");
        assert_eq!(vars.month_abbr, "Dec");
        assert_eq!(vars.weekday, "Tuesday");
        assert_eq!(vars.weekday_abbr, "Tue");
    }

    #[test]
    fn test_template_vars_builder_fluent_chaining() {
        let vars = TemplateVars::builder()
            .year(2025)
            .year(2024) // Should override
            .month(1)
            .month(12) // Should override
            .day(1)
            .day(31) // Should override
            .month_name("January")
            .month_name("December") // Should override
            .month_abbr("Jan")
            .month_abbr("Dec") // Should override
            .weekday("Monday")
            .weekday("Tuesday") // Should override
            .weekday_abbr("Mon")
            .weekday_abbr("Tue") // Should override
            .build()
            .unwrap();

        // Last values should win
        assert_eq!(vars.year, 2024);
        assert_eq!(vars.month, 12);
        assert_eq!(vars.day, 31);
        assert_eq!(vars.month_name, "December");
        assert_eq!(vars.month_abbr, "Dec");
        assert_eq!(vars.weekday, "Tuesday");
        assert_eq!(vars.weekday_abbr, "Tue");
    }

    #[test]
    fn test_template_vars_builder_mixed_construction() {
        // Start with chrono, then override some values manually
        let dt = Local.with_ymd_and_hms(2025, 3, 10, 14, 30, 0).unwrap();

        let vars = TemplateVars::builder()
            .from_chrono_datetime(&dt)
            .year(2024) // Override year
            .month_name("CustomMonth") // Override month name
            .build()
            .unwrap();

        assert_eq!(vars.year, 2024); // Overridden
        assert_eq!(vars.month, 3); // From chrono
        assert_eq!(vars.day, 10); // From chrono
        assert_eq!(vars.month_name, "CustomMonth"); // Overridden
        assert_eq!(vars.month_abbr, "Mar"); // From chrono
    }

    #[test]
    fn test_template_vars_builder_edge_case_dates() {
        // Test with various edge case dates via chrono
        let edge_dates = [
            (2025, 1, 1),   // New Year
            (2025, 12, 31), // New Year's Eve
            (2024, 2, 29),  // Leap year
            (2025, 2, 28),  // Non-leap year Feb
        ];

        for (year, month, day) in edge_dates {
            let dt = Local.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap();

            let vars = TemplateVars::builder()
                .from_chrono_datetime(&dt)
                .build()
                .unwrap();

            assert_eq!(vars.year, year);
            assert_eq!(vars.month, month);
            assert_eq!(vars.day, day);
            assert!(!vars.month_name.is_empty());
            assert!(!vars.month_abbr.is_empty());
            assert!(!vars.weekday.is_empty());
            assert!(!vars.weekday_abbr.is_empty());
        }
    }
}

#[cfg(test)]
mod builder_integration_tests {
    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn test_vault_builder_with_query_options() {
        // Test integration between VaultBuilder and QueryOptions
        let temp_dir = create_test_vault_dir();

        let vault = Vault::builder()
            .path(temp_dir.path())
            .verbose(true)
            .build()
            .unwrap();

        let options = QueryOptions::builder()
            .key("status")
            .value("published")
            .build()
            .unwrap();

        // Both should work together (this is mainly a compile test)
        assert_eq!(vault.verbose, true);
        assert_eq!(options.key, "status");
    }

    #[test]
    fn test_template_vars_with_vault_builder() {
        let temp_dir = create_test_vault_dir();
        let dt = Local::now();

        let vault = Vault::builder()
            .path(temp_dir.path())
            .journal_template("Journal/{year}/{month:02}")
            .build()
            .unwrap();

        let vars = TemplateVars::builder()
            .from_chrono_datetime(&dt)
            .build()
            .unwrap();

        // Both should be constructed successfully
        assert!(vault.journal_template.as_str().contains("{year}"));
        assert!(vars.year > 2020); // Reasonable sanity check
    }

    #[test]
    fn test_all_builders_default_then_customize() {
        let temp_dir = create_test_vault_dir();
        let dt = Local.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();

        // Create default instances then customize
        let vault = Vault::builder().path(temp_dir.path()).build().unwrap();

        let options = QueryOptions::builder().key("test").build().unwrap();

        let vars = TemplateVars::builder()
            .from_chrono_datetime(&dt)
            .build()
            .unwrap();

        // Verify defaults
        assert_eq!(vault.editor.as_str(), "vi");
        assert!(!vault.verbose);
        assert!(vault.blacklist.is_empty());

        assert!(options.value.is_none());
        assert!(!options.exists);

        assert_eq!(vars.year, 2025);
        assert_eq!(vars.month, 6);
    }

    fn create_test_vault_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();
        std::fs::create_dir(vault_path.join(".obsidian")).unwrap();
        temp_dir
    }
}
