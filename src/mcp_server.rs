use crate::errors::{ConfigError, ObsidianError, Result};
use crate::frontmatter;
use crate::types::Vault;
use crate::utils::is_path_blacklisted;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use walkdir::WalkDir;

pub async fn serve(vault: &Vault) -> Result<()> {
    println!("Starting Obsidian MCP Server...");

    let server = ObsidianMcpServer::new(vault.clone());

    println!("Obsidian MCP Server listening on stdio...");

    server.run_stdio().await
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
    #[serde(rename = "_meta")]
    pub meta: HashMap<String, Value>,
}

impl TextContent {
    #[must_use]
    pub fn new(text: String, operation: &str, status: &str) -> Self {
        let mut meta = HashMap::new();
        meta.insert(
            "operation".to_string(),
            Value::String(operation.to_string()),
        );
        meta.insert("status".to_string(), Value::String(status.to_string()));

        Self {
            content_type: "text".to_string(),
            text,
            meta,
        }
    }

    #[must_use]
    pub fn with_metadata(
        text: String,
        operation: &str,
        status: &str,
        additional_meta: HashMap<String, Value>,
    ) -> Self {
        let mut meta = HashMap::new();
        meta.insert(
            "operation".to_string(),
            Value::String(operation.to_string()),
        );
        meta.insert("status".to_string(), Value::String(status.to_string()));

        for (key, value) in additional_meta {
            meta.insert(key, value);
        }

        Self {
            content_type: "text".to_string(),
            text,
            meta,
        }
    }
}

pub struct ObsidianMcpServer {
    vault: Vault,
}

impl ObsidianMcpServer {
    #[must_use]
    pub fn new(vault: Vault) -> Self {
        Self { vault }
    }

