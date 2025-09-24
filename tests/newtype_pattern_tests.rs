//! Tests for newtype pattern methods and conversions
//! Tests the .new() methods and other newtype functionality that needs coverage

use obsidian_cli::types::{BlacklistPattern, EditorCommand, IdentKey, JournalTemplate};

#[cfg(test)]
mod newtype_constructor_tests {
    use super::*;

    #[test]
    fn test_ident_key_new_method() {
        let key1 = IdentKey::new("test_key");
        let key2 = IdentKey::new("another_key".to_string());
        let key3 = IdentKey::new(String::from("string_key"));

        assert_eq!(key1.as_str(), "test_key");
        assert_eq!(key2.as_str(), "another_key");
        assert_eq!(key3.as_str(), "string_key");
    }

    #[test]
    fn test_journal_template_new_method() {
        let template1 = JournalTemplate::new("Calendar/{year}");
        let template2 = JournalTemplate::new("Notes/{month:02d}".to_string());
        let template3 = JournalTemplate::new(String::from("Journal/{day}"));

        assert_eq!(template1.as_str(), "Calendar/{year}");
        assert_eq!(template2.as_str(), "Notes/{month:02d}");
        assert_eq!(template3.as_str(), "Journal/{day}");
    }

    #[test]
    fn test_editor_command_new_method() {
        let editor1 = EditorCommand::new("vim");
        let editor2 = EditorCommand::new("nvim".to_string());
        let editor3 = EditorCommand::new(String::from("code"));

        assert_eq!(editor1.as_str(), "vim");
        assert_eq!(editor2.as_str(), "nvim");
        assert_eq!(editor3.as_str(), "code");
    }

    #[test]
    fn test_blacklist_pattern_new_method() {
        let pattern1 = BlacklistPattern::new("*.tmp");
        let pattern2 = BlacklistPattern::new("cache/".to_string());
        let pattern3 = BlacklistPattern::new(String::from("node_modules/*"));

        assert_eq!(pattern1.as_str(), "*.tmp");
        assert_eq!(pattern2.as_str(), "cache/");
        assert_eq!(pattern3.as_str(), "node_modules/*");
    }
}

#[cfg(test)]
mod newtype_conversion_tests {
    use super::*;

    #[test]
    fn test_all_from_string_conversions() {
        let key = IdentKey::from("test".to_string());
        let template = JournalTemplate::from("template".to_string());
        let editor = EditorCommand::from("editor".to_string());
        let pattern = BlacklistPattern::from("pattern".to_string());

        assert_eq!(key.as_str(), "test");
        assert_eq!(template.as_str(), "template");
        assert_eq!(editor.as_str(), "editor");
        assert_eq!(pattern.as_str(), "pattern");
    }

    #[test]
    fn test_all_from_str_conversions() {
        let key = IdentKey::from("test");
        let template = JournalTemplate::from("template");
        let editor = EditorCommand::from("editor");
        let pattern = BlacklistPattern::from("pattern");

        assert_eq!(key.as_str(), "test");
        assert_eq!(template.as_str(), "template");
        assert_eq!(editor.as_str(), "editor");
        assert_eq!(pattern.as_str(), "pattern");
    }

    #[test]
    fn test_as_ref_implementations() {
        let key = IdentKey::new("test");
        let template = JournalTemplate::new("template");
        let editor = EditorCommand::new("editor");
        let pattern = BlacklistPattern::new("pattern");

        // Test AsRef<str> implementations
        let key_ref: &str = key.as_ref();
        let template_ref: &str = template.as_ref();
        let editor_ref: &str = editor.as_ref();
        let pattern_ref: &str = pattern.as_ref();

        assert_eq!(key_ref, "test");
        assert_eq!(template_ref, "template");
        assert_eq!(editor_ref, "editor");
        assert_eq!(pattern_ref, "pattern");
    }
}

#[cfg(test)]
mod newtype_display_tests {
    use super::*;

    #[test]
    fn test_display_implementations() {
        let key = IdentKey::new("display_key");
        let template = JournalTemplate::new("display/{template}");
        let editor = EditorCommand::new("display_editor");
        let pattern = BlacklistPattern::new("display_pattern");

        assert_eq!(format!("{}", key), "display_key");
        assert_eq!(format!("{}", template), "display/{template}");
        assert_eq!(format!("{}", editor), "display_editor");
        assert_eq!(format!("{}", pattern), "display_pattern");
    }

