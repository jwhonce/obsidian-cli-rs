use crate::errors::Result;
use crate::frontmatter;
use crate::types::State;
use std::path::Path;

pub async fn execute(state: &State, page_or_path: &Path, show_frontmatter: bool) -> Result<()> {
    let file_path = crate::resolve_page_or_path!(state, page_or_path)?;

    if show_frontmatter {
        // Simply read and display the entire file
        let content = std::fs::read_to_string(&file_path)?;
        print!("{}", content);
    } else {
        // Parse with frontmatter and only display the content/body
        let (_, content) = frontmatter::parse_file(&file_path)?;
        print!("{}", content);
    }

    Ok(())
}
