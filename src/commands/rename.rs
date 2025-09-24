use crate::errors::{ObsidianError, Result};
use crate::types::Vault;
use crate::utils::{is_path_blacklisted, wrap_filename};
use anyhow;
use colored::*;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub async fn execute(vault: &Vault, page_or_path: &Path, new_name: &str, update_links: bool) -> Result<()> {
    let old_file_path = crate::resolve_page_or_path!(vault, page_or_path)?;
    
    // Validate that the source file exists
    if !old_file_path.exists() {
        return Err(ObsidianError::FileNotFound {
            path: old_file_path.display().to_string(),
        });
    }

    // Construct the new file path
    let mut new_file_path = old_file_path.clone();
    
    // Determine if new_name is just a filename or a full path
    let new_name_path = Path::new(new_name);
    if new_name_path.parent().is_some() && new_name_path.parent().unwrap() != Path::new("") {
        // new_name contains directory components, treat as relative to vault
        new_file_path = vault.path.join(new_name);
    } else {
        // new_name is just a filename, keep in same directory
        new_file_path.set_file_name(new_name);
    }
    
    // Ensure the new filename has .md extension if the original did
    if old_file_path.extension().is_some_and(|ext| ext == "md") && 
       new_file_path.extension().map_or(true, |ext| ext != "md") {
        new_file_path.set_extension("md");
    }

    // Check if target file already exists
    if new_file_path.exists() {
        return Err(ObsidianError::Config(anyhow::anyhow!(
            "Target file already exists: {}", 
            new_file_path.display()
        )));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = new_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Get the old filename without extension for wiki link updates
    let old_name = old_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ObsidianError::Config(anyhow::anyhow!("Invalid old filename")))?;
    
    let new_name_stem = new_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ObsidianError::Config(anyhow::anyhow!("Invalid new filename")))?;

    // Perform the rename operation
    fs::rename(&old_file_path, &new_file_path)?;

    println!("{} Renamed: {} -> {}", 
             "✓".green().bold(), 
             wrap_filename(&old_file_path.display().to_string(), 40),
             wrap_filename(&new_file_path.display().to_string(), 40));

    // Update wiki links if requested
    if update_links {
        update_wiki_links(vault, old_name, new_name_stem).await?;
    }

    Ok(())
}

async fn update_wiki_links(vault: &Vault, old_name: &str, new_name: &str) -> Result<()> {
    println!("{} Searching for wiki links to update...", "🔍".blue().bold());
    
    // Create regex patterns for different wiki link formats
    let patterns = vec![
        // [[old_name]]
        Regex::new(&format!(r"\[\[{}\]\]", regex::escape(old_name)))
            .map_err(|e| ObsidianError::Config(anyhow::anyhow!("Regex error: {}", e)))?,
        // [[old_name|display text]]
        Regex::new(&format!(r"\[\[{}(\|[^\]]*)\]\]", regex::escape(old_name)))
            .map_err(|e| ObsidianError::Config(anyhow::anyhow!("Regex error: {}", e)))?,
        // [[old_name#section]]
        Regex::new(&format!(r"\[\[{}(#[^\]]*)\]\]", regex::escape(old_name)))
            .map_err(|e| ObsidianError::Config(anyhow::anyhow!("Regex error: {}", e)))?,
        // [[old_name#section|display text]]
        Regex::new(&format!(r"\[\[{}(#[^\]]*\|[^\]]*)\]\]", regex::escape(old_name)))
            .map_err(|e| ObsidianError::Config(anyhow::anyhow!("Regex error: {}", e)))?,
    ];

    let mut files_updated = 0;
    let mut total_links_updated = 0;

    // Walk through all markdown files in the vault
    for entry in WalkDir::new(&vault.path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            if let Ok(relative_path) = entry.path().strip_prefix(&vault.path) {
                if !is_path_blacklisted(relative_path, &vault.blacklist) {
                    let file_path = entry.path();
                    
                    // Read file contents
                    let content = fs::read_to_string(file_path)?;
                    
                    let mut updated_content = content.clone();
                    let mut file_links_updated = 0;
                    
                    // Apply each pattern replacement
                    for pattern in &patterns {
                        let new_content = pattern.replace_all(&updated_content, |caps: &regex::Captures| {
                            if let Some(suffix) = caps.get(1) {
                                // Handle cases with additional content (|display, #section, etc.)
                                format!("[[{}{}]]", new_name, suffix.as_str())
                            } else {
                                // Simple [[old_name]] -> [[new_name]]
                                format!("[[{}]]", new_name)
                            }
                        });
                        
                        // Count replacements made by this pattern
                        if new_content != updated_content {
                            let old_count = pattern.find_iter(&updated_content).count();
                            file_links_updated += old_count;
                            updated_content = new_content.to_string();
                        }
                    }
                    
                    // Write back the file if any changes were made
                    if updated_content != content {
                        fs::write(file_path, updated_content)?;
                        
                        files_updated += 1;
                        total_links_updated += file_links_updated;
                        
                        println!("  {} Updated {} link(s) in {}", 
                                "✓".green(), 
                                file_links_updated.to_string().yellow(),
                                wrap_filename(&relative_path.display().to_string(), 40));
                    }
                }
            }
        }
    }

    if files_updated > 0 {
        println!("{} Updated {} wiki link(s) across {} file(s)", 
                "✅".green().bold(),
                total_links_updated.to_string().yellow().bold(),
                files_updated.to_string().yellow().bold());
    } else {
        println!("{} No wiki links found that reference '{}'", "ℹ️".blue(), old_name.yellow());
    }

    Ok(())
}
