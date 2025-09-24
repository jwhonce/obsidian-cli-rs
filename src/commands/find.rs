use crate::errors::Result;
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::find_matching_files;
use colored::Colorize;
use serde_json::Value;

pub fn execute(vault: &Vault, page_name: &str, exact: bool) -> Result<()> {
    if vault.verbose {
        println!("Searching for page: '{page_name}'");
        println!("Exact match: {exact}");
    }

    let matches = find_matching_files(&vault.path, page_name, exact)?;

    if matches.is_empty() {
        eprintln!(
            "{}",
            format!("No files found matching '{page_name}'").yellow()
        );
        return Ok(());
    }
    for path in matches {
        println!("{}", path.display());

        // Show frontmatter title if verbose and it exists
        if vault.verbose {
            let full_path = vault.path.join(&path);
            if let Ok((frontmatter, _)) = frontmatter::parse_file(&full_path) {
                if let Some(Value::String(title)) = frontmatter.get("title") {
                    println!("  title: {title}");
                }
            }
        }
    }

    Ok(())
}
