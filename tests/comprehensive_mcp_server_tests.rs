//! Comprehensive MCP (Model Context Protocol) server tests - CI safe, no user input
//! Tests the complete JSON-RPC protocol implementation, tool operations, and resource management

use obsidian_cli::mcp_server::*;
use obsidian_cli::types::Vault;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

// Helper function to create a test vault with MCP server
fn create_test_vault_for_mcp(temp_dir: &TempDir) -> Vault {
    let vault_path = temp_dir.path();
    fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

    Vault {
        path: vault_path.to_path_buf(),
        blacklist: vec![".obsidian".to_string()],
        editor: "true".to_string(), // Safe mock editor
        ident_key: "uid".to_string(),
        journal_template: "Calendar/{year}/{month:02}/{year}-{month:02}-{day:02}".to_string(),
        verbose: false,
    }
}

// Helper function to create a test note
fn create_test_note_for_mcp(
    vault_path: &std::path::Path,
    name: &str,
    frontmatter: &str,
    content: &str,
) {
    let file_path = vault_path.join(format!("{}.md", name));
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let full_content = if frontmatter.is_empty() {
        content.to_string()
    } else {
        format!("---\n{}\n---\n{}", frontmatter, content)
    };
    fs::write(&file_path, full_content).unwrap();
}

#[cfg(test)]
mod comprehensive_mcp_server_tests {
    use super::*;

    // === CORE PROTOCOL TESTS ===

