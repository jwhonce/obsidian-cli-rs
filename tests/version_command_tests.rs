//! Tests for the version command functionality

use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_version_subcommand() {
    let mut cmd = Command::cargo_bin("obsidian-cli").unwrap();
    let output = cmd.arg("version").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("obsidian-cli"));
    assert!(stdout.contains("0.1.1"));
    assert_eq!(stdout.trim(), "obsidian-cli 0.1.1");
}

#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("obsidian-cli").unwrap();
    let output = cmd.arg("-V").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("obsidian-cli"));
    assert!(stdout.contains("0.1.1"));
    assert_eq!(stdout.trim(), "obsidian-cli 0.1.1");
}

#[test]
fn test_version_flag_long() {
    let mut cmd = Command::cargo_bin("obsidian-cli").unwrap();
    let output = cmd.arg("--version").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("obsidian-cli"));
    assert!(stdout.contains("0.1.1"));
    assert_eq!(stdout.trim(), "obsidian-cli 0.1.1");
}

#[test]
fn test_version_appears_in_help() {
    let mut cmd = Command::cargo_bin("obsidian-cli").unwrap();
    let output = cmd.arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("version"));
    assert!(stdout.contains("Show version information"));
}

#[test]
fn test_version_matches_cargo_toml() {
    // Read the version from Cargo.toml using cargo metadata
    let output = StdCommand::new("cargo")
        .args(&["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo metadata");

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata");

    let cargo_version = metadata["packages"][0]["version"]
        .as_str()
        .expect("Failed to get version from metadata");

    // Test that our CLI reports the same version
    let mut cmd = Command::cargo_bin("obsidian-cli").unwrap();
    let cli_output = cmd.arg("version").output().unwrap();

    assert!(cli_output.status.success());
    let cli_stdout = String::from_utf8(cli_output.stdout).unwrap();
    assert_eq!(cli_stdout.trim(), format!("obsidian-cli {}", cargo_version));
    assert_eq!(cargo_version, "0.1.1");
}
