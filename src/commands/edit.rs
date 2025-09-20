use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use crate::utils::launch_editor;
use chrono::Utc;
use serde_json::Value;
use std::path::Path;

pub async fn execute(state: &State, page_or_path: &Path) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(state, page_or_path)?;

    launch_editor(&state.editor, &file_path)?;

    // Update the modified timestamp in frontmatter
    frontmatter::update_frontmatter(
        &file_path,
        "modified",
        Value::String(Utc::now().to_rfc3339()),
    )?;

    Ok(())
}
