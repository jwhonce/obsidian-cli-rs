use crate::errors::Result;
use crate::types::Vault;
use crate::utils::{get_file_dates, is_path_blacklisted, wrap_filename};
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment,
    ContentArrangement, Table,
};
use std::fmt::Write as FmtWrite;
use walkdir::WalkDir;

pub fn execute(vault: &Vault, show_dates: bool) -> Result<()> {
    let output = render_ls_output(vault, show_dates);
    print!("{}", output);
    Ok(())
}

pub fn render_ls_output(vault: &Vault, show_dates: bool) -> String {
    let mut files = Vec::new();

    for entry in WalkDir::new(&vault.path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
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

    let mut buffer = String::new();

    if show_dates {
        if files.is_empty() {
            let _ = writeln!(buffer, "{}", "No markdown files found in vault".yellow());
            return buffer;
        }

        let _ = writeln!(buffer, "{}", "Vault Files with Dates".bold().blue());
        buffer.push('\n');

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

            let file_display = file.display().to_string();
            let wrapped_filename = wrap_filename(&file_display, 40);
            table.add_row(vec![
                Cell::new(wrapped_filename.as_ref()),
                Cell::new(created).set_alignment(CellAlignment::Right),
                Cell::new(modified).set_alignment(CellAlignment::Right),
            ]);
        }

        let _ = writeln!(buffer, "{table}");
    } else {
        for file in files {
            let _ = writeln!(buffer, "{}", file.display());
        }
    }

    buffer
}
