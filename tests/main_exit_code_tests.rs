//! Integration tests targeting the binary entry-point to verify error exit codes
//! and cover `src/main.rs` control flow along with related command error paths.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_vault() -> TempDir {
    let temp_dir = TempDir::new().expect("temp dir");
    let vault_path = temp_dir.path().join("vault");
    fs::create_dir(&vault_path).expect("vault dir");
    fs::create_dir(vault_path.join(".obsidian")).expect(".obsidian dir");
    temp_dir
}

fn vault_path(temp_dir: &TempDir) -> std::path::PathBuf {
    temp_dir.path().join("vault")
}

#[test]
fn test_main_cat_missing_file_exit_code() {
    let temp_dir = create_vault();
    let vault_path = vault_path(&temp_dir);

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.arg("--vault")
        .arg(&vault_path)
        .arg("cat")
        .arg("missing-file");

    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_main_new_existing_file_exit_code() {
    let temp_dir = create_vault();
    let vault_path = vault_path(&temp_dir);

    let existing = vault_path.join("existing.md");
    fs::write(&existing, "# Existing\n").expect("write existing file");

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.arg("--vault")
        .arg(&vault_path)
        .arg("new")
        .arg("existing");

    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("File already exists"));
}

#[test]
fn test_main_query_conflicting_arguments_exit_code() {
    let temp_dir = create_vault();
    let vault_path = vault_path(&temp_dir);

    // Create a note so the query command can scan the vault.
    let note_path = vault_path.join("note.md");
    fs::write(&note_path, "---\nstatus: draft\n---\nContent").expect("write note");

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.arg("--vault")
        .arg(&vault_path)
        .arg("query")
        .arg("status")
        .arg("--value")
        .arg("draft")
        .arg("--contains")
        .arg("draft");

    cmd.assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("Cannot specify both"));
}

#[test]
fn test_main_info_successful_exit() {
    let temp_dir = create_vault();
    let vault_path = vault_path(&temp_dir);

    // Provide a simple markdown note to ensure the info command has content.
    fs::write(
        vault_path.join("note.md"),
        "---\ntitle: Info Test\n---\nBody",
    )
    .expect("write info note");

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.arg("--vault").arg(&vault_path).arg("info");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OBSIDIAN VAULT INFORMATION"));
}

#[test]
fn test_main_missing_vault_configuration_exit_code() {
    // Running without --vault or OBSIDIAN_VAULT should trigger the missing vault error path.
    // Ensure no user config or environment leaks into the test by overriding HOME and
    // XDG_CONFIG_HOME to a fresh temporary directory. Also remove any environment
    // variables that could implicitly supply a vault path.
    let temp_dir = TempDir::new().expect("temp dir");
    let config_home = temp_dir.path().join("config");
    fs::create_dir_all(&config_home).expect("config dir");

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.env_remove("OBSIDIAN_VAULT")
        .env_remove("OBSIDIAN_CONFIG")
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("info");

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Missing required configuration field: vault",
        ));
}

#[test]
fn test_main_invalid_vault_structure_exit_code() {
    // Create a directory without .obsidian to trigger InvalidVault error.
    let temp_dir = TempDir::new().expect("temp dir");
    let invalid_vault = temp_dir.path().join("invalid-vault");
    fs::create_dir(&invalid_vault).expect("vault dir");

    let mut cmd = Command::cargo_bin("obsidian-cli").expect("binary");
    cmd.arg("--vault").arg(&invalid_vault).arg("info");

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid Obsidian vault"));
}

fn _cleanup_path(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}
