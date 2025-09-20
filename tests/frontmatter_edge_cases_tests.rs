//! Comprehensive frontmatter edge cases tests - CI safe, no user input
//! Tests all edge cases, error conditions, and complex scenarios for frontmatter handling

use obsidian_cli::frontmatter;
use tempfile::TempDir;
use std::fs;
use serde_json::{json, Value};
use std::collections::HashMap;

#[cfg(test)]
mod frontmatter_edge_cases_tests {
    use super::*;

    #[test]
    fn test_parse_file_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("does_not_exist.md");
        
        let result = frontmatter::parse_file(&nonexistent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.md");
        fs::write(&empty_file, "").unwrap();
        
        let result = frontmatter::parse_file(&empty_file);
        assert!(result.is_ok());
        let (frontmatter, content) = result.unwrap();
        assert!(frontmatter.is_empty());
        assert_eq!(content, "");
    }

    #[test]
    fn test_parse_file_binary_content() {
        let temp_dir = TempDir::new().unwrap();
        let binary_file = temp_dir.path().join("binary.md");
        fs::write(&binary_file, b"\x00\x01\x02\x03\xFF\xFE").unwrap();
        
        // Should handle binary content gracefully (may or may not succeed based on UTF-8 validity)
        let result = frontmatter::parse_file(&binary_file);
        // Don't assert success/failure since binary content handling is implementation dependent
        let _ = result;
    }

