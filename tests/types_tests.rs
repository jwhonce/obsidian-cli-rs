//! Tests for type-safe wrapper types
//! Tests for IdentKey, JournalTemplate, EditorCommand, and BlacklistPattern

use obsidian_cli::types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident_key_creation_and_access() {
        let key = IdentKey::new("uid");
        assert_eq!(key.as_str(), "uid");

        let key_from_string = IdentKey::from("test_key".to_string());
        assert_eq!(key_from_string.as_str(), "test_key");

        let key_from_str = IdentKey::from("another_key");
        assert_eq!(key_from_str.as_str(), "another_key");
    }

    #[test]
    fn test_ident_key_display() {
        let key = IdentKey::new("display_test");
        assert_eq!(format!("{}", key), "display_test");
    }

    #[test]
    fn test_ident_key_equality() {
        let key1 = IdentKey::new("same");
        let key2 = IdentKey::from("same");
        let key3 = IdentKey::new("different");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_journal_template_creation_and_access() {
        let template = JournalTemplate::new("Calendar/{year}/{month:02}");
        assert_eq!(template.as_str(), "Calendar/{year}/{month:02}");

        let template_from_string = JournalTemplate::from("Notes/{day:02}".to_string());
        assert_eq!(template_from_string.as_str(), "Notes/{day:02}");

        let template_from_str = JournalTemplate::from("Daily/{weekday}");
        assert_eq!(template_from_str.as_str(), "Daily/{weekday}");
    }

    #[test]
    fn test_journal_template_display() {
        let template = JournalTemplate::new("{year}-{month}-{day}");
        assert_eq!(format!("{}", template), "{year}-{month}-{day}");
    }

    #[test]
    fn test_journal_template_equality() {
        let template1 = JournalTemplate::new("same/pattern");
        let template2 = JournalTemplate::from("same/pattern");
        let template3 = JournalTemplate::new("different/pattern");

        assert_eq!(template1, template2);
        assert_ne!(template1, template3);
    }

    #[test]
    fn test_editor_command_creation_and_access() {
        let editor = EditorCommand::new("vim");
        assert_eq!(editor.as_str(), "vim");

        let editor_from_string = EditorCommand::from("code".to_string());
        assert_eq!(editor_from_string.as_str(), "code");

        let editor_from_str = EditorCommand::from("nano");
        assert_eq!(editor_from_str.as_str(), "nano");
    }

    #[test]
    fn test_editor_command_display() {
        let editor = EditorCommand::new("emacs");
        assert_eq!(format!("{}", editor), "emacs");
    }

    #[test]
    fn test_editor_command_default() {
        let default_editor = EditorCommand::default();
        assert_eq!(default_editor.as_str(), "vi");
    }

    #[test]
    fn test_editor_command_equality() {
        let editor1 = EditorCommand::new("same");
        let editor2 = EditorCommand::from("same");
        let editor3 = EditorCommand::new("different");

        assert_eq!(editor1, editor2);
        assert_ne!(editor1, editor3);
    }

    #[test]
    fn test_blacklist_pattern_creation_and_access() {
        let pattern = BlacklistPattern::new("*.tmp");
        assert_eq!(pattern.as_str(), "*.tmp");

        let pattern_from_string = BlacklistPattern::from("node_modules".to_string());
        assert_eq!(pattern_from_string.as_str(), "node_modules");

        let pattern_from_str = BlacklistPattern::from(".git/*");
        assert_eq!(pattern_from_str.as_str(), ".git/*");
    }

    #[test]
    fn test_blacklist_pattern_display() {
        let pattern = BlacklistPattern::new("build/*");
        assert_eq!(format!("{}", pattern), "build/*");
    }

    #[test]
    fn test_blacklist_pattern_contains() {
        let pattern = BlacklistPattern::new("*.log");
        assert!(pattern.contains('*'));
        assert!(pattern.contains('.'));
        assert!(!pattern.contains('#'));

        let simple_pattern = BlacklistPattern::new("node_modules");
        assert!(!simple_pattern.contains('*'));
        assert!(simple_pattern.contains('_'));
    }

    #[test]
    fn test_blacklist_pattern_equality() {
        let pattern1 = BlacklistPattern::new("same/*");
        let pattern2 = BlacklistPattern::from("same/*");
        let pattern3 = BlacklistPattern::new("different/*");

        assert_eq!(pattern1, pattern2);
        assert_ne!(pattern1, pattern3);
    }

    #[test]
    fn test_serde_serialization() {
        use serde_json;

        let key = IdentKey::new("test_key");
        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: IdentKey = serde_json::from_str(&serialized).unwrap();
        assert_eq!(key, deserialized);

        let template = JournalTemplate::new("test/template");
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: JournalTemplate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(template, deserialized);

        let editor = EditorCommand::new("test_editor");
        let serialized = serde_json::to_string(&editor).unwrap();
        let deserialized: EditorCommand = serde_json::from_str(&serialized).unwrap();
        assert_eq!(editor, deserialized);

        let pattern = BlacklistPattern::new("test/*");
        let serialized = serde_json::to_string(&pattern).unwrap();
        let deserialized: BlacklistPattern = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pattern, deserialized);
    }
}
