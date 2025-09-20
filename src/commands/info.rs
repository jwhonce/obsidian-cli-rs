use crate::errors::Result;
use crate::types::State;
use crate::utils::get_vault_info;
use colored::*;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment,
    ContentArrangement, Table,
};
use humansize::{format_size, DECIMAL};

pub async fn execute(state: &State) -> Result<()> {
    let vault_info = get_vault_info(state)?;

    println!("{}", "OBSIDIAN VAULT INFORMATION".bold().blue());
    println!();

    // Vault Summary Table
    let mut summary_table = Table::new();
    summary_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Property").add_attribute(Attribute::Bold),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Right),
        ]);

    summary_table
        .add_row(vec![
            Cell::new("Path"),
            Cell::new(vault_info.vault_path.to_string_lossy()).set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Total Directories"),
            Cell::new(vault_info.total_directories.to_string()).set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Total Files"),
            Cell::new(vault_info.total_files.to_string()).set_alignment(CellAlignment::Right),
        ]);

    println!("{}", summary_table);
    println!();

    // File Types Table
    if !vault_info.file_type_stats.is_empty() {
        println!("{}", "File Types by Extension".bold().italic());

        let mut file_table = Table::new();
        file_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Extension").add_attribute(Attribute::Bold),
                Cell::new("Count")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new("Size")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new("Percentage")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
            ]);

        let total_files = vault_info.total_files;
        let mut sorted_stats: Vec<_> = vault_info.file_type_stats.iter().collect();
        sorted_stats.sort_by_key(|(ext, _)| ext.as_str());

        for (ext, stats) in sorted_stats {
            let ext_display = if ext == "(no extension)" {
                "(no extension)"
            } else {
                ext
            };
            let size_str = format_size(stats.total_size, DECIMAL);
            let percentage = if total_files > 0 {
                format!("{:.1}%", (stats.count as f64 / total_files as f64) * 100.0)
            } else {
                "0.0%".to_string()
            };

            file_table.add_row(vec![
                Cell::new(ext_display),
                Cell::new(stats.count.to_string()).set_alignment(CellAlignment::Right),
                Cell::new(size_str).set_alignment(CellAlignment::Right),
                Cell::new(percentage).set_alignment(CellAlignment::Right),
            ]);
        }

        // Add totals row
        file_table.add_row(vec![
            Cell::new("TOTAL").add_attribute(Attribute::Bold),
            Cell::new(vault_info.total_files.to_string())
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Right),
            Cell::new(format_size(vault_info.usage_files, DECIMAL))
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Right),
            Cell::new("100.0%")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Right),
        ]);

        println!("{}", file_table);
        println!();
    } else {
        println!("{}", "No files found in vault".yellow());
        println!();
    }

    // Configuration Table
    let mut config_table = Table::new();
    config_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Setting").add_attribute(Attribute::Bold),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Right),
        ]);

    config_table
        .add_row(vec![
            Cell::new("Vault Blacklist"),
            Cell::new(vault_info.blacklist.join(":")).set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Editor"),
            Cell::new(&vault_info.editor).set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Journal Template"),
            Cell::new(format!(
                "{} => {}",
                vault_info.journal_template, vault_info.journal_path
            ))
            .set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Verbose"),
            Cell::new(if vault_info.verbose { "Yes" } else { "No" })
                .set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Version"),
            Cell::new(&vault_info.version).set_alignment(CellAlignment::Right),
        ]);

    println!("{}", config_table);

    Ok(())
}
