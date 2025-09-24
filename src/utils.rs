use crate::errors::{ObsidianError, Result};
use crate::frontmatter;
use crate::template;
use crate::types::{FileTypeStat, TemplateVars, Vault, VaultInfo};
use chrono::{DateTime, Datelike, Local};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn is_path_blacklisted(path: &Path, blacklist: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    blacklist.iter().any(|pattern| {
        if pattern.contains('*') {
            // Handle glob patterns
            glob_match(pattern, &path_str)
        } else {
            // Handle simple patterns - check both prefix and path component matching
            path_str.starts_with(pattern)
                || path
                    .components()
                    .any(|component| component.as_os_str().to_string_lossy() == *pattern)
        }
    })
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    fn match_recursive(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
        if pi >= pattern.len() {
            return ti >= text.len();
        }

        if pattern[pi] == '*' {
            // Try matching zero characters
            if match_recursive(pattern, text, pi + 1, ti) {
                return true;
            }
            // Try matching one or more characters
            for i in ti..text.len() {
                if match_recursive(pattern, text, pi + 1, i + 1) {
                    return true;
                }
            }
            false
        } else if ti >= text.len() || pattern[pi] != text[ti] {
            false
        } else {
            match_recursive(pattern, text, pi + 1, ti + 1)
        }
    }

    match_recursive(&pattern_chars, &text_chars, 0, 0)
}

