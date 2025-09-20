use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use chrono::Utc;
use colored::*;
use serde_json::Value;
use std::path::Path;

pub async fn execute(
    state: &State,
    page_or_path: &Path,
    key: Option<&str>,
    value: Option<&str>,
) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(state, page_or_path)?;
    let (frontmatter, _content) = frontmatter::parse_file(&file_path)?;

    match (key, value) {
        // List all frontmatter metadata
        (None, None) => {
            if frontmatter.is_empty() {
                eprintln!("{}", "No frontmatter metadata found for this page".red());
            } else {
                for (k, v) in &frontmatter {
                    println!("{}: {}", k, format_value(v));
                }
            }
        }
        // Display specific key
        (Some(k), None) => {
            if let Some(v) = frontmatter.get(k) {
                println!("{}: {}", k, format_value(v));
            } else {
                eprintln!(
                    "{}",
                    format!(
                        "Frontmatter metadata '{}' not found in '{}'",
                        k,
                        page_or_path.display()
                    )
                    .red()
                );
                std::process::exit(1);
            }
        }
        // Update key with value
        (Some(k), Some(v)) => {
            let new_value = parse_value(v);
            frontmatter::update_frontmatter(&file_path, k, new_value)?;

            if state.verbose {
                println!(
                    "Updated frontmatter metadata {{ '{}': '{}', 'modified': '{}' }} in {}",
                    k,
                    v,
                    Utc::now().to_rfc3339(),
                    file_path.display()
                );
            }
        }
        // Invalid combination (Some(None), Some(_)) - shouldn't happen with CLI
        (None, Some(_)) => unreachable!("CLI should prevent this case"),
    }

    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => format!(
            "[{}]",
            arr.iter().map(format_value).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(_) => "{ object }".to_string(),
        Value::Null => "null".to_string(),
    }
}

fn parse_value(s: &str) -> Value {
    // Try to parse as different types
    if let Ok(b) = s.parse::<bool>() {
        return Value::Bool(b);
    }

    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(serde_json::Number::from(n));
    }

    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Value::Number(n);
        }
    }

    // Try to parse as array (simple comma-separated values)
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let items: Vec<Value> = inner
            .split(',')
            .map(|item| Value::String(item.trim().to_string()))
            .collect();
        return Value::Array(items);
    }

    // Default to string
    Value::String(s.to_string())
}
