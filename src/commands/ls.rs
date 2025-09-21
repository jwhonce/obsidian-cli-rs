use crate::errors::Result;
use crate::types::Vault;
use crate::utils::{is_path_blacklisted, wrap_filename, get_file_dates};
use colored::*;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment,
    ContentArrangement, Table,
};
use walkdir::WalkDir;

pub async fn execute(vault: &Vault, show_dates: bool) -> Result<()> {
    let mut files = Vec::new();

    for entry in WalkDir::new(&vault.path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            if let Ok(relative_path) = entry.path().strip_prefix(&vault.path) {
                if !is_path_blacklisted(relative_path, &vault.blacklist) {
                    files.push(relative_path.to_path_buf());
                }
            }
        }
    }

    files.sort();

    if show_dates {
        if files.is_empty() {
            println!("{}", "No markdown files found in vault".yellow());
            return Ok(());
        }

        println!("{}", "Vault Files with Dates".bold().blue());
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("File").add_attribute(Attribute::Bold),
                Cell::new("Created")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new("Modified")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
            ]);

        for file in files {
            let full_path = vault.path.join(&file);
            let (created, modified) = get_file_dates(&full_path);
            
            let wrapped_filename = wrap_filename(&file.display().to_string(), 40);
            table.add_row(vec![
                Cell::new(wrapped_filename),
                Cell::new(created).set_alignment(CellAlignment::Right),
                Cell::new(modified).set_alignment(CellAlignment::Right),
            ]);
        }

        println!("{}", table);
    } else {
        for file in files {
            println!("{}", file.display());
        }
    }

    Ok(())
}


