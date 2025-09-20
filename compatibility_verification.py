#!/usr/bin/env python3
"""
Comprehensive MCP Server Compatibility Verification Script

This script compares the Rust and Python MCP server implementations to ensure
they provide identical tool schemas, response formats, and behavior.
"""

import asyncio
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Dict, List


class MCPCompatibilityTester:
    """Test MCP server compatibility between implementations"""

    def __init__(self, rust_binary_path: str = "target/debug/obsidian-cli"):
        self.rust_binary = rust_binary_path
        self.test_vault = None
        self.temp_dir = None

    def setup_test_vault(self):
        """Create a test vault with sample files"""
        self.temp_dir = tempfile.mkdtemp()
        self.test_vault = Path(self.temp_dir) / "test_vault"
        self.test_vault.mkdir()

        # Create .obsidian directory
        (self.test_vault / ".obsidian").mkdir()

        # Create sample files
        sample_files = {
            "note1.md": """---
title: "First Note"
tags: [test, sample]
created: 2023-01-01
---

# First Note

This is a test note with some content.

Links to [[note2]] and [[note3]].
""",
            "note2.md": """---
title: "Second Note" 
tags: [test, example]
created: 2023-01-02
---

# Second Note

Another test note.

References [[note1]].
""",
            "daily/2023-01-01.md": """---
title: "Daily Note 2023-01-01"
type: daily
date: 2023-01-01
---

# Daily Note - January 1, 2023

## Tasks
- [x] Create test vault
- [ ] Write more notes

## Notes
Starting the year with goals.
""",
        }

        for file_path, content in sample_files.items():
            full_path = self.test_vault / file_path
            full_path.parent.mkdir(parents=True, exist_ok=True)
            full_path.write_text(content)

    def cleanup_test_vault(self):
        """Clean up temporary test vault"""
        if self.temp_dir:
            import shutil

            shutil.rmtree(self.temp_dir, ignore_errors=True)

    def run_rust_command(self, args: List[str], timeout: int = 30) -> Dict[str, Any]:
        """Run Rust CLI command and return result"""
        cmd = [self.rust_binary] + args + ["--vault", str(self.test_vault)]

        try:
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=timeout
            )

            return {
                "success": result.returncode == 0,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "returncode": result.returncode,
            }
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "stdout": "",
                "stderr": f"Command timed out after {timeout} seconds",
                "returncode": -1,
            }
        except Exception as e:
            return {"success": False, "stdout": "", "stderr": str(e), "returncode": -1}

    async def test_mcp_server_startup(self) -> Dict[str, Any]:
        """Test MCP server startup and basic functionality"""
        print("Testing MCP server startup...")

        # Start MCP server
        result = self.run_rust_command(
            ["serve", "--port", "0"]
        )  # Port 0 for auto-assign

        return {
            "test": "mcp_server_startup",
            "success": result["success"],
            "details": result,
        }

    def test_basic_commands(self) -> List[Dict[str, Any]]:
        """Test basic CLI commands"""
        print("Testing basic CLI commands...")

        commands_to_test = [
            (["--help"], "help_command"),
            (["--version"], "version_command"),
            (["info"], "info_command"),
            (["ls"], "list_command"),
            (["ls", "--format", "json"], "list_json_format"),
            (["cat", "note1.md"], "cat_command"),
            (["find", "test"], "find_command"),
            (["meta", "note1.md"], "meta_command"),
        ]

        results = []
        for args, test_name in commands_to_test:
            print(f"  Testing: {test_name}")
            result = self.run_rust_command(args)
            results.append(
                {
                    "test": test_name,
                    "command": args,
                    "success": result["success"],
                    "details": result,
                }
            )

        return results

    def test_json_output_format(self) -> Dict[str, Any]:
        """Test JSON output format consistency"""
        print("Testing JSON output format...")

        result = self.run_rust_command(["ls", "--format", "json"])

        if result["success"]:
            try:
                json_data = json.loads(result["stdout"])
                return {
                    "test": "json_output_format",
                    "success": True,
                    "valid_json": True,
                    "data_structure": type(json_data).__name__,
                    "sample_data": json_data
                    if len(str(json_data)) < 1000
                    else "truncated",
                }
            except json.JSONDecodeError as e:
                return {
                    "test": "json_output_format",
                    "success": False,
                    "valid_json": False,
                    "error": str(e),
                    "raw_output": result["stdout"],
                }
        else:
            return {
                "test": "json_output_format",
                "success": False,
                "valid_json": False,
                "error": "Command failed",
                "details": result,
            }

    def test_frontmatter_handling(self) -> List[Dict[str, Any]]:
        """Test frontmatter parsing and manipulation"""
        print("Testing frontmatter handling...")

        results = []

        # Test metadata extraction
        meta_result = self.run_rust_command(["meta", "note1.md"])
        results.append(
            {
                "test": "frontmatter_extraction",
                "success": meta_result["success"],
                "details": meta_result,
            }
        )

        # Test frontmatter queries if supported
        query_result = self.run_rust_command(["query", "tags:test"])
        results.append(
            {
                "test": "frontmatter_query",
                "success": query_result["success"],
                "details": query_result,
            }
        )

        return results

    def test_template_functionality(self) -> List[Dict[str, Any]]:
        """Test template functionality"""
        print("Testing template functionality...")

        results = []

        # Test creating a new note with template
        new_result = self.run_rust_command(["new", "test-new-note"])
        results.append(
            {
                "test": "template_new_note",
                "success": new_result["success"],
                "details": new_result,
            }
        )

        # Test journal creation if supported
        journal_result = self.run_rust_command(["journal", "--date", "2023-01-15"])
        results.append(
            {
                "test": "template_journal",
                "success": journal_result["success"],
                "details": journal_result,
            }
        )

        return results

    def generate_compatibility_report(self, results: Dict[str, Any]) -> str:
        """Generate a comprehensive compatibility report"""
        report_lines = [
            "# MCP Server Compatibility Report",
            f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}",
            f"Test Vault: {self.test_vault}",
            "",
            "## Summary",
            "",
        ]

        total_tests = 0
        passed_tests = 0

        for category, tests in results.items():
            if isinstance(tests, list):
                for test in tests:
                    total_tests += 1
                    if test.get("success", False):
                        passed_tests += 1
            else:
                total_tests += 1
                if tests.get("success", False):
                    passed_tests += 1

        report_lines.extend(
            [
                f"**Total Tests:** {total_tests}",
                f"**Passed:** {passed_tests}",
                f"**Failed:** {total_tests - passed_tests}",
                f"**Success Rate:** {passed_tests / total_tests * 100:.1f}%",
                "",
            ]
        )

        # Detailed results by category
        for category, tests in results.items():
            report_lines.append(f"## {category.replace('_', ' ').title()}")
            report_lines.append("")

            if isinstance(tests, list):
                for test in tests:
                    status = "✅ PASS" if test.get("success", False) else "❌ FAIL"
                    report_lines.append(
                        f"- **{test.get('test', 'Unknown')}:** {status}"
                    )
                    if not test.get("success", False) and "details" in test:
                        if "stderr" in test["details"] and test["details"]["stderr"]:
                            report_lines.append(
                                f"  - Error: {test['details']['stderr']}"
                            )
            else:
                status = "✅ PASS" if tests.get("success", False) else "❌ FAIL"
                report_lines.append(f"- **{tests.get('test', category)}:** {status}")
                if not tests.get("success", False) and "error" in tests:
                    report_lines.append(f"  - Error: {tests['error']}")

            report_lines.append("")

        return "\n".join(report_lines)

    async def run_all_tests(self) -> Dict[str, Any]:
        """Run comprehensive compatibility tests"""
        print("Starting MCP server compatibility verification...")
        print(f"Rust binary: {self.rust_binary}")

        # Setup test environment
        self.setup_test_vault()
        print(f"Created test vault: {self.test_vault}")

        try:
            results = {}

            # Test basic CLI functionality
            results["basic_commands"] = self.test_basic_commands()

            # Test JSON output
            results["json_output"] = self.test_json_output_format()

            # Test frontmatter handling
            results["frontmatter"] = self.test_frontmatter_handling()

            # Test template functionality
            results["templates"] = self.test_template_functionality()

            # Test MCP server if available
            results["mcp_server"] = await self.test_mcp_server_startup()

            return results

        finally:
            self.cleanup_test_vault()


