use crate::errors::Result;
use crate::types::State;

/// Execute the serve command to start the MCP server.
///
/// This command starts a long-running MCP (Model Context Protocol) server
/// that exposes vault operations as tools for AI assistants and other MCP clients.
///
/// # Arguments
/// * `state` - The application state containing vault configuration
///
/// # Returns
/// * `Result<()>` - Ok on successful server shutdown, Err on startup/runtime errors
pub async fn execute(state: &State) -> Result<()> {
    // Delegate to the MCP server implementation
    // The actual server logic is kept in mcp_server.rs due to its complexity
    crate::mcp_server::serve(state).await
}
