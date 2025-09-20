# Obsidian CLI - Makefile for development tasks

# Variables
CARGO = cargo
TARPAULIN = cargo tarpaulin
COVERAGE_DIR = target/coverage

# Default target
.PHONY: help
help:
	@echo "Obsidian CLI Development Commands"
	@echo "================================="
	@echo ""
	@echo "Testing & Coverage:"
	@echo "  test                 Run all tests"
	@echo "  coverage            Generate full coverage report (HTML)"
	@echo "  coverage-text       Generate coverage report (text only)"
	@echo ""
	@echo "Code Quality:"
	@echo "  fmt                 Format all Rust code"
	@echo "  lint                Run clippy linter"
	@echo "  check               Check compilation without building"
	@echo "  clean               Clean build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  build               Build the project"
	@echo "  run                 Run the CLI (pass ARGS='...' for arguments)"
	@echo "  install             Install the binary locally"
	@echo "  quality             Run fmt + lint + check + test"
	@echo ""
	@echo "Examples:"
	@echo "  make coverage        # Generate HTML coverage report"
	@echo "  make quality         # Full quality check"
	@echo "  make run ARGS='--help'  # Run CLI with --help"

# Testing targets
.PHONY: test
test:
	@echo "Running all tests..."
	$(CARGO) test --lib --tests --quiet

# Coverage targets
.PHONY: coverage
coverage:
	@echo "Generating coverage report with HTML output..."
	@mkdir -p $(COVERAGE_DIR)
	@echo "Running all CI-safe test suites..."
	$(TARPAULIN) --test basic_tests \
		--test template_tests \
		--test simple_config_tests \
		--test simple_utils_tests \
		--test command_integration_tests \
		--test frontmatter_edge_cases_tests \
		--test template_error_path_tests \
		--test comprehensive_mcp_server_tests \
		--test advanced_query_engine_tests \
		--test essential_cli_tests \
		--test config_tests \
		--test utils_tests \
		--ignore-tests --timeout 240 --line \
		--jobs 1 \
		--out Html --output-dir $(COVERAGE_DIR) \
		--engine llvm || echo "Coverage completed with warnings"
	@echo "Coverage report generated in $(COVERAGE_DIR)/tarpaulin-report.html"

.PHONY: coverage-text
coverage-text:
	@echo "Running quick coverage check (text output)..."
	$(TARPAULIN) --test basic_tests \
		--test template_tests \
		--test simple_config_tests \
		--test simple_utils_tests \
		--test command_integration_tests \
		--test frontmatter_edge_cases_tests \
		--test template_error_path_tests \
		--test comprehensive_mcp_server_tests \
		--test advanced_query_engine_tests \
		--test essential_cli_tests \
		--test config_tests \
		--test utils_tests \
		--ignore-tests --timeout 240 --line \
		--jobs 1 \
		--engine llvm

# Code quality targets
.PHONY: fmt
fmt:
	@echo "Formatting Rust code..."
	$(CARGO) fmt --all

.PHONY: lint
lint:
	@echo "Running clippy linter..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: check
check:
	@echo "Checking compilation..."
	$(CARGO) check --all-targets

.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf $(COVERAGE_DIR)
	@echo "Clean complete"

# Build targets
.PHONY: build
build:
	@echo "Building project..."
	$(CARGO) build

.PHONY: run
run: build
	@echo "Running obsidian-cli..."
	$(CARGO) run -- $(ARGS)

.PHONY: install
install:
	@echo "Installing obsidian-cli..."
	$(CARGO) install --path .

# Combined targets
.PHONY: quality
quality: fmt lint check test
	@echo "Quality check complete!"

# Project information
.PHONY: stats
stats:
	@echo "Project Statistics"
	@echo "=================="
	@echo "Lines of Rust code:"
	@find src -name "*.rs" -exec wc -l {} + | tail -1
	@echo "Lines of test code:"
	@find tests -name "*.rs" -exec wc -l {} + | tail -1 2>/dev/null || echo "0 tests"
	@echo "Total Rust files:"
	@find . -name "*.rs" | wc -l
	@echo "Dependencies:"
	@grep -c "^[a-zA-Z]" Cargo.toml || echo "0"