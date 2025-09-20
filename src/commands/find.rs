use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use crate::utils::find_matching_files;
use colored::*;
use serde_json::Value;

pub async fn execute(state: &State, page_name: &str, exact: bool) -> Result<()> {
    if state.verbose {
        println!("Searching for page: '{}'", page_name);
        println!("Exact match: {}", exact);
    }

    let matches = find_matching_files(&state.vault, page_name, exact)?;

    if matches.is_empty() {
        eprintln!(
            "{}",
            format!("No files found matching '{}'", page_name).yellow()
        );
        return Ok(());
    }
    for path in matches {
        println!("{}", path.display());

        // Show frontmatter title if verbose and it exists
        if state.verbose {
            let full_path = state.vault.join(&path);
            if let Ok((frontmatter, _)) = frontmatter::parse_file(&full_path) {
                if let Some(Value::String(title)) = frontmatter.get("title") {
                    println!("  title: {}", title);
                }
            }
        }
    }

    Ok(())
}
