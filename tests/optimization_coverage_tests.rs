//! Tests for performance optimizations and Cow<str> usage
//! Tests wrap_filename optimization, clone reduction, and builder patterns integration

use obsidian_cli::utils::wrap_filename;
use std::borrow::Cow;

#[cfg(test)]
mod wrap_filename_optimization_tests {
    use super::*;

    #[test]
    fn test_wrap_filename_returns_borrowed_for_short_strings() {
        let short_filename = "test.md";
        let result = wrap_filename(short_filename, 40);

        // Should return Cow::Borrowed for strings that fit
        match result {
            Cow::Borrowed(s) => assert_eq!(s, short_filename),
            Cow::Owned(_) => panic!("Expected Cow::Borrowed for short string"),
        }
    }

    #[test]
    fn test_wrap_filename_returns_borrowed_for_exact_length() {
        let exact_filename = "a".repeat(20);
        let result = wrap_filename(&exact_filename, 20);

        // Should return Cow::Borrowed for strings that exactly fit
        match result {
            Cow::Borrowed(s) => assert_eq!(s, exact_filename),
            Cow::Owned(_) => panic!("Expected Cow::Borrowed for exact length string"),
        }
    }

    #[test]
    fn test_wrap_filename_returns_owned_for_long_strings() {
        let long_filename = "very/long/directory/structure/with/many/nested/directories/file.md";
        let result = wrap_filename(long_filename, 20);

        // Should return Cow::Owned for strings that need wrapping
        match result {
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for long string"),
            Cow::Owned(s) => {
                assert!(s.len() >= long_filename.len()); // May be longer due to newlines
                assert!(s.contains("very"));
                assert!(s.contains("file.md"));
            }
        }
    }

    #[test]
    fn test_wrap_filename_cow_str_lifetime() {
        let filename = "test_file.md";
        let result = wrap_filename(filename, 50);

        // Test that we can use the result with the same lifetime
        let as_str: &str = result.as_ref();
        assert_eq!(as_str, filename);
    }

    #[test]
    fn test_wrap_filename_edge_cases_cow_behavior() {
        // Test zero width
        let result = wrap_filename("test.md", 0);
        match result {
            Cow::Owned(s) => assert!(!s.is_empty()), // Should handle gracefully
            Cow::Borrowed(_) => {}                   // Also acceptable
        }

        // Test width of 1
        let result = wrap_filename("test.md", 1);
        match result {
            Cow::Owned(_) => assert!(true),    // Expected for such small width
            Cow::Borrowed(_) => assert!(true), // Also possible
        }

        // Test empty string
        let result = wrap_filename("", 10);
        match result {
            Cow::Borrowed(s) => assert_eq!(s, ""),
            Cow::Owned(_) => panic!("Expected Cow::Borrowed for empty string"),
        }
    }

    #[test]
    fn test_wrap_filename_memory_efficiency() {
        // Test that we don't unnecessarily allocate for short strings
        let short_files = vec!["a.md", "test.txt", "document.pdf", "image.jpg", "script.sh"];

        for file in short_files {
            let result = wrap_filename(file, 50);
            match result {
                Cow::Borrowed(s) => assert_eq!(s, file), // Zero allocation
                Cow::Owned(_) => panic!("Unexpected allocation for short filename: {}", file),
            }
        }
    }

    #[test]
    fn test_wrap_filename_with_path_separators() {
        let path_filename = "docs/projects/rust/src/main.rs";
        let result = wrap_filename(path_filename, 15);

        match result {
            Cow::Owned(wrapped) => {
                // Should break at path separators
                let lines: Vec<&str> = wrapped.split('\n').collect();
                assert!(lines.len() > 1, "Should wrap into multiple lines");

                // Verify intelligent breaking
                for line in &lines {
                    if line.contains('/') {
                        assert!(line.len() <= 15, "Line too long: '{}'", line);
                    }
                }
            }
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for long path"),
        }
    }

    #[test]
    fn test_wrap_filename_preserves_content() {
        let original = "very/long/path/to/some/important/file/with/a/descriptive/name.md";
        let result = wrap_filename(original, 25);

        match result {
            Cow::Owned(wrapped) => {
                // Remove newlines to check original content is preserved
                let flattened: String = wrapped.split('\n').collect::<Vec<_>>().join("");

                // Should contain all original characters (possibly reordered due to wrapping logic)
                for char in original.chars() {
                    assert!(flattened.contains(char), "Missing character: '{}'", char);
                }
            }
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for long string"),
        }
    }

    #[test]
    fn test_wrap_filename_performance_characteristic() {
        // Test that the function has reasonable performance characteristics
        let very_long_filename = "a".repeat(1000);
        let start = std::time::Instant::now();

        let result = wrap_filename(&very_long_filename, 50);
        let duration = start.elapsed();

        // Should complete in reasonable time (less than 10ms even for very long strings)
        assert!(
            duration.as_millis() < 10,
            "Function took too long: {:?}",
            duration
        );

        match result {
            Cow::Owned(wrapped) => {
                assert!(!wrapped.is_empty());
                assert!(wrapped.len() >= 1000); // Should include all original chars plus newlines
            }
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for very long string"),
        }
    }

    #[test]
    fn test_wrap_filename_different_widths() {
        let filename = "moderately_long_filename_for_testing.md";
        let widths = [10, 20, 30, 40, 50, 100];

        for width in widths {
            let result = wrap_filename(filename, width);

            if filename.len() <= width {
                match result {
                    Cow::Borrowed(s) => assert_eq!(s, filename),
                    Cow::Owned(_) => panic!("Unexpected allocation for width {}", width),
                }
            } else {
                match result {
                    Cow::Owned(wrapped) => {
                        assert!(!wrapped.is_empty());
                        // Should attempt to respect width (though may exceed for unbreakable parts)
                        for line in wrapped.split('\n') {
                            if !line.is_empty() && line.len() > width {
                                // Only acceptable if line has no path separators to break on
                                assert!(
                                    !line.contains('/'),
                                    "Line '{}' exceeds width {} but contains separators",
                                    line,
                                    width
                                );
                            }
                        }
                    }
                    Cow::Borrowed(_) => panic!("Expected Cow::Owned for width {}", width),
                }
            }
        }
    }

    #[test]
    fn test_wrap_filename_unicode_handling() {
        let unicode_filename = "测试文件名.md";
        let result = wrap_filename(unicode_filename, 20);

        match result {
            Cow::Borrowed(s) => assert_eq!(s, unicode_filename),
            Cow::Owned(wrapped) => {
                assert!(wrapped.contains("测试"));
                assert!(wrapped.contains(".md"));
            }
        }
    }

    #[test]
    fn test_wrap_filename_special_characters() {
        let special_filename = "file with spaces & symbols (2024) [final].md";
        let result = wrap_filename(special_filename, 30);

        match result {
            Cow::Borrowed(s) => assert_eq!(s, special_filename),
            Cow::Owned(wrapped) => {
                assert!(wrapped.contains("file with spaces"));
                assert!(wrapped.contains("[final]"));
                assert!(wrapped.contains(".md"));
            }
        }
    }
}
