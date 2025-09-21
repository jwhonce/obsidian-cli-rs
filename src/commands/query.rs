use crate::errors::Result;
use crate::frontmatter;
use crate::types::{OutputStyle, QueryResult, Vault};
use crate::utils::{is_path_blacklisted, format_value, matches_value, contains_value};
use colored::*;
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

pub async fn execute(vault: &Vault, options: QueryOptions<'_>) -> Result<()> {
    if options.value.is_some() && options.contains.is_some() {
        eprintln!(
            "{}",
            "Cannot specify both --value and --contains options".red()
        );
        std::process::exit(1);
    }

    if vault.verbose {
        println!("Searching for frontmatter key: {}", options.key);
        if let Some(v) = options.value {
            println!("Filtering for exact value: {}", v);
        }
        if let Some(c) = options.contains {
            println!("Filtering for substring: {}", c);
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
        .filter_map(|e| e.ok())
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
            let metadata_value = frontmatter.get(options.key).unwrap();

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
        display_query_results(&matches, options.style, options.key);
    }

    Ok(())
}


fn display_query_results(matches: &[QueryResult], style: OutputStyle, _key: &str) {
    if matches.is_empty() {
        eprintln!("{}", "No matching files found".yellow());
        return;
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

            println!("{}", table);
            println!("Total matches: {}", matches.len());
        }
        OutputStyle::Json => {
            let json_results: Vec<serde_json::Map<String, Value>> = matches
                .iter()
                .map(|result| {
                    let mut obj = serde_json::Map::new();
                    obj.insert(
                        "path".to_string(),
                        Value::String(result.path.to_string_lossy().to_string()),
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

            println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
        }
    }
}