pub fn find_matching_files(
    vault: &Path,
    search_term: &str,
    exact_match: bool,
) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();
    let matcher = SkimMatcherV2::default();
    for entry in WalkDir::new(vault)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            let relative_path =
                entry
                    .path()
                    .strip_prefix(vault)
                    .map_err(|_| ObsidianError::FileNotFound {
                        path: entry.path().to_string_lossy().to_string(),
                    })?;

            // Check filename match
            let file_stem = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let filename_matches = if exact_match {
                file_stem == search_term
            } else {
                file_stem
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
                    || matcher.fuzzy_match(file_stem, search_term).is_some()
            };

            if filename_matches {
                matches.push(relative_path.to_path_buf());
                continue;
            }

            // Check title in frontmatter (only for non-exact matches)
            if !exact_match {
                if let Ok((frontmatter, _)) = frontmatter::parse_file(entry.path()) {
                    if let Some(Value::String(title)) = frontmatter.get("title") {
                        if title.to_lowercase().contains(&search_term.to_lowercase())
                            || matcher.fuzzy_match(title, search_term).is_some()
                        {
                            matches.push(relative_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    Ok(matches)
}

pub fn resolve_page_path(page_or_path: &Path, vault: &Path) -> Result<PathBuf> {
    let mut path = page_or_path.to_path_buf();

    // Add .md extension if not present
    if path.extension().is_none() {
        path.set_extension("md");
    }

    // Check if it exists as an absolute path
    if path.is_absolute() && path.exists() {
        return Ok(path);
    }

    // Check relative to vault
    let vault_path = vault.join(&path);
    if vault_path.exists() {
        return Ok(vault_path);
    }

    Err(ObsidianError::FileNotFound {
        path: format!(
            "Page or file '{}' not found in vault: {}",
            page_or_path.display(),
            vault.display()
        ),
    })
}

pub fn get_template_vars(date: DateTime<Local>) -> TemplateVars {
    TemplateVars {
        year: date.year(),
        month: date.month(),
        day: date.day(),
        month_name: date.format("%B").to_string(),
        month_abbr: date.format("%b").to_string(),
        weekday: date.format("%A").to_string(),
        weekday_abbr: date.format("%a").to_string(),
    }
}

pub fn format_journal_template(template_str: &str, vars: &TemplateVars) -> Result<String> {
    // Use the new flexible template engine
    template::format_journal_template_with_vars(template_str, vars)
}

pub fn get_vault_info(vault: &Vault) -> Result<VaultInfo> {
    let mut file_type_stats: HashMap<String, FileTypeStat> = HashMap::new();
    let mut total_files = 0;
    let mut total_directories = 0;
    let mut usage_files = 0;
    let mut usage_directories = 0;
    let mut markdown_files = 0;

    for entry in WalkDir::new(&vault.path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let relative_path =
            entry
                .path()
                .strip_prefix(&vault.path)
                .map_err(|_| ObsidianError::FileNotFound {
                    path: entry.path().to_string_lossy().to_string(),
                })?;

        if is_path_blacklisted(relative_path, &vault.blacklist) {
            continue;
        }

        if entry.file_type().is_dir() {
            total_directories += 1;
            if let Ok(metadata) = entry.metadata() {
                usage_directories += metadata.len();
            }
        } else if entry.file_type().is_file() {
            total_files += 1;

            let extension = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("(no extension)")
                .to_string();

            let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            usage_files += file_size;

            if extension == "md" {
                markdown_files += 1;
            }

            let stat = file_type_stats.entry(extension).or_insert(FileTypeStat {
                count: 0,
                total_size: 0,
            });
            stat.count += 1;
            stat.total_size += file_size;
        }
    }

    let now = Local::now();
    let template_vars = get_template_vars(now);
    let journal_path = format_journal_template(&vault.journal_template, &template_vars)?;

    Ok(VaultInfo {
        vault_path: vault.path.clone(),
        total_files,
        total_directories,
        usage_files,
        usage_directories,
        file_type_stats,
        markdown_files,
        blacklist: vault.blacklist.clone(),
        editor: vault.editor.clone(),
        journal_template: vault.journal_template.clone(),
        journal_path,
        verbose: vault.verbose,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub fn launch_editor(editor: &str, file_path: &Path) -> Result<()> {
    let status = Command::new(editor).arg(file_path).status().map_err(|e| {
        ObsidianError::EditorExecution(format!("Failed to execute editor '{}': {}", editor, e))
    })?;

    if !status.success() {
        return Err(ObsidianError::EditorExecution(format!(
            "Editor '{}' exited with code: {:?}",
            editor,
            status.code()
        )));
    }

    Ok(())
}

/// Wrap filename at specified width, preferring to break at path separators
pub fn wrap_filename(filename: &str, max_width: usize) -> String {
    if filename.len() <= max_width {
        return filename.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();

    // Split by path separators first
    let parts: Vec<&str> = filename.split('/').collect();

    for (i, part) in parts.iter().enumerate() {
        let separator = if i == 0 { "" } else { "/" };
        let part_with_separator = format!("{}{}", separator, part);

        // If adding this part would exceed the width, start a new line
        if !current_line.is_empty() && current_line.len() + part_with_separator.len() > max_width {
            result.push_str(&current_line);
            result.push('\n');
            current_line = part.to_string(); // Start new line without separator
        } else {
            current_line.push_str(&part_with_separator);
        }

        // If even a single part is too long, break it within the part
        if current_line.len() > max_width {
            let mut temp_line = String::new();
            for ch in current_line.chars() {
                if temp_line.len() >= max_width {
                    result.push_str(&temp_line);
                    result.push('\n');
                    temp_line = ch.to_string();
                } else {
                    temp_line.push(ch);
                }
            }
            current_line = temp_line;
        }
    }

    // Add any remaining content
    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

/// Extract created and modified dates from frontmatter or filesystem
pub fn get_file_dates(file_path: &Path) -> (String, String) {
    // Try to get dates from frontmatter first
    if let Ok((frontmatter, _)) = frontmatter::parse_file(file_path) {
        let created = extract_date_from_frontmatter(&frontmatter, "created")
            .unwrap_or_else(|| get_filesystem_created_date(file_path));

        let modified = extract_date_from_frontmatter(&frontmatter, "modified")
            .unwrap_or_else(|| get_filesystem_modified_date(file_path));

        (created, modified)
    } else {
        // Fallback to filesystem dates
        (
            get_filesystem_created_date(file_path),
            get_filesystem_modified_date(file_path),
        )
    }
}

/// Extract date from frontmatter field and format as YYYY-MM-DD
pub fn extract_date_from_frontmatter(
    frontmatter: &HashMap<String, Value>,
    field: &str,
) -> Option<String> {
    frontmatter.get(field).and_then(|value| {
        match value {
            Value::String(date_str) => {
                // Try to parse ISO 8601 format (RFC3339)
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(date_str) {
                    Some(datetime.format("%Y-%m-%d").to_string())
                } else if let Ok(naive_date) =
                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                {
                    // Already in YYYY-MM-DD format
                    Some(naive_date.format("%Y-%m-%d").to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

/// Get filesystem created date formatted as YYYY-MM-DD
pub fn get_filesystem_created_date(file_path: &Path) -> String {
    std::fs::metadata(file_path)
        .and_then(|metadata| metadata.created())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get filesystem modified date formatted as YYYY-MM-DD
pub fn get_filesystem_modified_date(file_path: &Path) -> String {
    std::fs::metadata(file_path)
        .and_then(|metadata| metadata.modified())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Format a JSON value as a readable string
pub fn format_value(value: &Value) -> String {
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

/// Parse a string into a JSON value with intelligent type detection
pub fn parse_value(s: &str) -> Value {
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

/// Check if a JSON value matches an expected string
pub fn matches_value(metadata_value: &Value, expected: &str) -> bool {
    match metadata_value {
        Value::String(s) => s == expected,
        Value::Number(n) => n.to_string() == expected,
        Value::Bool(b) => b.to_string() == expected,
        _ => format!("{}", metadata_value) == expected,
    }
}

/// Check if a JSON value contains a specific string
pub fn contains_value(metadata_value: &Value, contains_str: &str) -> bool {
    match metadata_value {
        Value::String(s) => s.contains(contains_str),
        Value::Array(arr) => arr.iter().any(|v| contains_value(v, contains_str)),
        _ => format!("{}", metadata_value).contains(contains_str),
    }
}
