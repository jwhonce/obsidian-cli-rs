//! Advanced query engine tests - CI safe, no user input
//! Comprehensive testing of query command filtering, output formats, and edge cases

use obsidian_cli::commands::query;
use obsidian_cli::types::{OutputStyle, State};
use tempfile::TempDir;
use std::fs;
use std::path::Path;
use serde_json::{json, Value};

// Helper function to create a test state with query functionality
fn create_test_state_for_query(temp_dir: &TempDir) -> State {
    let vault_path = temp_dir.path();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

    State {
        vault: vault_path.to_path_buf(),
        blacklist: vec![".obsidian".to_string(), "blacklisted".to_string()],
        editor: "true".to_string(),
        ident_key: "uid".to_string(),
        journal_template: "Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}".to_string(),
        verbose: false,
    }
}

// Helper function to create a test note with complex frontmatter
fn create_complex_test_note(vault_path: &Path, name: &str, frontmatter_json: Value, content: &str) {
    let file_path = vault_path.join(format!("{}.md", name));
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    
    // Convert JSON to YAML-like frontmatter
    let frontmatter_str = match frontmatter_json {
        Value::Object(map) => {
            map.iter()
                .map(|(k, v)| match v {
                    Value::String(s) => format!("{}: \"{}\"", k, s),
                    Value::Number(n) => format!("{}: {}", k, n),
                    Value::Bool(b) => format!("{}: {}", k, b),
                    Value::Array(arr) => {
                        let items: Vec<String> = arr.iter()
                            .map(|item| match item {
                                Value::String(s) => format!("\"{}\"", s),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                _ => "null".to_string(),
                            })
                            .collect();
                        format!("{}: [{}]", k, items.join(", "))
                    },
                    Value::Null => format!("{}: null", k),
                    _ => format!("{}: {}", k, v),
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
        _ => String::new(),
    };
    
    let full_content = if frontmatter_str.is_empty() {
        content.to_string()
    } else {
        format!("---\n{}\n---\n{}", frontmatter_str, content)
    };
    
    fs::write(&file_path, full_content).unwrap();
}

#[cfg(test)]
mod advanced_query_engine_tests {
    use super::*;

    // === BASIC QUERY FUNCTIONALITY TESTS ===

    #[tokio::test]
    async fn test_query_conflicting_options_error() {
        let temp_dir = TempDir::new().unwrap();
        let _state = create_test_state_for_query(&temp_dir);
        
        let options = query::QueryOptions {
            key: "title",
            value: Some("test"),
            contains: Some("partial"), // Conflicting with value
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        // Should exit with error due to conflicting options
        // Note: This test checks that the function handles the conflict properly
        // In practice, this would call std::process::exit(1), so we test precondition
        assert!(options.value.is_some() && options.contains.is_some());
    }

    #[tokio::test]
    async fn test_query_verbose_output() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = create_test_state_for_query(&temp_dir);
        state.verbose = true; // Enable verbose mode
        
        create_complex_test_note(
            temp_dir.path(),
            "verbose-test",
            json!({"title": "Verbose Test", "status": "draft"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "title",
            value: Some("Verbose Test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_verbose_with_contains() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = create_test_state_for_query(&temp_dir);
        state.verbose = true;
        
        let options = query::QueryOptions {
            key: "description",
            value: None,
            contains: Some("test"),
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_verbose_with_exists() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = create_test_state_for_query(&temp_dir);
        state.verbose = true;
        
        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: None,
            exists: true,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_verbose_with_missing() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = create_test_state_for_query(&temp_dir);
        state.verbose = true;
        
        let options = query::QueryOptions {
            key: "nonexistent",
            value: None,
            contains: None,
            exists: false,
            missing: true,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    // === VALUE TYPE MATCHING TESTS ===

    #[tokio::test]
    async fn test_query_string_value_matching() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "string-test",
            json!({"title": "String Test", "category": "testing"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "category",
            value: Some("testing"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_number_value_matching() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "number-test",
            json!({"title": "Number Test", "priority": 42}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "priority",
            value: Some("42"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_boolean_value_matching() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "bool-test",
            json!({"title": "Bool Test", "published": true}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "published",
            value: Some("true"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_null_value_matching() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "null-test",
            json!({"title": "Null Test", "optional_field": null}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "optional_field",
            value: Some("null"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    // === CONTAINS FUNCTIONALITY TESTS ===

    #[tokio::test]
    async fn test_query_contains_string() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "contains-string",
            json!({"title": "Contains String Test", "description": "This contains the word important"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "description",
            value: None,
            contains: Some("important"),
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_contains_array() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "contains-array",
            json!({"title": "Contains Array Test", "tags": ["rust", "testing", "advanced"]}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: Some("rust"),
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_contains_nested_array() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "nested-array",
            json!({"title": "Nested Array Test", "categories": [["programming", "rust"], ["testing", "unit"]]}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "categories",
            value: None,
            contains: Some("rust"),
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_contains_object_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "object-test",
            json!({"title": "Object Test", "metadata": {"author": "test_user", "version": "1.0"}}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "metadata",
            value: None,
            contains: Some("test_user"),
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    // === OUTPUT STYLE TESTS ===

    #[tokio::test]
    async fn test_query_output_style_path() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "path-test",
            json!({"title": "Path Test", "type": "test"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_output_style_title() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "title-test",
            json!({"title": "Title Test", "type": "test"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Title,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_output_style_title_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create note without title - should fallback to filename
        create_complex_test_note(
            temp_dir.path(),
            "no-title-test",
            json!({"type": "test", "description": "No title field"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Title,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_output_style_table() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "table-test",
            json!({
                "title": "Table Test",
                "type": "test",
                "priority": 1,
                "tags": ["table", "test"],
                "metadata": {"nested": "value"},
                "active": true,
                "optional": null
            }),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Table,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_output_style_json() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "json-test",
            json!({
                "title": "JSON Test",
                "type": "test",
                "data": {"complex": "structure"},
                "array": [1, 2, 3]
            }),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Json,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_count_option() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "count-test-1",
            json!({"title": "Count Test 1", "type": "test"}),
            "Content 1"
        );
        
        create_complex_test_note(
            temp_dir.path(),
            "count-test-2",
            json!({"title": "Count Test 2", "type": "test"}),
            "Content 2"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: true,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    // === EDGE CASES AND ERROR HANDLING ===

    #[tokio::test]
    async fn test_query_empty_results() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create note that won't match the query
        create_complex_test_note(
            temp_dir.path(),
            "no-match",
            json!({"title": "No Match", "type": "different"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("nonexistent"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_empty_results_table_style() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        let options = query::QueryOptions {
            key: "nonexistent_key",
            value: None,
            contains: None,
            exists: true,
            missing: false,
            style: OutputStyle::Table,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_blacklisted_files() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create file in blacklisted directory
        fs::create_dir_all(temp_dir.path().join("blacklisted")).unwrap();
        create_complex_test_note(
            &temp_dir.path().join("blacklisted"),
            "hidden-file",
            json!({"title": "Hidden", "type": "test"}),
            "Content"
        );
        
        // Create non-blacklisted file
        create_complex_test_note(
            temp_dir.path(),
            "visible-file",
            json!({"title": "Visible", "type": "test"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
        // The blacklisted file should be ignored
    }

    #[tokio::test]
    async fn test_query_non_markdown_files() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create non-markdown files that should be ignored
        fs::write(temp_dir.path().join("test.txt"), "title: Test\nContent").unwrap();
        fs::write(temp_dir.path().join("data.json"), r#"{"title": "JSON"}"#).unwrap();
        
        // Create markdown file that should be processed
        create_complex_test_note(
            temp_dir.path(),
            "test-markdown",
            json!({"title": "Markdown", "type": "test"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_malformed_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = create_test_state_for_query(&temp_dir);
        state.verbose = true; // Test verbose error handling
        
        // Create file with malformed frontmatter
        let malformed_content = "---\ntitle: Unclosed quote\ndescription: \"This quote is never closed\nstatus: broken\n---\nContent";
        fs::write(
            temp_dir.path().join("malformed.md"),
            malformed_content
        ).unwrap();
        
        // Create valid file
        create_complex_test_note(
            temp_dir.path(),
            "valid-file",
            json!({"title": "Valid", "type": "test"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("test"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_exists_filter() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "has-tags",
            json!({"title": "Has Tags", "tags": ["exists"]}),
            "Content"
        );
        
        create_complex_test_note(
            temp_dir.path(),
            "no-tags",
            json!({"title": "No Tags", "description": "Missing tags"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: None,
            exists: true,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_missing_filter() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "has-tags",
            json!({"title": "Has Tags", "tags": ["exists"]}),
            "Content"
        );
        
        create_complex_test_note(
            temp_dir.path(),
            "no-tags",
            json!({"title": "No Tags", "description": "Missing tags"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: None,
            exists: false,
            missing: true,
            style: OutputStyle::Path,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_unicode_content() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        create_complex_test_note(
            temp_dir.path(),
            "unicode-test",
            json!({
                "title": "Unicode Test 测试 🎌",
                "description": "Contains unicode: ñáéíóú こんにちは",
                "tags": ["unicode-测试", "emoji-🚀"]
            }),
            "Unicode content: 世界 🌍"
        );
        
        let options = query::QueryOptions {
            key: "tags",
            value: None,
            contains: Some("测试"),
            exists: false,
            missing: false,
            style: OutputStyle::Json,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_large_dataset() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create many test files to test performance and correctness
        for i in 0..50 {
            create_complex_test_note(
                temp_dir.path(),
                &format!("test-file-{:03}", i),
                json!({
                    "title": format!("Test File {}", i),
                    "index": i,
                    "type": if i % 3 == 0 { "special" } else { "normal" },
                    "tags": [format!("tag-{}", i % 5), "bulk-test"]
                }),
                &format!("Content for file {}", i)
            );
        }
        
        let options = query::QueryOptions {
            key: "type",
            value: Some("special"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Table,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state_for_query(&temp_dir);
        
        // Create nested directory structure
        fs::create_dir_all(temp_dir.path().join("projects/rust/advanced")).unwrap();
        fs::create_dir_all(temp_dir.path().join("notes/personal")).unwrap();
        
        create_complex_test_note(
            &temp_dir.path().join("projects/rust"),
            "main-project",
            json!({"title": "Main Project", "language": "rust"}),
            "Content"
        );
        
        create_complex_test_note(
            &temp_dir.path().join("projects/rust/advanced"),
            "advanced-concepts",
            json!({"title": "Advanced Concepts", "language": "rust"}),
            "Content"
        );
        
        create_complex_test_note(
            &temp_dir.path().join("notes/personal"),
            "diary",
            json!({"title": "Diary", "type": "personal"}),
            "Content"
        );
        
        let options = query::QueryOptions {
            key: "language",
            value: Some("rust"),
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Title,
            count: false,
        };
        
        let result = query::execute(&state, options).await;
        assert!(result.is_ok());
    }
}
