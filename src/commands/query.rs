use crate::errors::{ConfigError, Result};
use crate::frontmatter;
use crate::types::{OutputStyle, QueryResult, Vault};
use crate::utils::{contains_value, format_value, is_path_blacklisted, matches_value};
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, ContentArrangement, Table,
};
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct QueryOptions<'a> {
    pub key: &'a str,
    pub value: Option<&'a str>,
    pub contains: Option<&'a str>,
    pub exists: bool,
    pub missing: bool,
    pub style: OutputStyle,
    pub count: bool,
}

/// Builder for constructing QueryOptions with fluent API
#[derive(Debug)]
pub struct QueryOptionsBuilder<'a> {
    key: Option<&'a str>,
    value: Option<&'a str>,
    contains: Option<&'a str>,
    exists: bool,
    missing: bool,
    style: OutputStyle,
    count: bool,
}

impl<'a> Default for QueryOptionsBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> QueryOptionsBuilder<'a> {
    /// Create a new QueryOptionsBuilder
    pub fn new() -> Self {
        Self {
            key: None,
            value: None,
            contains: None,
            exists: false,
            missing: false,
            style: OutputStyle::Path,
            count: false,
        }
    }

    /// Set the key to search for
    pub fn key(mut self, key: &'a str) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the exact value to match
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    /// Set the substring to search for in values
    pub fn contains(mut self, contains: &'a str) -> Self {
        self.contains = Some(contains);
        self
    }

    /// Only return files where the key exists
    pub fn exists(mut self, exists: bool) -> Self {
        self.exists = exists;
        self
    }

    /// Only return files where the key is missing
    pub fn missing(mut self, missing: bool) -> Self {
        self.missing = missing;
        self
    }

    /// Set the output style
    pub fn style(mut self, style: OutputStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable count mode (show counts instead of listing files)
    pub fn count(mut self, count: bool) -> Self {
        self.count = count;
        self
    }

    /// Build the QueryOptions instance
    pub fn build(self) -> std::result::Result<QueryOptions<'a>, &'static str> {
        let key = self.key.ok_or("Key is required for query")?;

        // Validate that conflicting options aren't set
        if self.value.is_some() && self.contains.is_some() {
            return Err("Cannot specify both value and contains options");
        }

        Ok(QueryOptions {
            key,
            value: self.value,
            contains: self.contains,
            exists: self.exists,
            missing: self.missing,
            style: self.style,
            count: self.count,
        })
    }
}

impl<'a> QueryOptions<'a> {
    /// Create a new QueryOptionsBuilder
    pub fn builder() -> QueryOptionsBuilder<'a> {
        QueryOptionsBuilder::new()
    }
}

pub fn execute(vault: &Vault, options: QueryOptions<'_>) -> Result<()> {
    if options.value.is_some() && options.contains.is_some() {
        return Err(crate::errors::ObsidianError::InvalidArguments {
            message: "Cannot specify both --value and --contains options".to_string(),
        });
    }

    if vault.verbose {
        println!("Searching for frontmatter key: {}", options.key);
        if let Some(v) = options.value {
            println!("Filtering for exact value: {v}");
        }
        if let Some(c) = options.contains {
            println!("Filtering for substring: {c}");
        }
        if options.exists {
            println!("Filtering for key existence");
        }
        if options.missing {
            println!("Filtering for key absence");
        }
    }
    let mut matches = Vec::new();

    for entry in WalkDir::new(&vault.path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() || entry.path().extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        let relative_path = match entry.path().strip_prefix(&vault.path) {
            Ok(path) => path,
            Err(_) => {
                if vault.verbose {
                    eprintln!(
                        "{}",
                        format!(
                            "Could not resolve relative path for {}",
                            entry.path().display()
                        )
                        .yellow()
                    );
                }
                continue;
            }
        };

        // Skip files in blacklisted directories
        if is_path_blacklisted(relative_path, &vault.blacklist) {
            if vault.verbose {
                println!("Skipping excluded file: {}", relative_path.display());
            }
            continue;
        }

        let (frontmatter, _content) = match frontmatter::parse_file(entry.path()) {
            Ok(parsed) => parsed,
            Err(_) => {
                if vault.verbose {
                    eprintln!(
                        "{}",
                        format!("Could not parse frontmatter in {}", relative_path.display())
                            .yellow()
                    );
                }
                continue;
            }
        };

        // Check if key exists and apply filters
        let has_key = frontmatter.contains_key(options.key);

        // Apply filters
        if options.missing && has_key {
            continue;
        }
        if options.exists && !has_key {
            continue;
        }

        if has_key {
            let metadata_value =
                frontmatter
                    .get(options.key)
                    .ok_or_else(|| ConfigError::InvalidValue {
                        field: options.key.to_string(),
                        value: "missing from frontmatter".to_string(),
                    })?;

            // Value filtering
            if let Some(expected_value) = options.value {
                if !matches_value(metadata_value, expected_value) {
                    continue;
                }
            }

            // Contains filtering
            if let Some(contains_str) = options.contains {
                if !contains_value(metadata_value, contains_str) {
                    continue;
                }
            }
        } else if !options.missing {
            // If the key doesn't exist and we're not specifically looking for missing keys
            continue;
        }

        // If we got here, the file matches all criteria
        matches.push(QueryResult {
            path: relative_path.to_path_buf(),
            frontmatter: frontmatter.clone(),
            value: frontmatter.get(options.key).cloned(),
        });
    }

    // Display results
    if options.count {
        println!("Found {} matching files", matches.len());
    } else {
        display_query_results(&matches, options.style, options.key)?;
    }

    Ok(())
}

fn display_query_results(matches: &[QueryResult], style: OutputStyle, _key: &str) -> Result<()> {
    if matches.is_empty() {
        eprintln!("{}", "No matching files found".yellow());
        return Ok(());
    }

    match style {
        OutputStyle::Path => {
            for result in matches {
                println!("{}", result.path.display());
            }
        }
        OutputStyle::Title => {
            for result in matches {
                let title = result
                    .frontmatter
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        result
                            .path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                    });
                println!("{}: {}", result.path.display(), title);
            }
        }
        OutputStyle::Table => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Path").add_attribute(Attribute::Bold),
                    Cell::new("Property").add_attribute(Attribute::Bold),
                    Cell::new("Value").add_attribute(Attribute::Bold),
                ]);

            for result in matches {
                let path_str = result.path.to_string_lossy();
                let mut first_row = true;

                for (k, v) in &result.frontmatter {
                    table.add_row(vec![
                        if first_row { path_str.as_ref() } else { "" },
                        k,
                        &format_value(v),
                    ]);
                    first_row = false;
                }

                if !result.frontmatter.is_empty() {
                    table.add_row(vec!["", "", ""]);
                }
            }

            println!("{table}");
            println!("Total matches: {}", matches.len());
        }
        OutputStyle::Json => {
            let json_results: Vec<serde_json::Map<String, Value>> = matches
                .iter()
                .map(|result| {
                    let mut obj = serde_json::Map::new();
                    obj.insert(
                        "path".to_string(),
                        Value::String(format!("{}", result.path.display())),
                    );
                    obj.insert(
                        "frontmatter".to_string(),
                        Value::Object(
                            result
                                .frontmatter
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect(),
                        ),
                    );
                    if let Some(value) = &result.value {
                        obj.insert("value".to_string(), value.clone());
                    }
                    obj
                })
                .collect();

            let json_output = serde_json::to_string_pretty(&json_results).map_err(|e| {
                ConfigError::InvalidValue {
                    field: "json_serialization".to_string(),
                    value: format!("failed: {e}"),
                }
            })?;
            println!("{json_output}");
        }
    }
    Ok(())
}
