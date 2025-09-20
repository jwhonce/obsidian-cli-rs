use crate::errors::{ObsidianError, Result};
use chrono::Utc;
use gray_matter::{engine::YAML, Matter};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

// Global static instance for better performance
static MATTER: std::sync::LazyLock<Matter<YAML>> = std::sync::LazyLock::new(Matter::<YAML>::new);

/// Parse frontmatter and content from a file
pub fn parse_file(path: &Path) -> Result<(HashMap<String, Value>, String)> {
    let content = std::fs::read_to_string(path)?;
    parse_string(&content)
}

/// Parse frontmatter and content from a string
pub fn parse_string(content: &str) -> Result<(HashMap<String, Value>, String)> {
    match MATTER.parse::<Value>(content) {
        Ok(parsed) => {
            let frontmatter = if let Some(Value::Object(map)) = parsed.data {
                // Data is already a serde_json::Value, extract as HashMap if it's an Object
                map.into_iter().collect()
            } else {
                HashMap::new()
            };

            // Check for incomplete frontmatter case - if frontmatter is empty but content
            // doesn't match original input, it might be incomplete frontmatter
            if frontmatter.is_empty()
                && content.starts_with("---\n")
                && !content.starts_with("---\n---\n")
            {
                // This looks like incomplete frontmatter, return original content
                Ok((HashMap::new(), content.to_string()))
            } else {
                Ok((frontmatter, parsed.content))
            }
        }
        Err(_) => {
            // If parsing fails, treat the entire content as having no frontmatter
            Ok((HashMap::new(), content.to_string()))
        }
    }
}

/// Serialize frontmatter and content back to a markdown string
pub fn serialize_with_frontmatter(
    frontmatter: &HashMap<String, Value>,
    content: &str,
) -> Result<String> {
    if frontmatter.is_empty() {
        return Ok(content.to_string());
    }

    // Convert HashMap to serde_json::Value::Object for serialization
    let frontmatter_obj: serde_json::Map<String, Value> = frontmatter
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let frontmatter_value = Value::Object(frontmatter_obj);

    // Manually serialize YAML frontmatter in standard format
    let yaml_data = serde_yaml::to_string(&frontmatter_value)
        .map_err(|e| ObsidianError::FrontmatterParsing(e.to_string()))?;

    Ok(format!("---\n{}---\n{}", yaml_data, content))
}

/// Update frontmatter in a file with a new key-value pair and auto-update modification time
pub fn update_frontmatter(path: &Path, key: &str, value: Value) -> Result<()> {
    let (mut frontmatter, content) = parse_file(path)?;

    frontmatter.insert(key.to_string(), value);
    frontmatter.insert(
        "modified".to_string(),
        Value::String(Utc::now().to_rfc3339()),
    );

    let serialized = serialize_with_frontmatter(&frontmatter, &content)?;
    std::fs::write(path, serialized)?;

    Ok(())
}

/// Add default frontmatter fields (created, modified, title, and unique identifier)
pub fn add_default_frontmatter(
    frontmatter: &mut HashMap<String, Value>,
    title: &str,
    ident_key: &str,
) {
    let now = Utc::now().to_rfc3339();

    frontmatter.insert("created".to_string(), Value::String(now.clone()));
    frontmatter.insert("modified".to_string(), Value::String(now));
    frontmatter.insert("title".to_string(), Value::String(title.to_string()));
    frontmatter.insert(
        ident_key.to_string(),
        Value::String(Uuid::new_v4().to_string()),
    );
}