    #[tokio::test]
    async fn test_obsidian_mcp_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);

        let server = ObsidianMcpServer::new(vault.clone());

        // Verify server can be created (test basic functionality instead of private fields)
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(999)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_request_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["capabilities"]["resources"].is_object());
        assert_eq!(result["serverInfo"]["name"], "obsidian-cli");
    }

    #[tokio::test]
    async fn test_handle_request_unknown_method() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "unknown_method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(2)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    // === TOOLS TESTS ===

    #[tokio::test]
    async fn test_handle_tools_list() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());

        // Check for expected tools
        let tool_names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"create_note"));
        assert!(tool_names.contains(&"find_notes"));
        assert!(tool_names.contains(&"get_note_content"));
        assert!(tool_names.contains(&"get_vault_info"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_params() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_tool_name() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(5)),
            method: "tools/call".to_string(),
            params: Some(json!({"arguments": {}})),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Missing tool name");
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_tool() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(6)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "unknown_tool",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown tool: unknown_tool"));
    }

    // === CREATE NOTE TOOL TESTS ===

    #[tokio::test]
    async fn test_create_note_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(7)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_note",
                "arguments": {
                    "filename": "test-note",
                    "content": "This is test content",
                    "force": false
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Verify file was created
        let note_path = temp_dir.path().join("test-note.md");
        assert!(note_path.exists());

        let content = fs::read_to_string(&note_path).unwrap();
        assert!(content.contains("This is test content"));
        // Note: UID is only added when content is empty, not when content is provided
    }

    #[tokio::test]
    async fn test_create_note_tool_missing_filename() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(8)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_note",
                "arguments": {
                    "content": "Content without filename"
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.message.contains("filename"));
    }

    #[tokio::test]
    async fn test_create_note_tool_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        // Create initial file
        create_test_note_for_mcp(
            temp_dir.path(),
            "existing-note",
            "title: Original",
            "Original content",
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(9)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_note",
                "arguments": {
                    "filename": "existing-note",
                    "content": "New overwritten content",
                    "force": true
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Verify file was overwritten
        let note_path = temp_dir.path().join("existing-note.md");
        let content = fs::read_to_string(&note_path).unwrap();
        assert!(content.contains("New overwritten content"));
        assert!(!content.contains("Original content"));
    }

    // === FIND NOTES TOOL TESTS ===

    #[tokio::test]
    async fn test_find_notes_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        // Create test notes
        create_test_note_for_mcp(
            temp_dir.path(),
            "findable-note",
            "title: Findable Note",
            "Content",
        );
        create_test_note_for_mcp(
            temp_dir.path(),
            "another-note",
            "title: Another Note",
            "Content",
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(10)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "find_notes",
                "arguments": {
                    "term": "findable",
                    "exact": false
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert!(!content_array.is_empty());

        let text_content = &content_array[0];
        assert_eq!(text_content["type"], "text");
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("findable-note"));
    }

    #[tokio::test]
    async fn test_find_notes_tool_exact_match() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        create_test_note_for_mcp(
            temp_dir.path(),
            "exact-match",
            "title: Exact Match",
            "Content",
        );
        create_test_note_for_mcp(
            temp_dir.path(),
            "partial-exact",
            "title: Partial",
            "Content",
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(11)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "find_notes",
                "arguments": {
                    "term": "exact-match",
                    "exact": true
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert_eq!(content_array.len(), 1);

        let text_content = &content_array[0];
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("exact-match"));
        assert!(!text_content["text"]
            .as_str()
            .unwrap()
            .contains("partial-exact"));
    }

    #[tokio::test]
    async fn test_find_notes_tool_missing_term() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(12)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "find_notes",
                "arguments": {
                    "exact": false
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.message.contains("term"));
    }

    // === GET NOTE CONTENT TOOL TESTS ===

    #[tokio::test]
    async fn test_get_note_content_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        create_test_note_for_mcp(
            temp_dir.path(),
            "content-note",
            "title: Content Note",
            "This is the note content",
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(13)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_note_content",
                "arguments": {
                    "filename": "content-note",
                    "show_frontmatter": true
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert!(!content_array.is_empty());

        let text_content = &content_array[0];
        assert_eq!(text_content["type"], "text");
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("This is the note content"));
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("title: Content Note"));
    }

    #[tokio::test]
    async fn test_get_note_content_tool_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(14)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_note_content",
                "arguments": {
                    "filename": "nonexistent-note"
                }
            })),
        };

        let response = server.handle_request(request).await;

        // MCP server returns success with error message in text content, not JSON-RPC error
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert!(!content_array.is_empty());

        let text_content = &content_array[0];
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("File not found"));
    }

    // === GET VAULT INFO TOOL TESTS ===

    #[tokio::test]
    async fn test_get_vault_info_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        // Create some test notes
        create_test_note_for_mcp(temp_dir.path(), "info-note-1", "title: Info 1", "Content");
        create_test_note_for_mcp(temp_dir.path(), "info-note-2", "title: Info 2", "Content");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(15)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_vault_info",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert!(!content_array.is_empty());

        let text_content = &content_array[0];
        assert_eq!(text_content["type"], "text");
        let info_text = text_content["text"].as_str().unwrap();
        assert!(info_text.contains("Vault")); // Should contain vault information
    }

    // === RESOURCES TESTS ===

    #[tokio::test]
    async fn test_handle_resources_list() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(16)),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let resources = result["resources"].as_array().unwrap();
        assert!(!resources.is_empty());

        let resource = &resources[0];
        assert!(resource["uri"]
            .as_str()
            .unwrap()
            .contains("obsidian://vault/"));
        assert_eq!(resource["name"], "Obsidian Vault");
        assert_eq!(resource["mimeType"], "application/x-obsidian-vault");
    }

    #[tokio::test]
    async fn test_handle_resources_read_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        create_test_note_for_mcp(
            temp_dir.path(),
            "resource-note",
            "title: Resource",
            "Resource content",
        );

        let vault_path = temp_dir.path().display().to_string();
        let uri = format!("obsidian://vault/{}/resource-note.md", vault_path);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(17)),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": uri
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let contents = result["contents"].as_array().unwrap();
        assert!(!contents.is_empty());

        let content = &contents[0];
        assert_eq!(content["mimeType"], "text/markdown");
        assert!(content["text"]
            .as_str()
            .unwrap()
            .contains("Resource content"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_params() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(18)),
            method: "resources/read".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_uri() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(19)),
            method: "resources/read".to_string(),
            params: Some(json!({"not_uri": "value"})),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing 'uri' parameter"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_unknown_uri() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(20)),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "https://example.com/unknown"
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown resource URI"));
    }

    // === PROMPTS TESTS ===

    #[tokio::test]
    async fn test_handle_prompts_list() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(21)),
            method: "prompts/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let prompts = result["prompts"].as_array().unwrap();
        assert!(prompts.is_empty()); // Currently returns empty array
    }

    // === TEXT CONTENT STRUCT TESTS ===

    #[test]
    fn test_text_content_creation() {
        let text_content = TextContent::new("Test content".to_string(), "create", "success");

        assert_eq!(text_content.content_type, "text");
        assert_eq!(text_content.text, "Test content");
        assert_eq!(text_content.meta["operation"], "create");
        assert_eq!(text_content.meta["status"], "success");
    }

    // === JSON-RPC STRUCTURE TESTS ===

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(123)),
            method: "test_method".to_string(),
            params: Some(json!({"key": "value"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("test_method"));
        assert!(serialized.contains("\"id\":123"));
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(456)),
            result: Some(json!({"data": "test"})),
            error: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"data\":\"test\""));
        assert!(serialized.contains("\"id\":456"));
        assert!(!serialized.contains("error")); // Should be omitted when None
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(json!({"details": "additional info"})),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("-32600"));
        assert!(serialized.contains("Invalid Request"));
        assert!(serialized.contains("additional info"));
    }

    // === EDGE CASES AND ERROR HANDLING ===

    #[tokio::test]
    async fn test_create_note_with_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(22)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_note",
                "arguments": {
                    "filename": "nested/folder/deep-note",
                    "content": "Deep nested content"
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Verify nested file was created
        let note_path = temp_dir.path().join("nested/folder/deep-note.md");
        assert!(note_path.exists());

        let content = fs::read_to_string(&note_path).unwrap();
        assert!(content.contains("Deep nested content"));
    }

    #[tokio::test]
    async fn test_find_notes_empty_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(23)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "find_notes",
                "arguments": {
                    "term": "anything",
                    "exact": false
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let content_array = result.as_array().unwrap();
        assert_eq!(content_array.len(), 1);

        let text_content = &content_array[0];
        assert!(text_content["text"]
            .as_str()
            .unwrap()
            .contains("No files found matching 'anything'"));
    }

    #[tokio::test]
    async fn test_unicode_content_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_test_vault_for_mcp(&temp_dir);
        let server = ObsidianMcpServer::new(vault);

        let unicode_content = "测试内容 🎌 ñáéíóú ✨ こんにちは";

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(24)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_note",
                "arguments": {
                    "filename": "unicode-test",
                    "content": unicode_content
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Verify unicode content is preserved
        let note_path = temp_dir.path().join("unicode-test.md");
        let content = fs::read_to_string(&note_path).unwrap();
        assert!(content.contains("测试内容"));
        assert!(content.contains("🎌"));
        assert!(content.contains("こんにちは"));
    }
}
