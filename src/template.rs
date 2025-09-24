use crate::errors::{ObsidianError, Result};
use chrono::{DateTime, Datelike, Utc};
use regex::Regex;
use std::collections::HashMap;

/// A flexible template engine that mimics Python's string formatting capabilities.
///
/// This engine supports:
/// - Simple variables: `{year}`, `{month}`, `{day}`
/// - Format specifiers: `{month:02d}`, `{day:02d}` (zero-padding)
/// - Date formatting: `{month_name}`, `{weekday}`, etc.
/// - Custom variables can be easily added
///
/// Examples:
/// - `"Calendar/{year}/{month:02d}/{day:02d}"` → `"Calendar/2025/01/15"`
/// - `"Notes/{year}-{month_name}/{weekday}"` → `"Notes/2025-January/Wednesday"`
#[derive(Debug)]
pub struct TemplateEngine {
    variables: HashMap<String, TemplateVariable>,
}

/// Represents a template variable with its value and formatting capabilities
#[derive(Debug, Clone)]
enum TemplateVariable {
    Integer(i32),
    String(String),
}

impl TemplateVariable {
    /// Format the variable according to the given format specifier
    fn format(&self, spec: Option<&str>) -> Result<String> {
        match self {
            TemplateVariable::Integer(value) => {
                if let Some(format_spec) = spec {
                    Self::format_integer(*value, format_spec)
                } else {
                    Ok(value.to_string())
                }
            }
            TemplateVariable::String(value) => {
                if spec.is_some() {
                    // For now, strings don't support format specifiers
                    // but this could be extended for alignment, truncation, etc.
                    Ok(value.clone())
                } else {
                    Ok(value.clone())
                }
            }
        }
    }

    /// Format an integer value according to format specifier
    fn format_integer(value: i32, spec: &str) -> Result<String> {
        if spec == "02d" || spec == "02" {
            // Zero-pad to 2 digits
            Ok(format!("{:02}", value))
        } else if spec == "03d" {
            // Zero-pad to 3 digits
            Ok(format!("{:03}", value))
        } else if spec == "04d" {
            // Zero-pad to 4 digits
            Ok(format!("{:04}", value))
        } else if spec.starts_with('0') && spec.ends_with('d') {
            // Generic zero-padding: {variable:0Nd}
            let width_str = &spec[1..spec.len() - 1];
            if let Ok(width) = width_str.parse::<usize>() {
                Ok(format!("{:0width$}", value, width = width))
            } else {
                Err(ObsidianError::TemplateFormatting(format!(
                    "Invalid format specifier: {}",
                    spec
                )))
            }
        } else if spec == "d" {
            // Plain decimal
            Ok(value.to_string())
        } else {
            Err(ObsidianError::TemplateFormatting(format!(
                "Unsupported format specifier for integer: {}",
                spec
            )))
        }
    }
}

impl TemplateEngine {
    /// Create a new template engine with date variables for the given date
    pub fn new(date: DateTime<Utc>) -> Self {
        let mut variables = HashMap::new();

        // Basic date components
        variables.insert("year".to_string(), TemplateVariable::Integer(date.year()));
        variables.insert(
            "month".to_string(),
            TemplateVariable::Integer(date.month() as i32),
        );
        variables.insert(
            "day".to_string(),
            TemplateVariable::Integer(date.day() as i32),
        );

        // Formatted date strings
        variables.insert(
            "month_name".to_string(),
            TemplateVariable::String(date.format("%B").to_string()),
        );
        variables.insert(
            "month_abbr".to_string(),
            TemplateVariable::String(date.format("%b").to_string()),
        );
        variables.insert(
            "weekday".to_string(),
            TemplateVariable::String(date.format("%A").to_string()),
        );
        variables.insert(
            "weekday_abbr".to_string(),
            TemplateVariable::String(date.format("%a").to_string()),
        );

        Self { variables }
    }

    /// Add a custom string variable to the template engine
    #[allow(dead_code)]
    pub fn add_string(&mut self, name: String, value: String) {
        self.variables.insert(name, TemplateVariable::String(value));
    }

    /// Add a custom integer variable to the template engine  
    #[allow(dead_code)]
    pub fn add_integer(&mut self, name: String, value: i32) {
        self.variables
            .insert(name, TemplateVariable::Integer(value));
    }

