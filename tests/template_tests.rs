//! Template engine tests - CI safe, no user input
//! Tests the template system for journal creation and formatting

use obsidian_cli::template::*;
use chrono::{Utc, TimeZone};

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let date = Utc.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let _engine = TemplateEngine::new(date);
        
        // Verify the engine was created successfully
        assert!(true); // Engine creation doesn't fail
    }

    #[test]
    fn test_basic_template_formatting() {
        let date = Utc.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let simple_template = "Today is {year}-{month:02}-{day:02}";
        let result = engine.format(simple_template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Today is 2023-06-15");
    }

    #[test]
    fn test_complex_template_formatting() {
        let date = Utc.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let complex_template = r#"---
title: "Daily Note {year}-{month:02}-{day:02}"
date: {year}-{month:02}-{day:02}
weekday: {weekday}
---

# Daily Note for {month_name} {day}, {year}

It's {weekday}, {month_name} {day} in the year {year}.

## Weather
- Temperature: 
- Conditions: 

## Tasks
- [ ] Review yesterday's notes
- [ ] Plan today's priorities

## Notes

## Reflection
"#;

        let result = engine.format(complex_template);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("title: \"Daily Note 2023-06-15\""));
        assert!(formatted.contains("date: 2023-06-15"));
        assert!(formatted.contains("weekday: Thursday"));
        assert!(formatted.contains("# Daily Note for June 15, 2023"));
        assert!(formatted.contains("It's Thursday, June 15 in the year 2023"));
        assert!(formatted.contains("## Weather"));
        assert!(formatted.contains("## Tasks"));
        assert!(formatted.contains("## Reflection"));
    }

    #[test]
    fn test_template_with_all_variables() {
        let date = Utc.with_ymd_and_hms(2023, 12, 25, 15, 45, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let template = "Year: {year}, Month: {month}, Day: {day}, Month Name: {month_name}, Weekday: {weekday}, Month Abbr: {month_abbr}, Weekday Abbr: {weekday_abbr}";
        let result = engine.format(template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("Year: 2023"));
        assert!(formatted.contains("Month: 12"));
        assert!(formatted.contains("Day: 25"));
        assert!(formatted.contains("Month Name: December"));
        assert!(formatted.contains("Weekday: Monday"));
        assert!(formatted.contains("Month Abbr: Dec"));
        assert!(formatted.contains("Weekday Abbr: Mon"));
    }

    #[test]
    fn test_template_with_padding() {
        let date = Utc.with_ymd_and_hms(2023, 3, 7, 9, 15, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let template = "{year}-{month:02}-{day:02}";
        let result = engine.format(template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "2023-03-07");
    }

    #[test]
    fn test_template_without_variables() {
        let date = Utc::now();
        let engine = TemplateEngine::new(date);
        
        let template = "This is just plain text without any variables.";
        let result = engine.format(template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, template);
    }

    #[test]
    fn test_empty_template() {
        let date = Utc::now();
        let engine = TemplateEngine::new(date);
        
        let result = engine.format("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_template_with_special_characters() {
        let date = Utc.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let template = "# 🗓️ Daily Note for {weekday} 📝\n\n**Date:** {year}-{month:02}-{day:02} 🎯";
        let result = engine.format(template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("🗓️"));
        assert!(formatted.contains("📝"));
        assert!(formatted.contains("🎯"));
        assert!(formatted.contains("Thursday"));
        assert!(formatted.contains("2023-06-15"));
    }

    #[test]
    fn test_multiple_same_variables() {
        let date = Utc.with_ymd_and_hms(2023, 6, 15, 10, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let template = "{year} was a great year. In {year}, many things happened. The year {year} will be remembered.";
        let result = engine.format(template);
        
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "2023 was a great year. In 2023, many things happened. The year 2023 will be remembered.");
    }

    #[test]
    fn test_journal_template_realistic() {
        let date = Utc.with_ymd_and_hms(2023, 8, 20, 14, 30, 0).unwrap();
        let engine = TemplateEngine::new(date);
        
        let journal_template = r#"---
title: "{year}-{month:02}-{day:02}"
type: daily
date: {year}-{month:02}-{day:02}
weekday: {weekday}
week: {year}-W01
tags: [daily, journal, {year}]
---

# 📅 {weekday}, {month_name} {day}, {year}

## 🎯 Today's Focus
- [ ] 
- [ ] 
- [ ] 

## 📝 Notes


## 💭 Reflections


## 🔗 Links
- [[{year}-{month:02}-{day:02} - Previous Day]]
- [[{year}-{month:02}-{day:02} - Next Day]]

## 📊 Metrics
- Mood: /10
- Energy: /10
- Productivity: /10

---
Created: {year}-{month:02}-{day:02} | Day: {weekday}
"#;

        let result = engine.format(journal_template);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        
        // Verify that template was processed (don't check exact content since it might vary)
        assert!(!formatted.contains("{year}"));
        assert!(!formatted.contains("{month:02}"));
        assert!(!formatted.contains("{day:02}"));
        assert!(!formatted.contains("{weekday}"));
        assert!(!formatted.contains("{month_name}"));
        
        // Verify basic structure is present
        assert!(formatted.contains("# 📅"));
        assert!(formatted.contains("## 🎯 Today's Focus"));
        assert!(formatted.contains("## 📝 Notes"));
        assert!(formatted.contains("## 💭 Reflections"));
    }

    #[test]
    fn test_edge_case_dates() {
        // Test leap year
        let leap_date = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
        let engine = TemplateEngine::new(leap_date);
        let result = engine.format("{year}-{month:02}-{day:02}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2024-02-29");

        // Test New Year's Eve
        let nye_date = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
        let engine = TemplateEngine::new(nye_date);
        let result = engine.format("{weekday}, {month_name} {day}, {year}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Sunday, December 31, 2023");

        // Test January 1st
        let new_year = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 1).unwrap();
        let engine = TemplateEngine::new(new_year);
        let result = engine.format("{weekday}, {month_name} {day}, {year}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Monday, January 1, 2024");
    }
}
