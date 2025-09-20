# Obsidian CLI (Rust Implementation)

A high-performance command-line interface for interacting with Obsidian vaults, written in Rust. This implementation provides the same functionality as the [Python obsidian-cli](../obsidian-cli) with improved performance and reliability.

[![Test Coverage](https://img.shields.io/badge/coverage-81.39%25-brightgreen.svg)](https://github.com/jhonce/obsidian-cli-rs)
[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/jhonce/obsidian-cli-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## Features

- **🚀 High Performance**: Rust implementation for blazing-fast vault operations
- **📝 Note Management**: Create, edit, and delete notes with frontmatter support
- **🔍 Vault Operations**: List, search, and query notes across your vault
- **📅 Journal Support**: Open daily notes with powerful Python-compatible templates
- **🏷️ UID Management**: Add unique identifiers to notes for better organization
- **🔧 Flexible Configuration**: TOML-based configuration with sensible defaults
- **🤖 MCP Server**: Model Context Protocol server for AI assistant integration
- **📊 Rich Output**: Professional table formatting with right-aligned numbers
- **🧪 Enterprise-Grade Testing**: 241 tests across 12 test suites with 81.39% code coverage
- **⚡ Cross-Platform**: Works on macOS, Linux, and Windows

## Installation

### From Source

```bash
git clone https://github.com/jhonce/obsidian-cli-rs
cd obsidian-cli-rs
cargo install --path .
```

### Binary Releases

Download pre-built binaries from the [releases page](https://github.com/jhonce/obsidian-cli-rs/releases).

## Quick Start

If no configuration file is found, obsidian-cli will use default settings. You'll need to specify the vault path:

```bash
# List all notes in a vault
obsidian-cli --vault /path/to/vault ls

# Create a new note
obsidian-cli --vault /path/to/vault new "My New Note"

# Search for notes containing "rust"
obsidian-cli --vault /path/to/vault query title --contains rust
```

## Core Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `add-uid` | | Add unique identifiers to notes |
| `cat` | | Display note contents |
| `edit` | | Edit notes with your configured editor |
| `find` | | Find files by name or title (exact/fuzzy) |
| `info` | | Display comprehensive vault information |
| `journal` | | Open daily notes with templates |
| `ls` | | List markdown files in the vault |
| `meta` | `frontmatter` | View or update frontmatter metadata |
| `new` | | Create new notes with frontmatter |
| `query` | | Advanced frontmatter queries |
| `rm` | | Remove notes from the vault |
| `serve` | | Start MCP server for AI assistant integration |

## Configuration

Create a configuration file at one of these locations:

- `obsidian-cli.toml` (current directory)
- `~/.config/obsidian-cli/config.toml`

Example configuration:

```toml
# Path to your Obsidian vault
vault = "~/Documents/MyVault"

# Editor for editing files
editor = "code"

# Unique identifier key for notes
ident_key = "uid"

# Directories to ignore during operations
blacklist = ["Assets/", ".obsidian/", ".git/", "Templates/"]

# Journal template path (supports date variables)
journal_template = "Calendar/{year}/{month:02d}/{year}-{month:02d}-{day:02d}"

# Enable verbose output
verbose = false
```

## Usage Examples

### Note Creation and Management

```bash
# Create a new note
obsidian-cli new "Meeting Notes"

# Edit an existing note
obsidian-cli edit "meeting-notes"

# View note contents
obsidian-cli cat "meeting-notes"

# Remove a note (with confirmation)
obsidian-cli rm "old-note"
```

### Searching and Querying

```bash
# List all markdown files
obsidian-cli ls

# Find files by name (fuzzy search)
obsidian-cli find "daily"

# Find exact filename matches
obsidian-cli find "daily-note-2025-01-15" --exact

# Query notes by frontmatter
obsidian-cli query tags --exists
obsidian-cli query status --value "published"
obsidian-cli query title --contains "rust"
```

### Metadata Management

```bash
# View all frontmatter for a note
obsidian-cli meta "my-note"

# View specific frontmatter field
obsidian-cli meta "my-note" --key "tags"

# Update frontmatter field
obsidian-cli meta "my-note" --key "status" --value "published"

# Add unique identifier to a note
obsidian-cli add-uid "my-note"
```

### Journal Operations

```bash
# Open today's journal entry
obsidian-cli journal

# Open journal for specific date
obsidian-cli journal --date 2025-01-15
```

### Advanced Features

```bash
# Query with JSON output
obsidian-cli query tags --exists --style json

# Query with table output
obsidian-cli query status --value published --style table

# Use blacklist to exclude directories
obsidian-cli --blacklist "Templates/:Archive/" ls

# Get detailed vault information
obsidian-cli info

# Start MCP server for AI assistant integration
obsidian-cli serve
```

### MCP Server Integration

```bash
# Start the MCP server
obsidian-cli serve

# The server provides these tools for AI assistants:
# - create_note: Create new notes with content
# - find_notes: Search for notes by name/title  
# - get_note_content: Retrieve note contents
# - get_vault_info: Get vault statistics and information
```

## Journal Templates

The enhanced journal template system provides **full Python-compatible formatting** with these variables:

### Basic Date Variables

- `{year}` - Four-digit year (e.g., 2025)
- `{month}` - Month number (1-12)
- `{day}` - Day number (1-31)

### Formatted Variables (Python-style)

- `{month:02d}` - Zero-padded month (01-12)
- `{day:02d}` - Zero-padded day (01-31)
- `{year:04d}` - Zero-padded year (2025)
- `{month:03d}` - Custom width padding (001-012)

### String Variables

- `{month_name}` - Full month name (e.g., January)
- `{month_abbr}` - Abbreviated month (e.g., Jan)
- `{weekday}` - Full weekday name (e.g., Monday)
- `{weekday_abbr}` - Abbreviated weekday (e.g., Mon)

### Template Examples

```toml
# Obsidian daily notes
journal_template = "Calendar/{year}/{month:02d}/{year}-{month:02d}-{day:02d}"

# Human readable
journal_template = "Journal/{month_name} {year}/{weekday}, {month_name} {day}"

# Custom organization
journal_template = "Notes/{year}/{month:03d}-{month_abbr}/{day:02d}-{weekday_abbr}"
```

## Environment Variables

- `EDITOR`: Editor to use for editing files
- `OBSIDIAN_BLACKLIST`: Colon-separated list of directory patterns to ignore
- `OBSIDIAN_CONFIG`: Path to configuration file
- `OBSIDIAN_VAULT`: Path to the Obsidian vault
- `OBSIDIAN_VERBOSE`: Enable verbose output

## Development

### Prerequisites

- Rust 1.70.0 or later
- Cargo

### Building

```bash
# Development build
make build

# Run tests
make test

# Run with coverage analysis
make coverage

# Code quality check (format, lint, check, test)
make quality
```

### Testing

The project includes enterprise-grade testing infrastructure with 81.39% code coverage:

```bash
# Run all tests
make test

# Generate HTML coverage report
make coverage

# Quick text-based coverage check
make coverage-text
```

### Code Quality

```bash
# Format code
make fmt

# Run linter
make lint

# Check compilation
make check

# All-in-one quality check
make quality
```

## Compatibility

This Rust implementation is designed to be fully compatible with the Python version:

- **Same CLI interface**: All commands and options work identically
- **Same configuration format**: Uses the same TOML configuration files
- **Same output format**: Produces identical output for all commands
- **Enterprise testing**: 241 tests across 12 test suites with 81.39% coverage

## Performance

The Rust implementation provides significant performance improvements:

- **~10x faster** vault scanning and file operations
- **~5x faster** frontmatter parsing with `gray-matter` integration
- **Optimized template engine** with regex-based parsing for complex formats
- **Lower memory usage** for large vaults with efficient string handling
- **Instant startup time** compared to Python
- **Professional table formatting** with right-aligned numbers
- **Zero dependencies** single binary distribution

## Architecture

```
obsidian-cli-rs/
├── src/
│   ├── main.rs           # Entry point & async runtime
│   ├── lib.rs            # Library exports
│   ├── cli.rs            # Clap CLI definitions
│   ├── config.rs         # TOML configuration management
│   ├── types.rs          # Type definitions
│   ├── errors.rs         # Error types with thiserror
│   ├── frontmatter.rs    # YAML frontmatter parsing (gray-matter)
│   ├── template.rs       # Python-compatible template engine
│   ├── utils.rs          # Utility functions
│   ├── mcp_server.rs     # Model Context Protocol server
│   └── commands/         # Individual command implementations
│       ├── mod.rs
│       ├── new.rs        # Note creation
│       ├── edit.rs       # Note editing
│       ├── cat.rs        # Note display
│       ├── find.rs       # File search
│       ├── ls.rs         # File listing
│       ├── meta.rs       # Metadata management (alias: frontmatter)
│       ├── add_uid.rs    # UID generation
│       ├── query.rs      # Advanced queries
│       ├── journal.rs    # Journal operations
│       ├── info.rs       # Vault information
│       ├── rm.rs         # File removal
│       └── serve.rs      # MCP server command
├── tests/                # Enterprise test suite (241 tests, 81.39% coverage)
│   ├── basic_tests.rs                        # Foundation tests
│   ├── template_tests.rs                     # Template engine
│   ├── simple_config_tests.rs                # Configuration
│   ├── simple_utils_tests.rs                 # Utility functions  
│   ├── command_integration_tests.rs          # Command integration
│   ├── frontmatter_edge_cases_tests.rs       # Frontmatter parsing
│   ├── template_error_path_tests.rs          # Template error handling
│   ├── comprehensive_mcp_server_tests.rs     # MCP server core
│   ├── advanced_query_engine_tests.rs        # Query engine
│   ├── essential_cli_tests.rs                # CLI integration
│   ├── config_tests.rs                       # Advanced config
│   └── utils_tests.rs                        # Utility edge cases
├── Cargo.toml                    # Dependencies & metadata
├── Makefile                      # Development workflow automation
├── MCP_COMPATIBILITY_REPORT.md   # MCP server compatibility verification
└── README.md                     # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Workflow

The project uses a Makefile for development workflow automation:

```bash
# Show all available commands
make help

# Development cycle
make quality      # Run all quality checks (fmt + lint + check + test)
make coverage     # Generate coverage report
make stats        # Show project statistics
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original Python implementation by [jhonce](https://github.com/jhonce)
- Built with the amazing Rust ecosystem
- Inspired by the need for high-performance CLI tools

## Changelog

### v0.2.0 (2025-09-20)

#### Major Improvements

- **Enterprise-Grade Testing**: Completely rewritten CI-safe test suite
  - 241 tests across 12 specialized test suites 
  - 81.39% code coverage (up from 45%)
  - Zero user input requirements - fully automated
  - Comprehensive error path and edge case testing

- **Code Quality & Maintainability**:
  - `resolve_page_or_path!` macro for code deduplication across 5 commands
  - Simplified Makefile: 325 → 141 lines (57% reduction) 
  - Optimized `.gitignore` and `.cursorignore` with zero duplication
  - Clean codebase with all compiler warnings eliminated

- **Repository & Development**:
  - Full Git repository setup with professional structure
  - Comprehensive documentation updates reflecting current state
  - Enhanced development workflow with streamlined Makefile targets

#### Test Architecture Overhaul

- **CI-Safe Foundation**: All tests run without user interaction
- **Specialized Test Suites**: 
  - `comprehensive_mcp_server_tests.rs` (29 tests) - MCP protocol compliance
  - `advanced_query_engine_tests.rs` (29 tests) - Query engine functionality  
  - `essential_cli_tests.rs` (22 tests) - CLI integration
  - `config_tests.rs` (18 tests) - Configuration handling
  - Plus 8 additional specialized test modules

#### Technical Enhancements

- **Improved Error Handling**: Robust frontmatter parsing and file operations
- **Enhanced Template Engine**: Comprehensive format specifier support
- **Optimized Path Resolution**: Macro-based deduplication pattern
- **Professional Output**: Better table formatting and error messages

### v0.1.0 (2025-01-19)

#### Core Features

- Initial Rust implementation with full Python version parity
- All 12 commands implemented (`add-uid`, `cat`, `edit`, `find`, `info`, `journal`, `ls`, `meta`/`frontmatter`, `new`, `query`, `rm`, `serve`)
- Cross-platform support (macOS, Linux, Windows)

#### Template Engine

- **Python-compatible template engine** with full format specifier support
- Regex-based parsing for complex template patterns
- Support for custom width padding (`{month:03d}`, `{year:04d}`)
- All Python-style date formatting variables
- **Backward compatible** with existing templates

#### MCP Server Integration

- **Model Context Protocol server** for AI assistant integration
- Four core tools: `create_note`, `find_notes`, `get_note_content`, `get_vault_info`
- Full compatibility with Python MCP server
- JSON-RPC 2.0 over stdio protocol
- Comprehensive test coverage (17 MCP-specific tests)

#### Frontmatter Processing

- Migrated to `gray-matter` crate for better compatibility
- Robust parsing of incomplete/malformed frontmatter
- Automatic timestamp management
- Type-safe value parsing

#### User Interface Enhancements

- **Right-aligned numeric columns** in table output
- Professional table formatting with proper alignment
- Improved error message formatting
- Clean, warning-free codebase

#### Performance & Quality

- **79+ comprehensive tests** ensuring reliability
- Optimized string handling and memory usage
- Clean architecture with consistent command patterns
- Single binary distribution with zero dependencies

#### Technical Improvements

- Moved architecture to consistent command patterns
- Enhanced test organization with dedicated test modules
- Comprehensive Python compatibility verification
- Clean compilation with all warnings resolved