    /// Format a template string with the available variables
    ///
    /// Supports format specifiers like:
    /// - `{year}` → "2025"
    /// - `{month:02d}` → "01" (zero-padded)
    /// - `{month_name}` → "January"
    ///
    /// # Arguments
    /// * `template` - Template string with variable placeholders
    ///
    /// # Returns
    /// * `Result<String>` - Formatted string or error if template is invalid
    pub fn format(&self, template: &str) -> Result<String> {
        // Regex to match {variable} and {variable:format} patterns
        let re = Regex::new(r"\{([^}:]+)(?::([^}]+))?\}")
            .map_err(|e| ObsidianError::TemplateFormatting(e.to_string()))?;

        let mut result = template.to_string();
        let mut offset = 0i32;

        // Process all matches and replace them
        for captures in re.captures_iter(template) {
            let full_match = captures.get(0).ok_or_else(|| {
                ObsidianError::TemplateFormatting("Regex match missing full capture".to_string())
            })?;
            let var_name = captures.get(1).ok_or_else(|| {
                ObsidianError::TemplateFormatting("Regex match missing variable name".to_string())
            })?.as_str();
            let format_spec = captures.get(2).map(|m| m.as_str());

            // Look up the variable
            let variable = self.variables.get(var_name).ok_or_else(|| {
                ObsidianError::TemplateFormatting(format!(
                    "Unknown template variable: {}",
                    var_name
                ))
            })?;

            // Format the variable
            let formatted_value = variable.format(format_spec)?;

            // Calculate positions adjusted for previous replacements
            let start = (full_match.start() as i32 + offset) as usize;
            let end = (full_match.end() as i32 + offset) as usize;

            // Replace the placeholder with the formatted value
            result.replace_range(start..end, &formatted_value);

            // Update offset for next replacement
            offset += formatted_value.len() as i32 - full_match.len() as i32;
        }

        Ok(result)
    }

    /// List all available variables in the template engine
    #[allow(dead_code)]
    pub fn available_variables(&self) -> Vec<&String> {
        self.variables.keys().collect()
    }
}

/// Format a journal template using TemplateVars (for backward compatibility)
pub fn format_journal_template_with_vars(
    template: &str,
    vars: &crate::types::TemplateVars,
) -> Result<String> {
    use chrono::{TimeZone, Utc};

    // Convert TemplateVars to a DateTime for the template engine
    let date = Utc
        .with_ymd_and_hms(vars.year, vars.month, vars.day, 0, 0, 0)
        .single()
        .ok_or_else(|| {
            crate::errors::ObsidianError::TemplateFormatting(format!(
                "Invalid date: {}-{:02}-{:02}",
                vars.year, vars.month, vars.day
            ))
        })?;

    let engine = TemplateEngine::new(date);
    engine.format(template)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_basic_template() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine.format("{year}/{month}/{day}").unwrap();
        assert_eq!(result, "2025/1/15");
    }

    #[test]
    fn test_zero_padding() {
        let date = Utc.with_ymd_and_hms(2025, 1, 5, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine.format("{year}/{month:02d}/{day:02d}").unwrap();
        assert_eq!(result, "2025/01/05");
    }

    #[test]
    fn test_string_variables() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine.format("{year}-{month_name}-{weekday}").unwrap();
        assert_eq!(result, "2025-January-Wednesday");
    }

    #[test]
    fn test_custom_variables() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let mut engine = TemplateEngine::new(date);

        engine.add_string("project".to_string(), "MyProject".to_string());
        engine.add_integer("version".to_string(), 42);

        let result = engine.format("{project}/v{version:03d}/{year}").unwrap();
        assert_eq!(result, "MyProject/v042/2025");
    }

    #[test]
    fn test_complex_template() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine
            .format("Calendar/{year}/{month:02d}/{year}-{month:02d}-{day:02d}")
            .unwrap();
        assert_eq!(result, "Calendar/2025/01/2025-01-15");
    }

    #[test]
    fn test_unknown_variable() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine.format("{unknown_var}");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown template variable: unknown_var"));
    }

    #[test]
    fn test_invalid_format_spec() {
        let date = Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);

        let result = engine.format("{month:invalid}");
        assert!(result.is_err());
    }
}
