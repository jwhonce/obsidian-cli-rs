use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use colored::*;
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

pub async fn execute(state: &State, page_or_path: &Path, force: bool) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(state, page_or_path)?;
    let (frontmatter, _content) = frontmatter::parse_file(&file_path)?;

    // Check if UID already exists
    if frontmatter.contains_key(&state.ident_key) && !force {
        if state.verbose {
            eprintln!(
                "{}",
                format!(
                    "Use --force to replace value of existing {}.",
                    state.ident_key
                )
                .yellow()
            );
        }

        if let Some(existing_value) = frontmatter.get(&state.ident_key) {
            eprintln!(
                "{}",
                format!(
                    "Page '{}' already has {{ '{}': '{}' }}",
                    page_or_path.display(),
                    state.ident_key,
                    existing_value
                )
                .red()
            );
            std::process::exit(1);
        }
    }

    let new_uuid = Uuid::new_v4().to_string();

    if state.verbose {
        println!("Generated new {{ '{}': '{}' }}", state.ident_key, new_uuid);
    }

    // Update frontmatter with the new UUID
    frontmatter::update_frontmatter(&file_path, &state.ident_key, Value::String(new_uuid))?;

    Ok(())
}
