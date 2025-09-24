use crate::errors::Result;
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::launch_editor;
use chrono::Utc;
use serde_json::Value;
use std::path::Path;

pub fn execute(vault: &Vault, page_or_path: &Path) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(vault, page_or_path)?;

    launch_editor(&vault.editor, &file_path)?;

    // Update the modified timestamp in frontmatter
    frontmatter::update_frontmatter(
        &file_path,
        "modified",
        Value::String(Utc::now().to_rfc3339()),
    )?;

    Ok(())
}
