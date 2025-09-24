use crate::errors::Result;
use crate::types::{FileTypeStat, Vault, VaultInfo};
use crate::utils::get_vault_info;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment,
    ContentArrangement, Table,
};
use humansize::{format_size, DECIMAL};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

pub fn execute(vault: &Vault) -> Result<()> {
    let vault_info = get_vault_info(vault)?;
    let output = render_info_output(&vault_info);
    print!("{}", output);

    Ok(())
}

pub fn render_info_output(vault_info: &VaultInfo) -> String {
    let mut buffer = String::new();

    let _ = writeln!(buffer, "{}", "OBSIDIAN VAULT INFORMATION".bold().blue());
    buffer.push('\n');

    let summary_table = build_summary_table(vault_info);
    let _ = writeln!(buffer, "{summary_table}\n");

    if vault_info.file_type_stats.is_empty() {
        let _ = writeln!(buffer, "{}", "No files found in vault".yellow());
        buffer.push('\n');
    } else {
        let _ = writeln!(buffer, "{}", "File Types by Extension".bold().italic());
        let file_table = build_file_type_table(
            vault_info.total_files,
            &vault_info.file_type_stats,
            vault_info.usage_files,
        );
        let _ = writeln!(buffer, "{file_table}\n");
    }

    let config_table = build_config_table(vault_info);
    let _ = writeln!(buffer, "{config_table}");

    buffer
}

fn build_summary_table(vault_info: &VaultInfo) -> Table {
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

    summary_table
}

fn build_file_type_table(
    total_files: usize,
    stats: &HashMap<String, FileTypeStat>,
    usage_files: u64,
) -> Table {
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

    let mut sorted_stats: Vec<(&String, &FileTypeStat)> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| a.0.cmp(b.0));

    for (ext, stat) in sorted_stats {
        let ext_display = if ext.as_str() == "(no extension)" {
            "(no extension)"
        } else {
            ext.as_str()
        };

        let size_str = format_size(stat.total_size, DECIMAL);
        let percentage = if total_files > 0 {
            format!("{:.1}%", (stat.count as f64 / total_files as f64) * 100.0)
        } else {
            "0.0%".to_string()
        };

        file_table.add_row(vec![
            Cell::new(ext_display),
            Cell::new(stat.count.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(size_str).set_alignment(CellAlignment::Right),
            Cell::new(percentage).set_alignment(CellAlignment::Right),
        ]);
    }

    file_table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(total_files.to_string())
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        Cell::new(format_size(usage_files, DECIMAL))
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        Cell::new("100.0%")
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
    ]);

    file_table
}

fn build_config_table(vault_info: &VaultInfo) -> Table {
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
            Cell::new(
                vault_info
                    .blacklist
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(":"),
            )
            .set_alignment(CellAlignment::Right),
        ])
        .add_row(vec![
            Cell::new("Editor"),
            Cell::new(vault_info.editor.as_str()).set_alignment(CellAlignment::Right),
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

    config_table
}