    #[test]
    fn test_parse_string_incomplete_frontmatter() {
        // Test incomplete frontmatter (starts with --- but no closing ---)
        let content = "---\ntitle: Incomplete\nstatus: draft\n\nThis has incomplete frontmatter";
        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        // Should treat as no frontmatter due to incomplete format
        assert!(frontmatter.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_string_empty_frontmatter() {
        let content = "---\n---\nContent without any frontmatter fields";
        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        assert!(frontmatter.is_empty());
        assert_eq!(body, "Content without any frontmatter fields");
    }

    #[test]
    fn test_parse_string_malformed_yaml() {
        let content = r#"---
title: "Malformed YAML
invalid: [unclosed array
unquoted: string with: colons: everywhere
---
Content after malformed YAML"#;
        
        let result = frontmatter::parse_string(content);
        assert!(result.is_ok());
        
        // When YAML parsing fails, should return empty frontmatter and original content
        let (frontmatter, body) = result.unwrap();
        assert!(frontmatter.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_string_yaml_with_special_characters() {
        let content = r#"---
title: "Title with unicode: 测试 🎌 ñáéíóú"
description: |
  Multi-line description
  with special chars: @#$%^&*()
  and unicode: こんにちは世界
tags: ["tag-with-dashes", "tag_with_underscores", "unicode-tag-测试"]
special_numbers: [1, 2.5, -3, 1.5e10]
boolean_values: [true, false, null]
---
Content with unicode: 世界 🌍"#;

        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert_eq!(frontmatter["title"], "Title with unicode: 测试 🎌 ñáéíóú");
        assert!(frontmatter["description"].as_str().unwrap().contains("Multi-line"));
        assert!(frontmatter["tags"].is_array());
        assert!(frontmatter["special_numbers"].is_array());
        assert!(frontmatter["boolean_values"].is_array());
        assert_eq!(body, "Content with unicode: 世界 🌍");
    }

    #[test]
    fn test_parse_string_nested_yaml_structures() {
        let content = r#"---
metadata:
  author:
    name: "John Doe"
    email: "john@example.com"
  publication:
    year: 2023
    journal: "Test Journal"
tags:
  - category: "research"
    priority: 1
  - category: "draft"
    priority: 2
complex_data:
  matrix: [[1, 2], [3, 4]]
  settings:
    enabled: true
    config:
      timeout: 30
      retries: 3
---
Content with nested structures"#;

        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert!(frontmatter.contains_key("metadata"));
        assert!(frontmatter.contains_key("tags"));
        assert!(frontmatter.contains_key("complex_data"));
        assert_eq!(body, "Content with nested structures");
        
        // Test nested access
        if let Value::Object(metadata) = &frontmatter["metadata"] {
            assert!(metadata.contains_key("author"));
            assert!(metadata.contains_key("publication"));
        }
    }

    #[test]
    fn test_parse_string_yaml_with_quotes() {
        let content = r#"---
single_quoted: 'Single quoted string with "double quotes" inside'
double_quoted: "Double quoted string with 'single quotes' inside"
mixed_quotes: "String with \"escaped quotes\" and 'normal singles'"
special_yaml: 'YAML: special chars like : and - and #'
---
Content"#;

        let (frontmatter, _body) = frontmatter::parse_string(content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert!(frontmatter["single_quoted"].as_str().unwrap().contains("double quotes"));
        assert!(frontmatter["double_quoted"].as_str().unwrap().contains("single quotes"));
        assert!(frontmatter["mixed_quotes"].as_str().unwrap().contains("escaped quotes"));
        assert!(frontmatter["special_yaml"].as_str().unwrap().contains("YAML:"));
    }

    #[test]
    fn test_parse_string_no_frontmatter_with_dashes() {
        // Content that starts with dashes but isn't frontmatter
        let content = "---\nThis is not frontmatter, just content that starts with dashes\n---\nMore content";
        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        // Should recognize this as incomplete frontmatter and return original content
        assert!(frontmatter.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_string_frontmatter_only() {
        let content = r#"---
title: "Frontmatter Only"
status: "draft"
---"#;

        let (frontmatter, body) = frontmatter::parse_string(content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert_eq!(frontmatter["title"], "Frontmatter Only");
        assert_eq!(frontmatter["status"], "draft");
        assert_eq!(body, "");
    }

    #[test]
    fn test_serialize_with_empty_frontmatter() {
        let frontmatter = HashMap::new();
        let content = "Just plain content without frontmatter";
        
        let result = frontmatter::serialize_with_frontmatter(&frontmatter, content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_serialize_with_complex_values() {
        let mut frontmatter = HashMap::new();
        frontmatter.insert("title".to_string(), Value::String("Complex Test".to_string()));
        frontmatter.insert("tags".to_string(), json!(["rust", "test", "unicode-测试"]));
        frontmatter.insert("metadata".to_string(), json!({
            "author": "Test Author",
            "nested": {
                "deep": "value",
                "array": [1, 2, 3]
            }
        }));
        frontmatter.insert("numbers".to_string(), json!([1, 2.5, -3.14159]));
        frontmatter.insert("boolean".to_string(), json!(true));
        frontmatter.insert("null_value".to_string(), json!(null));
        
        let content = "Content with complex frontmatter";
        let result = frontmatter::serialize_with_frontmatter(&frontmatter, content);
        
        assert!(result.is_ok());
        let serialized = result.unwrap();
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("---\n"));
        assert!(serialized.ends_with(content));
        assert!(serialized.contains("Complex Test"));
    }

    #[test]
    fn test_serialize_roundtrip() {
        let original_content = r#"---
title: "Roundtrip Test"
tags: ["test", "roundtrip"]
metadata:
  count: 42
  enabled: true
---
Original content here"#;

        // Parse the content
        let (frontmatter, body) = frontmatter::parse_string(original_content).unwrap();
        
        // Serialize it back
        let serialized = frontmatter::serialize_with_frontmatter(&frontmatter, &body).unwrap();
        
        // Parse again to verify roundtrip
        let (roundtrip_frontmatter, roundtrip_body) = frontmatter::parse_string(&serialized).unwrap();
        
        assert_eq!(frontmatter.len(), roundtrip_frontmatter.len());
        assert_eq!(body, roundtrip_body);
        assert_eq!(frontmatter["title"], roundtrip_frontmatter["title"]);
        assert_eq!(frontmatter["tags"], roundtrip_frontmatter["tags"]);
    }

    #[test]
    fn test_update_frontmatter_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("update_test.md");
        
        // Create initial file
        fs::write(&test_file, r#"---
title: "Original Title"
---
Original content"#).unwrap();
        
        // Update frontmatter
        let result = frontmatter::update_frontmatter(&test_file, "status", json!("published"));
        assert!(result.is_ok());
        
        // Verify update
        let (updated_frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert_eq!(updated_frontmatter["title"], "Original Title");
        assert_eq!(updated_frontmatter["status"], "published");
        assert!(updated_frontmatter.contains_key("modified"));
    }

    #[test]
    fn test_update_frontmatter_no_existing_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("no_fm_test.md");
        
        // Create file without frontmatter
        fs::write(&test_file, "Just plain content").unwrap();
        
        // Update should add frontmatter
        let result = frontmatter::update_frontmatter(&test_file, "new_field", json!("new_value"));
        assert!(result.is_ok());
        
        // Verify frontmatter was added
        let (updated_frontmatter, content) = frontmatter::parse_file(&test_file).unwrap();
        assert_eq!(updated_frontmatter["new_field"], "new_value");
        assert!(updated_frontmatter.contains_key("modified"));
        assert_eq!(content, "Just plain content");
    }

    #[test]
    fn test_update_frontmatter_complex_values() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("complex_update.md");
        
        fs::write(&test_file, "# Content").unwrap();
        
        // Update with complex nested value
        let complex_value = json!({
            "nested": {
                "array": [1, 2, 3],
                "string": "test",
                "boolean": true
            },
            "tags": ["a", "b", "c"]
        });
        
        let result = frontmatter::update_frontmatter(&test_file, "complex_data", complex_value.clone());
        assert!(result.is_ok());
        
        // Verify complex value was stored correctly
        let (updated_frontmatter, _content) = frontmatter::parse_file(&test_file).unwrap();
        assert_eq!(updated_frontmatter["complex_data"], complex_value);
    }

    #[test]
    fn test_update_frontmatter_readonly_file() {
        let temp_dir = TempDir::new().unwrap();
        let readonly_file = temp_dir.path().join("readonly.md");
        
        // Create file
        fs::write(&readonly_file, "Content").unwrap();
        
        // Make file readonly (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
            perms.set_mode(0o444); // readonly
            fs::set_permissions(&readonly_file, perms).unwrap();
            
            // Update should fail
            let result = frontmatter::update_frontmatter(&readonly_file, "test", json!("value"));
            assert!(result.is_err());
            
            // Restore write permissions for cleanup
            let mut restore_perms = fs::metadata(&readonly_file).unwrap().permissions();
            restore_perms.set_mode(0o644);
            fs::set_permissions(&readonly_file, restore_perms).unwrap();
        }
    }

    #[test]
    fn test_add_default_frontmatter_basic() {
        let mut frontmatter = HashMap::new();
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "Test Title", "uid");
        
        assert_eq!(frontmatter["title"], "Test Title");
        assert!(frontmatter.contains_key("created"));
        assert!(frontmatter.contains_key("modified"));
        assert!(frontmatter.contains_key("uid"));
        
        // Verify UUID format
        let uid_str = frontmatter["uid"].as_str().unwrap();
        assert_eq!(uid_str.len(), 36); // Standard UUID length
        assert!(uid_str.contains("-"));
    }

    #[test]
    fn test_add_default_frontmatter_custom_ident_key() {
        let mut frontmatter = HashMap::new();
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "Custom Key Test", "custom_id");
        
        assert_eq!(frontmatter["title"], "Custom Key Test");
        assert!(frontmatter.contains_key("custom_id"));
        assert!(!frontmatter.contains_key("uid"));
        
        // Verify the custom_id is a valid UUID
        let id_str = frontmatter["custom_id"].as_str().unwrap();
        assert_eq!(id_str.len(), 36);
    }

    #[test]
    fn test_add_default_frontmatter_existing_values() {
        let mut frontmatter = HashMap::new();
        frontmatter.insert("title".to_string(), json!("Existing Title"));
        frontmatter.insert("custom_field".to_string(), json!("Keep Me"));
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "New Title", "uid");
        
        // Should overwrite title but keep other fields
        assert_eq!(frontmatter["title"], "New Title");
        assert_eq!(frontmatter["custom_field"], "Keep Me");
        assert!(frontmatter.contains_key("uid"));
        assert!(frontmatter.contains_key("created"));
        assert!(frontmatter.contains_key("modified"));
    }

    #[test]
    fn test_add_default_frontmatter_unicode_title() {
        let mut frontmatter = HashMap::new();
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "Unicode Title 测试 🌟", "identifier");
        
        assert_eq!(frontmatter["title"], "Unicode Title 测试 🌟");
        assert!(frontmatter.contains_key("identifier"));
    }

    #[test]
    fn test_add_default_frontmatter_empty_title() {
        let mut frontmatter = HashMap::new();
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "", "uid");
        
        assert_eq!(frontmatter["title"], "");
        assert!(frontmatter.contains_key("uid"));
    }

    #[test]
    fn test_add_default_frontmatter_special_characters_in_key() {
        let mut frontmatter = HashMap::new();
        
        frontmatter::add_default_frontmatter(&mut frontmatter, "Test", "special-key_with.dots");
        
        assert!(frontmatter.contains_key("special-key_with.dots"));
        let id_str = frontmatter["special-key_with.dots"].as_str().unwrap();
        assert_eq!(id_str.len(), 36); // Still a valid UUID
    }

    #[test]
    fn test_yaml_date_formats() {
        let content = r#"---
created: 2023-01-15T10:30:00Z
modified: "2023-01-16 15:45:30"
date_only: 2023-01-17
time_only: "14:30:00"
custom_format: "Jan 15, 2023"
---
Content with various date formats"#;

        let (frontmatter, _body) = frontmatter::parse_string(content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert!(frontmatter.contains_key("created"));
        assert!(frontmatter.contains_key("modified"));
        assert!(frontmatter.contains_key("date_only"));
        assert!(frontmatter.contains_key("time_only"));
        assert!(frontmatter.contains_key("custom_format"));
    }

    #[test]
    fn test_very_large_frontmatter() {
        let mut content = String::from("---\n");
        
        // Create frontmatter with many fields
        for i in 0..100 {
            content.push_str(&format!("field_{}: \"value_{}\"\n", i, i));
        }
        
        content.push_str("tags:\n");
        for i in 0..50 {
            content.push_str(&format!("  - tag_{}\n", i));
        }
        
        content.push_str("---\nContent with large frontmatter");
        
        let (frontmatter, body) = frontmatter::parse_string(&content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert!(frontmatter.len() >= 100); // At least 100 fields + tags array
        assert_eq!(body, "Content with large frontmatter");
        
        // Verify some fields
        assert_eq!(frontmatter["field_0"], "value_0");
        assert_eq!(frontmatter["field_99"], "value_99");
        assert!(frontmatter["tags"].is_array());
    }

    #[test]
    fn test_frontmatter_with_very_long_strings() {
        let long_string = "a".repeat(10000);
        let content = format!(r#"---
title: "Regular Title"
long_description: "{}"
---
Content here"#, long_string);

        let (frontmatter, body) = frontmatter::parse_string(&content).unwrap();
        
        assert!(!frontmatter.is_empty());
        assert_eq!(frontmatter["title"], "Regular Title");
        assert_eq!(frontmatter["long_description"], long_string);
        assert_eq!(body, "Content here");
    }
}
