use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObsidianError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Editor execution error: {0}")]
    EditorExecution(String),

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Frontmatter parsing error: {0}")]
    FrontmatterParsing(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template formatting error: {0}")]
    TemplateFormatting(String),
}

pub type Result<T> = std::result::Result<T, ObsidianError>;
