use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObsidianError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Editor execution errors
    #[error("Editor execution error: {0}")]
    EditorExecution(String),

    /// File system errors
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File already exists: {path}")]
    FileExists { path: String },

    /// Frontmatter parsing and processing errors
    #[error("Frontmatter parsing error: {0}")]
    FrontmatterParsing(String),

    #[error("Frontmatter key '{key}' not found in '{file}'")]
    FrontmatterKeyNotFound { key: String, file: String },

    #[error("Page '{file}' already has {key}: {value}")]
    FrontmatterKeyExists {
        key: String,
        value: String,
        file: String,
    },

    /// Argument validation errors
    #[error("Invalid arguments: {message}")]
    InvalidArguments { message: String },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Template processing errors
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    /// Legacy template formatting error for backward compatibility
    #[error("Template formatting error: {0}")]
    TemplateFormatting(String),

    /// Vault validation errors
    #[error("Vault error: {0}")]
    Vault(#[from] VaultError),
}

/// Configuration-specific errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {path}")]
    NotFound { path: String },

    #[error("Invalid TOML syntax in config file: {0}")]
    InvalidToml(#[from] toml::de::Error),

    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },

    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to expand path: {path}")]
    PathExpansion { path: String },
}

/// Template processing errors
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Invalid format specifier: {spec}")]
    InvalidFormatSpecifier { spec: String },

    #[error("Variable not found: {var}")]
    VariableNotFound { var: String },

    #[error("Invalid template syntax: {message}")]
    InvalidSyntax { message: String },

    #[error("Date/time conversion error: {message}")]
    DateTimeConversion { message: String },
}

/// Vault-specific errors
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault directory does not exist: {path}")]
    NotFound { path: String },

    #[error("Invalid Obsidian vault: missing .obsidian directory in {path}")]
    InvalidVault { path: String },

    #[error("Vault path must be a directory: {path}")]
    NotDirectory { path: String },

    #[error("Cannot access vault directory: {path}")]
    AccessDenied { path: String },
}

pub type Result<T> = std::result::Result<T, ObsidianError>;
