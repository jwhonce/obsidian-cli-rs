#!/usr/bin/env python3
"""
Python Template Demo Script

This script demonstrates template functionality and can be used for comparison
testing with the Rust implementation.
"""

import json
import re
import sys
from datetime import datetime, date, timedelta
from pathlib import Path
from typing import Dict, List, Any, Optional


class TemplateDemo:
    """Python equivalent of the Rust TemplateDemo for comparison testing"""
    
    def __init__(self):
        self.variables = {}
    
    def process_template(self, template: str, variables: Dict[str, str]) -> str:
        """Simple template processing using string replacement"""
        result = template
        for key, value in variables.items():
            placeholder = "{{" + key + "}}"
            result = result.replace(placeholder, str(value))
        return result
    
    def demo_basic_substitution(self) -> str:
        """Demonstrate basic template variable substitution"""
        template = """# {{title}}

Created on: {{date}}
Author: {{author}}

## Content
{{content}}

---
Tags: {{tags}}
"""
        
        variables = {
            "title": "Demo Note",
            "date": datetime.now().strftime("%Y-%m-%d"),
            "author": "Template Demo",
            "content": "This is a demonstration of basic template substitution.",
            "tags": "#demo #template #example"
        }
        
        return self.process_template(template, variables)
    
    def demo_daily_journal(self, target_date: Optional[date] = None) -> str:
        """Demonstrate daily journal template"""
        target_date = target_date or date.today()
        
        template = """---
title: "Daily Journal - {{date}}"
date: {{date}}
type: daily
tags: [daily, journal]
---

# Daily Journal - {{date_formatted}}

## 🎯 Today's Focus
- 

## ✅ Tasks
- [ ] 
- [ ] 
- [ ] 

## 📝 Notes


## 🔗 Links


## 💭 Reflections


---
*Previous: [[{{prev_date}}]] | Next: [[{{next_date}}]]*
"""
        
        variables = {
            "date": target_date.strftime("%Y-%m-%d"),
            "date_formatted": target_date.strftime("%B %d, %Y"),
            "prev_date": (target_date - timedelta(days=1)).strftime("%Y-%m-%d"),
            "next_date": (target_date + timedelta(days=1)).strftime("%Y-%m-%d")
        }
        
        return self.process_template(template, variables)
    
    def demo_meeting_notes(self, meeting_title: str, attendees: List[str]) -> str:
        """Demonstrate meeting notes template"""
        template = """---
title: "{{meeting_title}}"
date: {{date}}
time: {{time}}
type: meeting
attendees: [{{attendees}}]
tags: [meeting]
---

# {{meeting_title}}

**Date:** {{date}}  
**Time:** {{time}}  
**Attendees:** {{attendees_formatted}}

## 📋 Agenda
1. 
2. 
3. 

## 📝 Discussion Notes


## ✅ Action Items
- [ ] **[Person]** - Task description
- [ ] **[Person]** - Task description

## 🔄 Follow-up
- Next meeting: 
- Review date: 

## 🔗 Related
- 

---
*Meeting facilitated by: {{facilitator}}*
"""
        
        now = datetime.now()
        variables = {
            "meeting_title": meeting_title,
            "date": now.strftime("%Y-%m-%d"),
            "time": now.strftime("%H:%M"),
            "attendees": ", ".join(attendees),
            "attendees_formatted": "\n".join(f"- {attendee}" for attendee in attendees),
            "facilitator": attendees[0] if attendees else "Unknown"
        }
        
        return self.process_template(template, variables)
    
    def demo_project_note(self, project_name: str, status: str) -> str:
        """Demonstrate project note template"""
        template = """---
title: "{{project_name}}"
status: {{status}}
created: {{date}}
type: project
tags: [project, {{status}}]
---

# {{project_name}}

**Status:** {{status}}  
**Created:** {{date}}  
**Last Updated:** {{date}}

## 🎯 Objective


## 📋 Requirements
- [ ] 
- [ ] 
- [ ] 

## 🏗️ Implementation Plan
### Phase 1
- [ ] 

### Phase 2
- [ ] 

### Phase 3
- [ ] 

## 📈 Progress
- **Started:** {{date}}
- **Current Phase:** 
- **Completion:** %

## 🧠 Notes & Ideas


## 🔗 Resources
- 

## 👥 Stakeholders
- **Owner:** 
- **Contributors:** 
- **Reviewers:** 

---
*Project Template v1.0*
"""
        
        variables = {
            "project_name": project_name,
            "status": status,
            "date": datetime.now().strftime("%Y-%m-%d")
        }
        
        return self.process_template(template, variables)
    
    def demo_book_note(self, title: str, author: str, book_type: str) -> str:
        """Demonstrate book/article notes template"""
        template = """---
title: "{{title}}"
author: "{{author}}"
type: {{book_type}}
status: reading
tags: [{{book_type}}, reading, notes]
rating: 
started: {{date}}
finished: 
---

# {{title}}
*by {{author}}*

## 📚 Book Information
- **Type:** {{book_type}}
- **Status:** Reading
- **Started:** {{date}}
- **Rating:** ⭐⭐⭐⭐⭐

## 📝 Key Concepts


## 💡 Insights


## 📋 Chapter Notes

### Chapter 1
- 

### Chapter 2
- 

## 🔖 Quotes
> 

## 🧠 My Thoughts


## 📊 Summary


## 🔗 Related
- 

---
*Book notes template - capture knowledge effectively*
"""
        
        variables = {
            "title": title,
            "author": author,
            "book_type": book_type,
            "date": datetime.now().strftime("%Y-%m-%d")
        }
        
        return self.process_template(template, variables)
    
    def run_all_demos(self) -> List[tuple]:
        """Run all template demonstrations"""
        return [
            ("Basic Substitution", self.demo_basic_substitution()),
            ("Daily Journal", self.demo_daily_journal()),
            ("Meeting Notes", self.demo_meeting_notes("Weekly Standup", ["Alice", "Bob", "Carol"])),
            ("Project Note", self.demo_project_note("Obsidian CLI", "active")),
            ("Book Note", self.demo_book_note("The Pragmatic Programmer", "Andy Hunt & Dave Thomas", "book")),
        ]