    #[test]
    fn test_display_with_special_characters() {
        let key = IdentKey::new("key_with_underscores");
        let template = JournalTemplate::new("template/{year}-{month:02d}");
        let editor = EditorCommand::new("editor with spaces");
        let pattern = BlacklistPattern::new("pattern/**/glob");

        assert_eq!(format!("{}", key), "key_with_underscores");
        assert_eq!(format!("{}", template), "template/{year}-{month:02d}");
        assert_eq!(format!("{}", editor), "editor with spaces");
        assert_eq!(format!("{}", pattern), "pattern/**/glob");
    }

    #[test]
    fn test_debug_implementations() {
        let key = IdentKey::new("debug_key");
        let template = JournalTemplate::new("debug_template");
        let editor = EditorCommand::new("debug_editor");
        let pattern = BlacklistPattern::new("debug_pattern");

        let key_debug = format!("{:?}", key);
        let template_debug = format!("{:?}", template);
        let editor_debug = format!("{:?}", editor);
        let pattern_debug = format!("{:?}", pattern);

        assert!(key_debug.contains("IdentKey"));
        assert!(key_debug.contains("debug_key"));
        assert!(template_debug.contains("JournalTemplate"));
        assert!(template_debug.contains("debug_template"));
        assert!(editor_debug.contains("EditorCommand"));
        assert!(editor_debug.contains("debug_editor"));
        assert!(pattern_debug.contains("BlacklistPattern"));
        assert!(pattern_debug.contains("debug_pattern"));
    }
}

#[cfg(test)]
mod newtype_equality_tests {
    use super::*;

    #[test]
    fn test_equality_implementations() {
        let key1 = IdentKey::new("same");
        let key2 = IdentKey::from("same");
        let key3 = IdentKey::new("different");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);

        let template1 = JournalTemplate::new("same");
        let template2 = JournalTemplate::from("same");
        let template3 = JournalTemplate::new("different");

        assert_eq!(template1, template2);
        assert_ne!(template1, template3);

        let editor1 = EditorCommand::new("same");
        let editor2 = EditorCommand::from("same");
        let editor3 = EditorCommand::new("different");

        assert_eq!(editor1, editor2);
        assert_ne!(editor1, editor3);

        let pattern1 = BlacklistPattern::new("same");
        let pattern2 = BlacklistPattern::from("same");
        let pattern3 = BlacklistPattern::new("different");

        assert_eq!(pattern1, pattern2);
        assert_ne!(pattern1, pattern3);
    }

    #[test]
    fn test_clone_implementations() {
        let key = IdentKey::new("clone_test");
        let template = JournalTemplate::new("clone_test");
        let editor = EditorCommand::new("clone_test");
        let pattern = BlacklistPattern::new("clone_test");

        let key_clone = key.clone();
        let template_clone = template.clone();
        let editor_clone = editor.clone();
        let pattern_clone = pattern.clone();

        assert_eq!(key, key_clone);
        assert_eq!(template, template_clone);
        assert_eq!(editor, editor_clone);
        assert_eq!(pattern, pattern_clone);

        // Ensure they are separate instances
        assert_eq!(key.as_str(), key_clone.as_str());
        assert_eq!(template.as_str(), template_clone.as_str());
        assert_eq!(editor.as_str(), editor_clone.as_str());
        assert_eq!(pattern.as_str(), pattern_clone.as_str());
    }
}

#[cfg(test)]
mod newtype_edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_strings() {
        let key = IdentKey::new("");
        let template = JournalTemplate::new("");
        let editor = EditorCommand::new("");
        let pattern = BlacklistPattern::new("");

        assert_eq!(key.as_str(), "");
        assert_eq!(template.as_str(), "");
        assert_eq!(editor.as_str(), "");
        assert_eq!(pattern.as_str(), "");
    }

    #[test]
    fn test_unicode_strings() {
        let key = IdentKey::new("测试键");
        let template = JournalTemplate::new("模板/{年份}");
        let editor = EditorCommand::new("编辑器");
        let pattern = BlacklistPattern::new("模式/*.文件");

        assert_eq!(key.as_str(), "测试键");
        assert_eq!(template.as_str(), "模板/{年份}");
        assert_eq!(editor.as_str(), "编辑器");
        assert_eq!(pattern.as_str(), "模式/*.文件");
    }

    #[test]
    fn test_very_long_strings() {
        let long_string = "a".repeat(1000);

        let key = IdentKey::new(&long_string);
        let template = JournalTemplate::new(&long_string);
        let editor = EditorCommand::new(&long_string);
        let pattern = BlacklistPattern::new(&long_string);

        assert_eq!(key.as_str().len(), 1000);
        assert_eq!(template.as_str().len(), 1000);
        assert_eq!(editor.as_str().len(), 1000);
        assert_eq!(pattern.as_str().len(), 1000);
    }

    #[test]
    fn test_special_characters_and_whitespace() {
        let special = "  whitespace  \n\t  and \r\n special chars: !@#$%^&*()  ";

        let key = IdentKey::new(special);
        let template = JournalTemplate::new(special);
        let editor = EditorCommand::new(special);
        let pattern = BlacklistPattern::new(special);

        assert_eq!(key.as_str(), special);
        assert_eq!(template.as_str(), special);
        assert_eq!(editor.as_str(), special);
        assert_eq!(pattern.as_str(), special);
    }
}

