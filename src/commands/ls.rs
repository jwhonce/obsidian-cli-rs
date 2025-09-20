use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use crate::utils::is_path_blacklisted;
use walkdir::WalkDir;

pub async fn execute(state: &State, show_dates: bool) -> Result<()> {
    let mut files = Vec::new();

    for entry in WalkDir::new(&state.vault)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            if let Ok(relative_path) = entry.path().strip_prefix(&state.vault) {
                if !is_path_blacklisted(relative_path, &state.blacklist) {
                    files.push(relative_path.to_path_buf());
                }
            }
        }
    }

    files.sort();

    if show_dates {
        for file in files {
            let full_path = state.vault.join(&file);
            let (created, modified) = get_file_dates(&full_path);
            println!(
                "{:<40} created: {} modified: {}",
                file.display(),
                created,
                modified
            );
        }
    } else {
        for file in files {
            println!("{}", file.display());
        }
    }

    Ok(())
}

/// Extract created and modified dates from frontmatter or filesystem
fn get_file_dates(file_path: &std::path::Path) -> (String, String) {
    // Try to get dates from frontmatter first
    if let Ok((frontmatter, _)) = frontmatter::parse_file(file_path) {
        let created = extract_date_from_frontmatter(&frontmatter, "created")
            .unwrap_or_else(|| get_filesystem_created_date(file_path));

        let modified = extract_date_from_frontmatter(&frontmatter, "modified")
            .unwrap_or_else(|| get_filesystem_modified_date(file_path));

        (created, modified)
    } else {
        // Fallback to filesystem dates
        (
            get_filesystem_created_date(file_path),
            get_filesystem_modified_date(file_path),
        )
    }
}

/// Extract date from frontmatter field and format as YYYY-MM-DD
fn extract_date_from_frontmatter(
    frontmatter: &std::collections::HashMap<String, serde_json::Value>,
    field: &str,
) -> Option<String> {
    frontmatter.get(field).and_then(|value| {
        match value {
            serde_json::Value::String(date_str) => {
                // Try to parse ISO 8601 format (RFC3339)
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(date_str) {
                    Some(datetime.format("%Y-%m-%d").to_string())
                } else if let Ok(naive_date) =
                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                {
                    // Already in YYYY-MM-DD format
                    Some(naive_date.format("%Y-%m-%d").to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

/// Get filesystem created date formatted as YYYY-MM-DD
fn get_filesystem_created_date(file_path: &std::path::Path) -> String {
    std::fs::metadata(file_path)
        .and_then(|metadata| metadata.created())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get filesystem modified date formatted as YYYY-MM-DD
fn get_filesystem_modified_date(file_path: &std::path::Path) -> String {
    std::fs::metadata(file_path)
        .and_then(|metadata| metadata.modified())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}