def compare_with_rust_output(python_demo: TemplateDemo, rust_output_file: Optional[str] = None):
    """Compare Python template output with Rust implementation"""
    print("=== Python Template Demo Comparison ===\n")
    
    demos = python_demo.run_all_demos()
    
    for demo_name, output in demos:
        print(f"--- {demo_name} ---")
        print(output)
        print("\n" + "="*50 + "\n")


def run_specific_demo(demo_name: str, *args):
    """Run a specific demo by name"""
    demo = TemplateDemo()
    
    if demo_name == "basic":
        return demo.demo_basic_substitution()
    elif demo_name == "journal":
        target_date = None
        if args and args[0]:
            try:
                target_date = datetime.strptime(args[0], "%Y-%m-%d").date()
            except ValueError:
                print(f"Invalid date format: {args[0]}. Using today.")
        return demo.demo_daily_journal(target_date)
    elif demo_name == "meeting":
        meeting_title = args[0] if args else "Test Meeting"
        attendees = list(args[1:]) if len(args) > 1 else ["Alice", "Bob"]
        return demo.demo_meeting_notes(meeting_title, attendees)
    elif demo_name == "project":
        project_name = args[0] if args else "Test Project"
        status = args[1] if len(args) > 1 else "active"
        return demo.demo_project_note(project_name, status)
    elif demo_name == "book":
        title = args[0] if args else "Test Book"
        author = args[1] if len(args) > 1 else "Test Author"
        book_type = args[2] if len(args) > 2 else "book"
        return demo.demo_book_note(title, author, book_type)
    else:
        raise ValueError(f"Unknown demo: {demo_name}")


def main():
    """Main entry point for the demo script"""
    if len(sys.argv) < 2:
        print("Usage: python test_template_demo.py <command> [args...]")
        print("Commands:")
        print("  all                    - Run all demos")
        print("  basic                  - Basic substitution demo")
        print("  journal [date]         - Daily journal demo")
        print("  meeting <title> [attendees...] - Meeting notes demo")
        print("  project <name> [status] - Project note demo")
        print("  book <title> [author] [type] - Book note demo")
        sys.exit(1)
    
    command = sys.argv[1]
    args = sys.argv[2:]
    
    demo = TemplateDemo()
    
    if command == "all":
        compare_with_rust_output(demo)
    else:
        try:
            result = run_specific_demo(command, *args)
            print(result)
        except ValueError as e:
            print(f"Error: {e}")
            sys.exit(1)


if __name__ == "__main__":
    main()
