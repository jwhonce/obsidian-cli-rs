//! Essential CLI tests - CI safe, focused on core CLI orchestration
//! Tests CLI argument parsing, config merging, state construction, and command dispatch

use clap::Parser;
use obsidian_cli::cli::{Cli, OutputStyleArg};
use obsidian_cli::types::OutputStyle;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test vault with config
fn create_test_vault_for_cli(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let vault_path = temp_dir.path().join("test-vault");
    let config_path = temp_dir.path().join("config.toml");

    fs::create_dir_all(&vault_path).unwrap();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

    // Create a test config file
    let config_content = r#"
vault = "default-vault-path"
editor = "config-editor"
verbose = false
ident_key = "config-uid"
journal_template = "Journal/{year}/{month:02}/{day:02}"
blacklist = ["config-blacklist", "another-excluded"]
"#;
    fs::write(&config_path, config_content).unwrap();

    (vault_path, config_path)
}

// Helper function to create a test note
fn create_test_note_for_cli(vault_path: &std::path::Path, name: &str, content: &str) {
    let file_path = vault_path.join(format!("{}.md", name));
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();
}

#[cfg(test)]
mod essential_cli_tests {
    use super::*;

    // === CLI ARGUMENT PARSING TESTS ===

    #[tokio::test]
    async fn test_cli_basic_info_command() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "test-editor",
            "info",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_with_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, config_path) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "info",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_verbose_flag() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "cli-editor",
            "--verbose",
            "info",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_blacklist_option() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, config_path) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "--blacklist",
            "cli-override:another-cli-blacklist",
            "info",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    // === COMMAND DISPATCH TESTS ===

    #[tokio::test]
    async fn test_cli_add_uid_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "test-uid", "---\ntitle: Test\n---\nContent");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "true",
            "add-uid",
            "test-uid",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_cat_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(
            &vault_path,
            "test-cat",
            "---\ntitle: Cat Test\n---\nContent",
        );

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "cat",
            "--show-frontmatter",
            "test-cat",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_edit_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "test-edit", "Content to edit");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "true", // Mock editor
            "edit",
            "test-edit",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_find_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "findable-note", "Content");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "find",
            "findable",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_find_exact_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "exact-match", "Content");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "find",
            "--exact",
            "exact-match",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_journal_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "true",
            "journal",
            "--date",
            "2024-09-20",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_ls_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "list-me", "Content");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "ls",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_ls_with_dates_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "dated-note", "Content");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "ls",
            "--date",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_meta_view_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(
            &vault_path,
            "meta-test",
            "---\ntitle: Meta Test\n---\nContent",
        );

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "meta",
            "--key",
            "title",
            "meta-test",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_new_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--editor",
            "true",
            "new",
            "new-note",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_query_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(
            &vault_path,
            "query-test",
            "---\nstatus: active\n---\nContent",
        );

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--value",
            "active",
            "status",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_query_all_styles() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "style-test", "---\ntype: test\n---\nContent");

        // Test Path style
        let args_path = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--style",
            "path",
            "--value",
            "test",
            "type",
        ];
        let cli_path = Cli::try_parse_from(args_path).unwrap();
        assert!(cli_path.run().await.is_ok());

        // Test Title style
        let args_title = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--style",
            "title",
            "--value",
            "test",
            "type",
        ];
        let cli_title = Cli::try_parse_from(args_title).unwrap();
        assert!(cli_title.run().await.is_ok());

        // Test Table style
        let args_table = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--style",
            "table",
            "--value",
            "test",
            "type",
        ];
        let cli_table = Cli::try_parse_from(args_table).unwrap();
        assert!(cli_table.run().await.is_ok());

        // Test JSON style
        let args_json = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--style",
            "json",
            "--value",
            "test",
            "type",
        ];
        let cli_json = Cli::try_parse_from(args_json).unwrap();
        assert!(cli_json.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_cli_rm_force_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        create_test_note_for_cli(&vault_path, "to-delete", "Content to delete");

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "rm",
            "--force",
            "to-delete",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_serve_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "serve",
        ];

        // Test CLI parsing for serve command (should succeed without hanging)
        let _cli = Cli::try_parse_from(args).unwrap();

        // Test that we can create the State object (prerequisites for serve)
        use obsidian_cli::config::Config;
        use obsidian_cli::types::State;

        let config = Config::default();
        let editor = config.get_editor();
        let state = State {
            vault: vault_path.to_path_buf(),
            blacklist: config.blacklist,
            editor,
            ident_key: config.ident_key,
            journal_template: config.journal_template,
            verbose: false,
        };

        // Verify State creation succeeds
        assert!(state.vault.exists());
        assert!(!state.blacklist.is_empty());

        // Test that we can create MCP server instance (without running it)
        use obsidian_cli::mcp_server::ObsidianMcpServer;
        let _server = ObsidianMcpServer::new(state);

        // If we reach here, serve command prerequisites are working
        // This tests everything except the actual infinite loop serve execution
    }

    // === OUTPUT STYLE CONVERSION TESTS ===

    #[test]
    fn test_output_style_conversion_all() {
        assert!(matches!(
            Into::<OutputStyle>::into(OutputStyleArg::Path),
            OutputStyle::Path
        ));
        assert!(matches!(
            Into::<OutputStyle>::into(OutputStyleArg::Title),
            OutputStyle::Title
        ));
        assert!(matches!(
            Into::<OutputStyle>::into(OutputStyleArg::Table),
            OutputStyle::Table
        ));
        assert!(matches!(
            Into::<OutputStyle>::into(OutputStyleArg::Json),
            OutputStyle::Json
        ));
    }

    // === ERROR HANDLING TESTS ===

    #[tokio::test]
    async fn test_cli_invalid_config_path() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, _) = create_test_vault_for_cli(&temp_dir);

        let args = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--config",
            "/nonexistent/config.toml",
            "info",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        let result = cli.run().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_invalid_argument_parsing() {
        // Test invalid command
        let args_invalid = vec!["obsidian-cli", "invalid-command"];
        assert!(Cli::try_parse_from(args_invalid).is_err());

        // Test missing required argument
        let args_missing = vec!["obsidian-cli", "add-uid"];
        assert!(Cli::try_parse_from(args_missing).is_err());
    }

    // === COMPREHENSIVE WORKFLOW TEST ===

    #[tokio::test]
    async fn test_cli_complete_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let (vault_path, config_path) = create_test_vault_for_cli(&temp_dir);

        // 1. Create new note
        let args_new = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "--verbose",
            "--editor",
            "true",
            "new",
            "workflow-test",
        ];
        let cli_new = Cli::try_parse_from(args_new).unwrap();
        assert!(cli_new.run().await.is_ok());

        // 2. Add UID to note (with force since new command already adds UID)
        let args_uid = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "add-uid",
            "--force",
            "workflow-test",
        ];
        let cli_uid = Cli::try_parse_from(args_uid).unwrap();
        assert!(cli_uid.run().await.is_ok());

        // 3. List files with dates
        let args_ls = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "--blacklist",
            "excluded",
            "ls",
            "--date",
        ];
        let cli_ls = Cli::try_parse_from(args_ls).unwrap();
        assert!(cli_ls.run().await.is_ok());

        // 4. Query by UID existence with JSON output
        let args_query = vec![
            "obsidian-cli",
            "--vault",
            vault_path.to_str().unwrap(),
            "query",
            "--exists",
            "--style",
            "json",
            "uid",
        ];
        let cli_query = Cli::try_parse_from(args_query).unwrap();
        assert!(cli_query.run().await.is_ok());
    }
}
