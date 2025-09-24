use crate::errors::Result;
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::launch_editor;
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;

pub fn execute(vault: &Vault, page_or_path: &Path, force: bool) -> Result<()> {
    let mut path = vault.path.join(page_or_path);
    if path.extension().is_none() {
        path.set_extension("md");
    }

    let is_overwrite = path.exists();
    if is_overwrite && !force {
        eprintln!(
            "{}",
            format!("File already exists: {}", path.display()).red()
        );
        std::process::exit(1);
    }

    if is_overwrite && vault.verbose {
        println!(
            "{}",
            format!("Overwriting existing file: {}", path.display()).yellow()
        );
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let title = page_or_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");

    let mut frontmatter = HashMap::new();

    // Check if content is being piped in
    let content = if atty::isnt(atty::Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if vault.verbose {
            println!("Using content from stdin");
        }
        buffer.trim().to_string()
    } else {
        format!("# {title}\n\n")
    };

    frontmatter::add_default_frontmatter(&mut frontmatter, title, &vault.ident_key);

    let serialized = frontmatter::serialize_with_frontmatter(&frontmatter, &content)?;
    std::fs::write(&path, serialized)?;

    // Open file in editor (if not using stdin input)
    if atty::is(atty::Stream::Stdin) {
        launch_editor(&vault.editor, &path)?;
    }

    if vault.verbose {
        let action = if is_overwrite {
            "Overwritten"
        } else {
            "Created"
        };
        println!("{} {}: {}", action.green(), "file".green(), path.display());
    }

    Ok(())
}