#[cfg(test)]
mod editor_command_specific_tests {
    use super::*;

    #[test]
    fn test_editor_command_default() {
        let default_editor = EditorCommand::default();
        assert_eq!(default_editor.as_str(), "vi");
    }

    #[test]
    fn test_editor_command_common_editors() {
        let editors = [
            "vim", "nvim", "nano", "emacs", "code", "atom", "sublime", "notepad", "gedit", "kate",
            "vi",
        ];

        for editor_name in editors {
            let editor = EditorCommand::new(editor_name);
            assert_eq!(editor.as_str(), editor_name);

            let editor_from_string = EditorCommand::from(editor_name.to_string());
            assert_eq!(editor_from_string.as_str(), editor_name);
            assert_eq!(editor, editor_from_string);
        }
    }
}

#[cfg(test)]
mod blacklist_pattern_specific_tests {
    use super::*;

    #[test]
    fn test_blacklist_pattern_contains_method() {
        let glob_pattern = BlacklistPattern::new("*.tmp");
        let simple_pattern = BlacklistPattern::new("cache/");
        let complex_pattern = BlacklistPattern::new("**/node_modules/**");

        // Test the contains method
        assert!(glob_pattern.contains('*'));
        assert!(glob_pattern.contains('.'));
        assert!(!glob_pattern.contains('/'));

        assert!(!simple_pattern.contains('*'));
        assert!(simple_pattern.contains('/'));
        assert!(simple_pattern.contains('c'));

        assert!(complex_pattern.contains('*'));
        assert!(complex_pattern.contains('/'));
        assert!(complex_pattern.contains('_'));
    }

    #[test]
    fn test_blacklist_pattern_common_patterns() {
        let patterns = [
            "*.tmp",
            "*.log",
            "*.bak",
            ".git/",
            ".svn/",
            "node_modules/",
            "target/",
            "build/",
            "dist/",
            "cache/*",
            "**/logs/**",
            "*.o",
        ];

        for pattern_str in patterns {
            let pattern = BlacklistPattern::new(pattern_str);
            assert_eq!(pattern.as_str(), pattern_str);

            let pattern_from_string = BlacklistPattern::from(pattern_str.to_string());
            assert_eq!(pattern_from_string.as_str(), pattern_str);
            assert_eq!(pattern, pattern_from_string);
        }
    }

    #[test]
    fn test_blacklist_pattern_complex_globs() {
        let complex_patterns = [
            "**/*.{tmp,log,bak}",
            "**/target/**/debug/**",
            ".git/**/*",
            "node_modules/**/.*",
            "**/cache/**/*.cache",
        ];

        for pattern_str in complex_patterns {
            let pattern = BlacklistPattern::new(pattern_str);
            assert_eq!(pattern.as_str(), pattern_str);

            // These should all contain glob characters
            assert!(pattern.contains('*') || pattern.contains('?') || pattern.contains('['));
        }
    }
}

#[cfg(test)]
mod serde_compatibility_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serde_round_trip_all_types() {
        let key = IdentKey::new("serde_key");
        let template = JournalTemplate::new("serde/{template}");
        let editor = EditorCommand::new("serde_editor");
        let pattern = BlacklistPattern::new("serde_pattern");

        // Test serialization
        let key_json = serde_json::to_string(&key).unwrap();
        let template_json = serde_json::to_string(&template).unwrap();
        let editor_json = serde_json::to_string(&editor).unwrap();
        let pattern_json = serde_json::to_string(&pattern).unwrap();

        // Test deserialization
        let key_deser: IdentKey = serde_json::from_str(&key_json).unwrap();
        let template_deser: JournalTemplate = serde_json::from_str(&template_json).unwrap();
        let editor_deser: EditorCommand = serde_json::from_str(&editor_json).unwrap();
        let pattern_deser: BlacklistPattern = serde_json::from_str(&pattern_json).unwrap();

        assert_eq!(key, key_deser);
        assert_eq!(template, template_deser);
        assert_eq!(editor, editor_deser);
        assert_eq!(pattern, pattern_deser);
    }
}
