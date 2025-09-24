use crate::errors::Result;
use crate::frontmatter;
use crate::types::Vault;
use colored::Colorize;
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

pub fn execute(vault: &Vault, page_or_path: &Path, force: bool) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(vault, page_or_path)?;
    let (frontmatter, _content) = frontmatter::parse_file(&file_path)?;

    // Check if UID already exists
    if frontmatter.contains_key(&vault.ident_key) && !force {
        if vault.verbose {
            eprintln!(
                "{}",
                format!(
                    "Use --force to replace value of existing {}.",
                    vault.ident_key
                )
                .yellow()
            );
        }

        if let Some(existing_value) = frontmatter.get(&vault.ident_key) {
            eprintln!(
                "{}",
                format!(
                    "Page '{}' already has {{ '{}': '{}' }}",
                    page_or_path.display(),
                    vault.ident_key,
                    existing_value
                )
                .red()
            );
            std::process::exit(1);
        }
    }

    let new_uuid = Uuid::new_v4().to_string();

    if vault.verbose {
        println!("Generated new {{ '{}': '{}' }}", vault.ident_key, new_uuid);
    }

    // Update frontmatter with the new UUID
    frontmatter::update_frontmatter(&file_path, &vault.ident_key, Value::String(new_uuid))?;

    Ok(())
}