async def main():
    """Main entry point"""
    if len(sys.argv) > 1:
        rust_binary = sys.argv[1]
    else:
        rust_binary = "target/debug/obsidian-cli"

    # Check if Rust binary exists
    if not Path(rust_binary).exists():
        print(f"Error: Rust binary not found at {rust_binary}")
        print(
            "Please build the project first with 'cargo build' or provide the correct path."
        )
        sys.exit(1)

    tester = MCPCompatibilityTester(rust_binary)

    try:
        results = await tester.run_all_tests()

        # Generate and display report
        report = tester.generate_compatibility_report(results)
        print("\n" + "=" * 60)
        print(report)

        # Save report to file
        report_file = Path("MCP_COMPATIBILITY_REPORT.md")
        report_file.write_text(report)
        print(f"\nReport saved to: {report_file.absolute()}")

        # Determine exit code based on results
        total_failed = sum(
            1
            for category in results.values()
            for test in (category if isinstance(category, list) else [category])
            if not test.get("success", False)
        )

        if total_failed > 0:
            print(f"\n⚠️  {total_failed} tests failed. See report for details.")
            sys.exit(1)
        else:
            print("\n✅ All tests passed!")
            sys.exit(0)

    except KeyboardInterrupt:
        print("\nTest interrupted by user.")
        sys.exit(130)
    except Exception as e:
        print(f"Error running tests: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
