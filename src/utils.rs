use crate::errors::{ObsidianError, Result};
use crate::frontmatter;
use crate::template;
use crate::types::{FileTypeStat, State, TemplateVars, VaultInfo};
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
            path_str.starts_with(pattern) || 
            path.components().any(|component| {
                component.as_os_str().to_string_lossy() == *pattern
            })
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
        } else {
            if ti >= text.len() || pattern[pi] != text[ti] {
                false
            } else {
                match_recursive(pattern, text, pi + 1, ti + 1)
            }
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

pub fn get_vault_info(state: &State) -> Result<VaultInfo> {
    let mut file_type_stats: HashMap<String, FileTypeStat> = HashMap::new();
    let mut total_files = 0;
    let mut total_directories = 0;
    let mut usage_files = 0;
    let mut usage_directories = 0;
    let mut markdown_files = 0;

    for entry in WalkDir::new(&state.vault)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let relative_path =
            entry
                .path()
                .strip_prefix(&state.vault)
                .map_err(|_| ObsidianError::FileNotFound {
                    path: entry.path().to_string_lossy().to_string(),
                })?;

        if is_path_blacklisted(relative_path, &state.blacklist) {
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
    let journal_path = format_journal_template(&state.journal_template, &template_vars)?;

    Ok(VaultInfo {
        vault_path: state.vault.clone(),
        total_files,
        total_directories,
        usage_files,
        usage_directories,
        file_type_stats,
        markdown_files,
        blacklist: state.blacklist.clone(),
        editor: state.editor.clone(),
        journal_template: state.journal_template.clone(),
        journal_path,
        verbose: state.verbose,
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
