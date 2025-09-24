use obsidian_cli::commands::rename;
use obsidian_cli::types::Vault;
use std::fs;
use tempfile::TempDir;

fn create_test_vault() -> (TempDir, Vault) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().to_path_buf();

    let vault = Vault {
        path: vault_path,
        blacklist: vec![],
        editor: "nano".to_string(),
        ident_key: "id".to_string(),
        journal_template: "Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}".to_string(),
        verbose: false,
    };

    (temp_dir, vault)
}

#[tokio::test]
async fn test_rename_simple_file() {
    let (_temp_dir, vault) = create_test_vault();

    // Create a test file
    let test_file = vault.path.join("old_name.md");
    fs::write(&test_file, "# Test Content\nThis is a test file.").unwrap();

    // Rename the file
    let result = rename::execute(&vault, &test_file, "new_name.md", false);
    assert!(result.is_ok());

    // Check that old file doesn't exist
    assert!(!test_file.exists());

    // Check that new file exists
    let new_file = vault.path.join("new_name.md");
    assert!(new_file.exists());

    // Check content is preserved
    let content = fs::read_to_string(&new_file).unwrap();
    assert_eq!(content, "# Test Content\nThis is a test file.");
}

#[tokio::test]
async fn test_rename_with_subdirectory() {
    let (_temp_dir, vault) = create_test_vault();

    // Create subdirectory
    fs::create_dir_all(vault.path.join("subdir")).unwrap();
    
    // Create a test file
    let test_file = vault.path.join("old_name.md");
    fs::write(&test_file, "# Test Content").unwrap();

    // Rename to subdirectory
    let result = rename::execute(&vault, &test_file, "subdir/new_name.md", false);
    assert!(result.is_ok());

    // Check that old file doesn't exist
    assert!(!test_file.exists());

    // Check that new file exists in subdirectory
    let new_file = vault.path.join("subdir/new_name.md");
    assert!(new_file.exists());
}

#[tokio::test]
async fn test_rename_auto_add_md_extension() {
    let (_temp_dir, vault) = create_test_vault();

    // Create a test markdown file
    let test_file = vault.path.join("old_name.md");
    fs::write(&test_file, "# Test Content").unwrap();

    // Rename without .md extension
    let result = rename::execute(&vault, &test_file, "new_name", false);
    assert!(result.is_ok());

    // Check that new file has .md extension
    let new_file = vault.path.join("new_name.md");
    assert!(new_file.exists());
}

#[tokio::test]
async fn test_rename_with_wiki_links() {
    let (_temp_dir, vault) = create_test_vault();

    // Create test files
    let old_file = vault.path.join("old_name.md");
    let linking_file = vault.path.join("linking_file.md");
    
    fs::write(&old_file, "# Old File\nThis is the original file.").unwrap();
    fs::write(&linking_file, "# Linking File\n\nThis links to [[old_name]] and also [[old_name|display text]].\n\nAlso see [[old_name#section]].").unwrap();

    // Rename with link updating
    let result = rename::execute(&vault, &old_file, "new_name.md", true);
    assert!(result.is_ok());

    // Check that the linking file was updated
    let updated_content = fs::read_to_string(&linking_file).unwrap();
    assert!(updated_content.contains("[[new_name]]"));
    assert!(updated_content.contains("[[new_name|display text]]"));
    assert!(updated_content.contains("[[new_name#section]]"));
    assert!(!updated_content.contains("[[old_name]]"));
}

#[tokio::test]
async fn test_rename_file_not_found() {
    let (_temp_dir, vault) = create_test_vault();

    // Try to rename non-existent file
    let non_existent = vault.path.join("does_not_exist.md");
    let result = rename::execute(&vault, &non_existent, "new_name.md", false);
    
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("File not found"));
}

#[tokio::test]
async fn test_rename_target_exists() {
    let (_temp_dir, vault) = create_test_vault();

    // Create both source and target files
    let source_file = vault.path.join("source.md");
    let target_file = vault.path.join("target.md");
    
    fs::write(&source_file, "Source content").unwrap();
    fs::write(&target_file, "Target content").unwrap();

    // Try to rename to existing file
    let result = rename::execute(&vault, &source_file, "target.md", false);
    
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Target file already exists"));
}

#[tokio::test]
async fn test_rename_complex_wiki_links() {
    let (_temp_dir, vault) = create_test_vault();

    // Create test files
    let old_file = vault.path.join("complex_name.md");
    let linking_file = vault.path.join("complex_links.md");
    
    fs::write(&old_file, "# Complex File").unwrap();
    fs::write(&linking_file, "# Complex Links\n\n[[complex_name]] simple link\n[[complex_name|Custom Display]] with display\n[[complex_name#intro]] with section\n[[complex_name#section|Display]] with both\n\nNot affected: [[other_file]] and [[complex_name_extended]].").unwrap();

    // Rename with link updating
    let result = rename::execute(&vault, &old_file, "renamed_complex.md", true);
    assert!(result.is_ok());

    // Check that links were updated correctly
    let updated_content = fs::read_to_string(&linking_file).unwrap();
    
    // Should be updated
    assert!(updated_content.contains("[[renamed_complex]] simple link"));
    assert!(updated_content.contains("[[renamed_complex|Custom Display]] with display"));
    assert!(updated_content.contains("[[renamed_complex#intro]] with section"));
    assert!(updated_content.contains("[[renamed_complex#section|Display]] with both"));
    
    // Should not be affected
    assert!(updated_content.contains("[[other_file]]"));
    assert!(updated_content.contains("[[complex_name_extended]]"));
    
    // Old references should be gone
    assert!(!updated_content.contains("[[complex_name]]"));
    assert!(!updated_content.contains("[[complex_name|"));
    assert!(!updated_content.contains("[[complex_name#"));
}
