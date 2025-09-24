use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

//=============================================================================
// Newtype Wrappers for Type Safety
//=============================================================================

/// Wrapper for identifier keys used in frontmatter (e.g., "uid", "id")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentKey(String);

impl IdentKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for IdentKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for IdentKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for IdentKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Wrapper for journal template strings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalTemplate(String);

impl JournalTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self(template.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JournalTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for JournalTemplate {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for JournalTemplate {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for JournalTemplate {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Wrapper for editor command strings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCommand(String);

impl EditorCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self(command.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EditorCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EditorCommand {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EditorCommand {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for EditorCommand {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for EditorCommand {
    fn default() -> Self {
        Self("vi".to_string())
    }
}

/// Wrapper for blacklist pattern strings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlacklistPattern(String);

impl BlacklistPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn contains(&self, ch: char) -> bool {
        self.0.contains(ch)
    }
}

impl fmt::Display for BlacklistPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for BlacklistPattern {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for BlacklistPattern {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for BlacklistPattern {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

//=============================================================================
// Core Data Structures
//=============================================================================

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

/// Represents an Obsidian vault with its configuration and metadata.
///
/// This struct contains all the necessary information to operate on an Obsidian vault,
/// including the vault path, configuration settings, and operational parameters.
#[derive(Debug, Clone)]
pub struct Vault {
    /// Path to the Obsidian vault directory
    pub path: PathBuf,
    /// List of directory patterns to exclude from operations
    pub blacklist: Vec<BlacklistPattern>,
    /// Editor command to use for editing files
    pub editor: EditorCommand,
    /// Key used for unique identifiers in frontmatter
    pub ident_key: IdentKey,
    /// Template string for journal file paths
    pub journal_template: JournalTemplate,
    /// Whether to enable verbose output
    pub verbose: bool,
}

/// Builder for constructing Vault instances with fluent API
#[derive(Debug, Default)]
pub struct VaultBuilder {
    path: Option<PathBuf>,
    blacklist: Vec<BlacklistPattern>,
    editor: Option<EditorCommand>,
    ident_key: Option<IdentKey>,
    journal_template: Option<JournalTemplate>,
    verbose: bool,
}

impl VaultBuilder {
    /// Create a new VaultBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the vault path
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add a blacklist pattern
    pub fn blacklist_pattern(mut self, pattern: impl Into<BlacklistPattern>) -> Self {
        self.blacklist.push(pattern.into());
        self
    }

    /// Set multiple blacklist patterns
    pub fn blacklist_patterns(
        mut self,
        patterns: impl IntoIterator<Item = impl Into<BlacklistPattern>>,
    ) -> Self {
        self.blacklist.extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Set the editor command
    pub fn editor(mut self, editor: impl Into<EditorCommand>) -> Self {
        self.editor = Some(editor.into());
        self
    }

    /// Set the identifier key
    pub fn ident_key(mut self, key: impl Into<IdentKey>) -> Self {
        self.ident_key = Some(key.into());
        self
    }

    /// Set the journal template
    pub fn journal_template(mut self, template: impl Into<JournalTemplate>) -> Self {
        self.journal_template = Some(template.into());
        self
    }

    /// Enable or disable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Build the Vault instance
    pub fn build(self) -> Result<Vault, &'static str> {
        let path = self.path.ok_or("Vault path is required")?;
        let editor = self.editor.unwrap_or_else(|| EditorCommand::default());
        let ident_key = self.ident_key.unwrap_or_else(|| IdentKey::from("uid"));
        let journal_template = self.journal_template.unwrap_or_else(|| {
            JournalTemplate::from("Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}")
        });

        Ok(Vault {
            path,
            blacklist: self.blacklist,
            editor,
            ident_key,
            journal_template,
            verbose: self.verbose,
        })
    }
}

impl Vault {
    /// Create a new VaultBuilder
    pub fn builder() -> VaultBuilder {
        VaultBuilder::new()
    }
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

/// Builder for constructing TemplateVars
#[derive(Debug)]
pub struct TemplateVarsBuilder {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    month_name: Option<String>,
    month_abbr: Option<String>,
    weekday: Option<String>,
    weekday_abbr: Option<String>,
}

impl TemplateVarsBuilder {
    /// Create a new TemplateVarsBuilder
    pub fn new() -> Self {
        Self {
            year: None,
            month: None,
            day: None,
            month_name: None,
            month_abbr: None,
            weekday: None,
            weekday_abbr: None,
        }
    }

    /// Set the year
    pub fn year(mut self, year: i32) -> Self {
        self.year = Some(year);
        self
    }

    /// Set the month
    pub fn month(mut self, month: u32) -> Self {
        self.month = Some(month);
        self
    }

    /// Set the day
    pub fn day(mut self, day: u32) -> Self {
        self.day = Some(day);
        self
    }

    /// Set the month name
    pub fn month_name(mut self, name: impl Into<String>) -> Self {
        self.month_name = Some(name.into());
        self
    }

    /// Set the month abbreviation
    pub fn month_abbr(mut self, abbr: impl Into<String>) -> Self {
        self.month_abbr = Some(abbr.into());
        self
    }

    /// Set the weekday name
    pub fn weekday(mut self, weekday: impl Into<String>) -> Self {
        self.weekday = Some(weekday.into());
        self
    }

    /// Set the weekday abbreviation
    pub fn weekday_abbr(mut self, abbr: impl Into<String>) -> Self {
        self.weekday_abbr = Some(abbr.into());
        self
    }

    /// Build from a DateTime-like object (requires all fields)
    pub fn from_chrono_datetime<Tz>(mut self, dt: &chrono::DateTime<Tz>) -> Self
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        use chrono::Datelike;

        self.year = Some(dt.year());
        self.month = Some(dt.month());
        self.day = Some(dt.day());
        self.month_name = Some(dt.format("%B").to_string());
        self.month_abbr = Some(dt.format("%b").to_string());
        self.weekday = Some(dt.format("%A").to_string());
        self.weekday_abbr = Some(dt.format("%a").to_string());
        self
    }

    /// Build the TemplateVars instance
    pub fn build(self) -> Result<TemplateVars, &'static str> {
        Ok(TemplateVars {
            year: self.year.ok_or("Year is required")?,
            month: self.month.ok_or("Month is required")?,
            day: self.day.ok_or("Day is required")?,
            month_name: self.month_name.ok_or("Month name is required")?,
            month_abbr: self.month_abbr.ok_or("Month abbreviation is required")?,
            weekday: self.weekday.ok_or("Weekday is required")?,
            weekday_abbr: self
                .weekday_abbr
                .ok_or("Weekday abbreviation is required")?,
        })
    }
}

impl Default for TemplateVarsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateVars {
    /// Create a new TemplateVarsBuilder
    pub fn builder() -> TemplateVarsBuilder {
        TemplateVarsBuilder::new()
    }
}

/// Information about an Obsidian vault including statistics and configuration.
///
/// This struct is used to provide comprehensive information about a vault,
/// including file counts, sizes, and configuration details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub blacklist: Vec<BlacklistPattern>,
    pub editor: EditorCommand,
    pub file_type_stats: HashMap<String, FileTypeStat>,
    pub journal_path: String,
    pub journal_template: JournalTemplate,
    pub markdown_files: usize,
    pub total_directories: usize,
    pub total_files: usize,
    pub usage_directories: u64,
    pub usage_files: u64,
    pub vault_path: PathBuf,
    pub verbose: bool,
    pub version: String,
}