    async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
                        let response = self.handle_request(request).await;
                        let response_json = serde_json::to_string(&response).map_err(|e| {
                            ConfigError::InvalidValue {
                                field: "json_response".to_string(),
                                value: format!("serialization failed: {e}"),
                            }
                        })?;
                        stdout
                            .write_all(response_json.as_bytes())
                            .await
                            .map_err(ObsidianError::Io)?;
                        stdout.write_all(b"\n").await.map_err(ObsidianError::Io)?;
                        stdout.flush().await.map_err(ObsidianError::Io)?;
                    }
                }
                Err(e) => return Err(ObsidianError::Io(e)),
            }
        }

        Ok(())
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "resources": {
                        "listChanged": false
                    },
                    "prompts": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "obsidian-cli",
                    "version": "0.1.0"
                }
            })),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request.params).await,
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(request.params),
            "prompts/list" => Ok(json!({ "prompts": [] })),
            _ => Err(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    fn handle_tools_list(&self) -> std::result::Result<Value, JsonRpcError> {
        Ok(json!({
            "tools": [
                {
                    "name": "create_note",
                    "description": "Create a new note in the Obsidian vault",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": {
                                "type": "string",
                                "description": "Name of the note file"
                            },
                            "content": {
                                "type": "string",
                                "description": "Initial content",
                                "default": ""
                            },
                            "force": {
                                "type": "boolean",
                                "description": "Overwrite if exists",
                                "default": false
                            }
                        },
                        "required": ["filename"]
                    }
                },
                {
                    "name": "find_notes",
                    "description": "Find notes by name or title",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "term": {
                                "type": "string",
                                "description": "Search term"
                            },
                            "exact": {
                                "type": "boolean",
                                "description": "Exact match only",
                                "default": false
                            }
                        },
                        "required": ["term"]
                    }
                },
                {
                    "name": "get_note_content",
                    "description": "Get the content of a specific note",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": {
                                "type": "string",
                                "description": "Name of the note file"
                            },
                            "show_frontmatter": {
                                "type": "boolean",
                                "description": "Include frontmatter",
                                "default": false
                            }
                        },
                        "required": ["filename"]
                    }
                },
                {
                    "name": "get_vault_info",
                    "description": "Get information about the Obsidian vault",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                }
            ]
        }))
    }

    async fn handle_tools_call(
        &self,
        params: Option<Value>,
    ) -> std::result::Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing tool name".to_string(),
                data: None,
            })?;

        let default_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&default_args);

        match name {
            "create_note" => self.handle_create_note(arguments),
            "find_notes" => self.handle_find_notes(arguments),
            "get_note_content" => self.handle_get_note_content(arguments),
            "get_vault_info" => self.handle_get_vault_info(),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Unknown tool: {name}"),
                data: None,
            }),
        }
    }

    fn handle_create_note(&self, arguments: &Value) -> std::result::Result<Value, JsonRpcError> {
        let filename = arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'filename' parameter".to_string(),
                data: None,
            })?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let force = arguments
            .get("force")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Normalize filename for metadata
        let normalized_filename = if std::path::Path::new(filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            filename.to_string()
        } else {
            format!("{filename}.md")
        };

        let full_path = self.vault.path.join(&normalized_filename);

        // Check if file already exists
        if full_path.exists() && !force {
            let mut meta = HashMap::new();
            meta.insert("filename".to_string(), Value::String(normalized_filename));
            meta.insert("exit_code".to_string(), Value::String("1".to_string()));

            let text_content = TextContent::with_metadata(
                format!(
                    "File {}.md already exists. Use force=true to overwrite.",
                    filename
                ),
                "create_note",
                "error",
                meta,
            );

            return Ok(json!([text_content]));
        }

        // Create directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to create directory: {e}"),
                data: None,
            })?;
        }

        // Create the note with content or default template
        let final_content = if content.is_empty() {
            // Create with default frontmatter
            let mut fm = HashMap::new();
            frontmatter::add_default_frontmatter(&mut fm, filename, self.vault.ident_key.as_str());
            frontmatter::serialize_with_frontmatter(&fm, "").map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to create frontmatter: {e}"),
                data: None,
            })?
        } else {
            content.to_string()
        };

        std::fs::write(&full_path, final_content).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to create note: {e}"),
            data: None,
        })?;

        let mut meta = HashMap::new();
        meta.insert(
            "filename".to_string(),
            Value::String(normalized_filename.clone()),
        );

        let text_content = TextContent::with_metadata(
            format!("Successfully created note: {normalized_filename}"),
            "create_note",
            "success",
            meta,
        );

        Ok(json!([text_content]))
    }

    fn handle_find_notes(&self, arguments: &Value) -> std::result::Result<Value, JsonRpcError> {
        let term = arguments
            .get("term")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'term' parameter".to_string(),
                data: None,
            })?;

        let exact = arguments
            .get("exact")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let matches =
            crate::utils::find_matching_files(&self.vault.path, term, exact).map_err(|e| {
                JsonRpcError {
                    code: -32603,
                    message: format!("Error finding notes: {e}"),
                    data: None,
                }
            })?;

        let result_count = matches.len();
        let mut meta = HashMap::new();
        meta.insert("term".to_string(), Value::String(term.to_string()));
        meta.insert("exact".to_string(), Value::Bool(exact));
        meta.insert(
            "result_count".to_string(),
            Value::Number(result_count.into()),
        );

        let text_content = if matches.is_empty() {
            TextContent::with_metadata(
                format!("No files found matching '{term}'"),
                "find_notes",
                "success",
                meta,
            )
        } else {
            let file_list: Vec<String> = matches
                .iter()
                .map(|path| {
                    path.strip_prefix(&self.vault.path)
                        .unwrap_or(path)
                        .display()
                        .to_string()
                })
                .collect();

            let result = format!(
                "Found {} file(s) matching '{}':\n{}",
                result_count,
                term,
                file_list
                    .iter()
                    .map(|f| format!("- {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            TextContent::with_metadata(result, "find_notes", "success", meta)
        };

        Ok(json!([text_content]))
    }

    fn handle_get_note_content(
        &self,
        arguments: &Value,
    ) -> std::result::Result<Value, JsonRpcError> {
        let filename = arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'filename' parameter".to_string(),
                data: None,
            })?;

        let show_frontmatter = arguments
            .get("show_frontmatter")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Try different file paths
        let mut full_path = self.vault.path.join(filename);
        if !full_path.exists()
            && !std::path::Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            full_path = self.vault.path.join(format!("{filename}.md"));
        }

        if !full_path.exists() {
            let mut meta = HashMap::new();
            meta.insert("filename".to_string(), Value::String(filename.to_string()));
            meta.insert(
                "show_frontmatter".to_string(),
                Value::Bool(show_frontmatter),
            );
            meta.insert("exit_code".to_string(), Value::String("2".to_string()));

            let text_content = TextContent::with_metadata(
                format!("File not found: {filename}"),
                "get_note_content",
                "error",
                meta,
            );

            return Ok(json!([text_content]));
        }

        let content = std::fs::read_to_string(&full_path).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to read file {filename}: {e}"),
            data: None,
        })?;

        let final_content = if show_frontmatter {
            content
        } else {
            // Remove frontmatter
            match frontmatter::parse_string(&content) {
                Ok((_, body)) => body,
                Err(_) => content, // If parsing fails, return original content
            }
        };

        let mut meta = HashMap::new();
        meta.insert("filename".to_string(), Value::String(filename.to_string()));
        meta.insert(
            "show_frontmatter".to_string(),
            Value::Bool(show_frontmatter),
        );

        let text_content =
            TextContent::with_metadata(final_content, "get_note_content", "success", meta);

        Ok(json!([text_content]))
    }

    fn handle_get_vault_info(&self) -> std::result::Result<Value, JsonRpcError> {
        let vault_info = self.get_vault_info_for_mcp().map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to get vault info: {e}"),
            data: None,
        })?;

        // Format like Python version
        let file_types_section = if !vault_info.file_type_stats.is_empty() {
            let mut file_types = Vec::new();
            for (ext, stats) in vault_info.file_type_stats.iter() {
                file_types.push(format!(
                    "  - {}: {} files ({})",
                    ext,
                    stats.count,
                    humansize::format_size(stats.total_size, humansize::DECIMAL)
                ));
            }
            format!("\n- File Types by Extension:\n{}\n", file_types.join("\n"))
        } else {
            "\n- File Types: No files found\n".to_string()
        };

        let info = format!(
            "Obsidian Vault Information:\n\
            - Path: {}\n\
            - Total files: {}\n\
            - Usage files: {}\n\
            - Total directories: {}\n\
            - Usage directories: {}\n\
            {}\
            - Editor: {}\n\
            - Blacklist: {:?}\n\
            - Journal template: {}\n\
            - Version: {}",
            vault_info.vault_path.display(),
            vault_info.total_files,
            humansize::format_size(vault_info.usage_files, humansize::DECIMAL),
            vault_info.total_directories,
            humansize::format_size(vault_info.usage_directories, humansize::DECIMAL),
            file_types_section,
            vault_info.editor,
            vault_info.blacklist,
            vault_info.journal_template,
            vault_info.version
        );

        let text_content = TextContent::new(info, "get_vault_info", "success");
        Ok(json!([text_content]))
    }

    fn handle_resources_list(&self) -> std::result::Result<Value, JsonRpcError> {
        Ok(json!({
            "resources": [
                {
                    "uri": format!("obsidian://vault/{}", self.vault.path.display()),
                    "name": "Obsidian Vault",
                    "description": "Access to the Obsidian vault files and metadata",
                    "mimeType": "application/x-obsidian-vault"
                }
            ]
        }))
    }

    fn handle_resources_read(
        &self,
        params: Option<Value>,
    ) -> std::result::Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'uri' parameter".to_string(),
                data: None,
            })?;

        if uri.starts_with("obsidian://vault/") {
            let vault_path = uri
                .strip_prefix("obsidian://vault/")
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid vault URI format".to_string(),
                    data: None,
                })?;
            let full_path = self.vault.path.join(vault_path);

            let content = std::fs::read_to_string(&full_path).map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read file {vault_path}: {e}"),
                data: None,
            })?;

            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": content
                }]
            }))
        } else {
            Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown resource URI: {uri}"),
                data: None,
            })
        }
    }

    // Helper method to get vault info as data structure for MCP
    fn get_vault_info_for_mcp(&self) -> Result<crate::types::VaultInfo> {
        use crate::types::{FileTypeStat, VaultInfo};
        use std::collections::HashMap;

        let mut total_files = 0;
        let mut total_directories = 0;
        let mut usage_files = 0;
        let mut usage_directories = 0;
        let mut file_type_stats: HashMap<String, FileTypeStat> = HashMap::new();
        let mut markdown_files = 0;

        for entry in WalkDir::new(&self.vault.path).follow_links(false) {
            let entry = entry.map_err(|e| ObsidianError::Io(std::io::Error::other(e)))?;
            let path = entry.path();

            if is_path_blacklisted(path, &self.vault.blacklist) {
                continue;
            }

            if path.is_file() {
                total_files += 1;

                if let Ok(metadata) = std::fs::metadata(path) {
                    usage_files += metadata.len();
                }

                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    let ext = extension.to_lowercase();
                    let stat = file_type_stats.entry(ext.clone()).or_insert(FileTypeStat {
                        count: 0,
                        total_size: 0,
                    });
                    stat.count += 1;
                    if let Ok(metadata) = std::fs::metadata(path) {
                        stat.total_size += metadata.len();
                    }

                    if ext == "md" {
                        markdown_files += 1;
                    }
                } else {
                    // Files without extension
                    let stat =
                        file_type_stats
                            .entry("no_extension".to_string())
                            .or_insert(FileTypeStat {
                                count: 0,
                                total_size: 0,
                            });
                    stat.count += 1;
                    if let Ok(metadata) = std::fs::metadata(path) {
                        stat.total_size += metadata.len();
                    }
                }
            } else if path.is_dir() && path != self.vault.path {
                total_directories += 1;
                if let Ok(metadata) = std::fs::metadata(path) {
                    usage_directories += metadata.len();
                }
            }
        }

        let journal_path = crate::utils::format_journal_template(
            self.vault.journal_template.as_str(),
            &crate::types::TemplateVars {
                year: chrono::Utc::now().year(),
                month: chrono::Utc::now().month(),
                day: chrono::Utc::now().day(),
                month_name: chrono::Utc::now().format("%B").to_string(),
                month_abbr: chrono::Utc::now().format("%b").to_string(),
                weekday: chrono::Utc::now().format("%A").to_string(),
                weekday_abbr: chrono::Utc::now().format("%a").to_string(),
            },
        )?;

        Ok(VaultInfo {
            vault_path: self.vault.path.clone(),
            total_files,
            total_directories,
            usage_files,
            usage_directories,
            file_type_stats,
            markdown_files,
            blacklist: self.vault.blacklist.clone(),
            editor: self.vault.editor.clone(),
            journal_template: self.vault.journal_template.clone(),
            journal_path,
            verbose: self.vault.verbose,
            version: "0.1.0".to_string(),
        })
    }
}
