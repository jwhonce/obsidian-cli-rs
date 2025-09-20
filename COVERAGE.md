# Test Coverage Guide

This document explains how to run and analyze test coverage for the Obsidian CLI project.

## Quick Start

```bash
# View all available commands
make help

# Run quick coverage check
make coverage-minimal

# Generate full HTML coverage report
make coverage-html

# Run only MCP server tests
make test-mcp
```

## Coverage Targets

### Basic Coverage Commands

| Command | Description | Output |
|---------|-------------|--------|
| `make coverage-minimal` | Quick coverage check (text only) | Terminal output |
| `make coverage` | Full coverage analysis | HTML + terminal |
| `make coverage-html` | Generate HTML report with LCOV | HTML + LCOV files |
| `make coverage-ci` | CI-friendly XML output | XML for CI/CD |

### Specialized Coverage Commands

| Command | Description | Focus |
|---------|-------------|-------|
| `make coverage-mcp` | MCP server coverage only | `src/mcp_server.rs` |
| `make coverage-new` | New test suites only | Recently added tests |
| `make coverage-before` | Baseline coverage | Original tests only |
| `make coverage-after` | Full coverage | All tests including new ones |

## Test Commands

### Running Tests

```bash
# Run all tests
make test

# Run specific test categories  
make test-unit           # Unit tests only
make test-integration    # Integration tests only
make test-mcp            # MCP server tests only
make test-commands       # Command tests only
make test-errors         # Error path tests only
```

### Development Workflow

```bash
# Check code quality
make quality             # Format + lint + check + test + coverage

# CI simulation
make ci                  # Full CI pipeline locally

# Watch for changes (requires cargo-watch)
make watch-test          # Auto-run tests on change
make watch-coverage      # Auto-run coverage on change
```

## Coverage Reports

### HTML Reports

HTML coverage reports are generated in the `target/coverage/` directory:

- **Full coverage**: `target/coverage/tarpaulin-report.html`
- **MCP coverage**: `target/coverage/mcp/tarpaulin-report.html`

Open these files in your browser for detailed line-by-line coverage analysis.

### Understanding Coverage Output

```bash
|| src/mcp_server.rs: 213/264 +80.68%
|| 
48.17% coverage, 422/876 lines covered, +4.22% change in coverage
```

- `213/264` = covered lines / total lines
- `+80.68%` = coverage percentage for this module  
- `48.17%` = overall project coverage percentage
- `+4.22%` = change from previous run

## Test Suites

### Current Test Organization

| Test Suite | File | Focus | Tests |
|------------|------|-------|-------|
| **MCP Server Core** | `comprehensive_mcp_server_tests.rs` | JSON-RPC, tools, basic MCP functionality | 20 |
| **MCP Server Extended** | `mcp_server_coverage_extension.rs` | Advanced scenarios, edge cases | 15 |
| **Command Coverage** | `command_coverage_tests.rs` | Previously untested commands | 20 |
| **Error Paths** | `error_path_coverage_tests.rs` | Error handling, edge cases | 19 |
| **Unit Tests** | `unit_tests.rs` | Core functionality | 7 |
| **Integration** | `integration_tests.rs` | CLI integration | 7 |
| **Templates** | `template_tests.rs` | Template engine | 6 |
| **Compatibility** | `compatibility_tests.rs` | Cross-platform | 6 |

### Coverage by Module

| Module | Coverage | Lines | Status |
|--------|----------|--------|-------|
| `mcp_server.rs` | **80.68%** | 213/264 | ✅ Excellent |
| `utils.rs` | **88.76%** | 79/89 | ✅ Excellent |  
| `frontmatter.rs` | **93.55%** | 29/31 | ✅ Excellent |
| `journal.rs` | **96%** | 24/25 | ✅ Excellent |
| `serve.rs` | **100%** | 2/2 | ✅ Complete |
| `rm.rs` | **81.82%** | 9/11 | ✅ Good |
| `template.rs` | **61.33%** | 46/75 | ⚠️ Moderate |
| `config.rs` | **21.43%** | 9/42 | ❌ Needs work |
| `query.rs` | **0%** | 0/116 | ❌ No coverage |
| `ls.rs` | **0%** | 0/44 | ❌ No coverage |

## Development Tips

### Installing Dependencies

```bash
# Install required tools
make setup

# Install cargo-tarpaulin manually if needed
cargo install cargo-tarpaulin
```

### Performance Optimization

```bash
# Quick feedback during development
make coverage-minimal    # ~30 seconds

# Full analysis for CI/reporting  
make coverage-html       # ~2 minutes

# Module-specific testing
make coverage-mcp        # Focus on MCP server only
```

### Troubleshooting

**Coverage reports not generating?**
```bash
# Check if tarpaulin is installed
cargo tarpaulin --version

# Install if missing
cargo install cargo-tarpaulin
```

**Tests failing?**
```bash
# Run individual test suites
make test-mcp
make test-commands
make test-errors

# Check specific test
cargo test test_name -- --nocapture
```

**HTML report not opening?**
```bash
# Manual path (macOS)
open target/coverage/tarpaulin-report.html

# Manual path (Linux)
xdg-open target/coverage/tarpaulin-report.html
```

## Coverage Goals

### Current Status: 48.17% ✅

### Next Milestones:

- **60% Coverage**: Add `ls` and `query` command tests
- **70% Coverage**: Expand `template` and `config` module tests  
- **80% Coverage**: Add comprehensive error path testing
- **90% Coverage**: Add integration and end-to-end scenarios

### Priority Areas:

1. **`query.rs`** (116 untested lines) - Highest impact
2. **`ls.rs`** (44 untested lines) - Medium impact
3. **`config.rs`** (33 untested lines) - Configuration edge cases
4. **`template.rs`** (29 untested lines) - Template error handling

## Best Practices

### Writing Coverage Tests

1. **Focus on critical paths** - Test the most important functionality first
2. **Test error conditions** - Cover error handling and edge cases
3. **Use realistic data** - Test with real-world file structures and content
4. **Verify behavior** - Don't just call functions, verify correct results

### Measuring Progress

```bash
# Before making changes
make coverage-minimal

# After making changes  
make coverage-minimal

# Compare the output to see improvement
```

### CI Integration

```bash
# Generate CI-friendly coverage data
make coverage-ci

# This creates coverage.xml for tools like Codecov
```

## Contributing

When adding new code:

1. **Run existing tests**: `make test`
2. **Check current coverage**: `make coverage-minimal` 
3. **Add tests for new code**: Follow existing patterns
4. **Verify improvement**: `make coverage-minimal`
5. **Generate HTML report**: `make coverage-html` (for detailed review)

The goal is to maintain or improve coverage with each contribution.
