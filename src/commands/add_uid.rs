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
    if frontmatter.contains_key(vault.ident_key.as_str()) && !force {
        if vault.verbose {
            eprintln!(
                "{}",
                format!(
                    "Use --force to replace value of existing {}.",
                    vault.ident_key.as_str()
                )
                .yellow()
            );
        }

        if let Some(existing_value) = frontmatter.get(vault.ident_key.as_str()) {
            return Err(crate::errors::ObsidianError::FrontmatterKeyExists {
                key: vault.ident_key.as_str().to_string(),
                value: format!("{existing_value}"),
                file: format!("{}", page_or_path.display()),
            });
        }
    }

    let new_uuid = Uuid::new_v4().to_string();

    if vault.verbose {
        println!(
            "Generated new {{ '{}': '{}' }}",
            vault.ident_key.as_str(),
            new_uuid
        );
    }

    // Update frontmatter with the new UUID
    frontmatter::update_frontmatter(
        &file_path,
        vault.ident_key.as_str(),
        Value::String(new_uuid),
    )?;

    Ok(())
}
