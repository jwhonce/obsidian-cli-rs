use crate::errors::Result;
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::{format_value, parse_value};
use chrono::Utc;
use colored::Colorize;
use std::path::Path;

pub fn execute(
    vault: &Vault,
    page_or_path: &Path,
    key: Option<&str>,
    value: Option<&str>,
) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(vault, page_or_path)?;
    let (frontmatter, _content) = frontmatter::parse_file(&file_path)?;

    match (key, value) {
        // List all frontmatter metadata
        (None, None) => {
            if frontmatter.is_empty() {
                eprintln!("{}", "No frontmatter metadata found for this page".red());
            } else {
                for (k, v) in &frontmatter {
                    println!("{}: {}", k, format_value(v));
                }
            }
        }
        // Display specific key
        (Some(k), None) => {
            if let Some(v) = frontmatter.get(k) {
                println!("{}: {}", k, format_value(v));
            } else {
                eprintln!(
                    "{}",
                    format!(
                        "Frontmatter metadata '{}' not found in '{}'",
                        k,
                        page_or_path.display()
                    )
                    .red()
                );
                std::process::exit(1);
            }
        }
        // Update key with value
        (Some(k), Some(v)) => {
            let new_value = parse_value(v);
            frontmatter::update_frontmatter(&file_path, k, new_value)?;

            if vault.verbose {
                println!(
                    "Updated frontmatter metadata {{ '{}': '{}', 'modified': '{}' }} in {}",
                    k,
                    v,
                    Utc::now().to_rfc3339(),
                    file_path.display()
                );
            }
        }
        // Invalid combination (Some(None), Some(_)) - shouldn't happen with CLI
        (None, Some(_)) => unreachable!("CLI should prevent this case"),
    }

    Ok(())
}
