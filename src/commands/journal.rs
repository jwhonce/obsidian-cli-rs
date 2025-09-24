use crate::errors::{ObsidianError, Result};
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::{format_journal_template, get_template_vars, launch_editor};
use chrono::{Local, NaiveDate};
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn execute(vault: &Vault, date: Option<&str>) -> Result<()> {
    let target_date = if let Some(date_str) = date {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            ObsidianError::TemplateFormatting(
                "Invalid date format. Use ISO format YYYY-MM-DD.".to_string(),
            )
        })?;

        let naive_datetime = naive_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            ObsidianError::TemplateFormatting("Failed to construct datetime from date".to_string())
        })?;

        naive_datetime
            .and_local_timezone(Local)
            .single()
            .ok_or_else(|| {
                ObsidianError::TemplateFormatting(
                    "Ambiguous or invalid timezone conversion for date".to_string(),
                )
            })?
    } else {
        Local::now()
    };

    let template_vars = get_template_vars(target_date);
    let journal_path_str =
        format_journal_template(vault.journal_template.as_str(), &template_vars)?;
    let mut page_path = PathBuf::from(journal_path_str);
    page_path.set_extension("md");

    // Convert to full path within vault
    let full_path = vault.path.join(&page_path);

    if vault.verbose {
        println!("Using journal template: {}", vault.journal_template);
        println!("Resolved journal path: {}", page_path.display());
        println!("Full journal path: {}", full_path.display());
    }

    // Create the journal file if it doesn't exist
    if !full_path.exists() {
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let title = page_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Journal Entry");
        let mut frontmatter = HashMap::new();
        frontmatter::add_default_frontmatter(&mut frontmatter, title, vault.ident_key.as_str());

        let content = format!("# {title}\n\n");
        let serialized = frontmatter::serialize_with_frontmatter(&frontmatter, &content)?;
        std::fs::write(&full_path, serialized)?;

        if vault.verbose {
            println!(
                "{} {}: {}",
                "Created".green(),
                "journal file".green(),
                full_path.display()
            );
        }
    }

    // Launch editor
    launch_editor(vault.editor.as_str(), &full_path)?;

    Ok(())
}
