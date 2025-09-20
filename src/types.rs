use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeStat {
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub enum OutputStyle {
    Path,
    Title,
    Table,
    Json,
}

impl From<&str> for OutputStyle {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "title" => OutputStyle::Title,
            "table" => OutputStyle::Table,
            "json" => OutputStyle::Json,
            _ => OutputStyle::Path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub path: PathBuf,
    pub frontmatter: HashMap<String, Value>,
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub vault: PathBuf,
    pub blacklist: Vec<String>,
    pub editor: String,
    pub ident_key: String,
    pub journal_template: String,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVars {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub month_name: String,
    pub month_abbr: String,
    pub weekday: String,
    pub weekday_abbr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub blacklist: Vec<String>,
    pub editor: String,
    pub file_type_stats: HashMap<String, FileTypeStat>,
    pub journal_path: String,
    pub journal_template: String,
    pub markdown_files: usize,
    pub total_directories: usize,
    pub total_files: usize,
    pub usage_directories: u64,
    pub usage_files: u64,
    pub vault_path: PathBuf,
    pub verbose: bool,
    pub version: String,
}
