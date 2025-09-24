use crate::errors::Result;
use crate::types::Vault;
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;

pub fn execute(vault: &Vault, page_or_path: &Path, force: bool) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(vault, page_or_path)?;

    if !force {
        print!(
            "Are you sure you want to delete '{}'? [y/N]: ",
            file_path.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    std::fs::remove_file(&file_path)?;

    if vault.verbose {
        println!("{}: {}", "File removed".green(), file_path.display());
    }

    Ok(())
}
