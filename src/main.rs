use eyre::Result;
use async_trait::async_trait;
use omnisearch_mcp::{config::validate_config, providers::initialize_providers};
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequest, CallToolResult, Implementation, InitializeResult,
    ListToolsRequest, ListToolsResult, RpcError, ServerCapabilities, ServerCapabilitiesTools,
    LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{
    mcp_server::{server_runtime, ServerHandler, ServerRuntime},
    McpServer, StdioTransport, TransportOptions,
};

struct OmnisearchServerHandler;

#[async_trait]
impl ServerHandler for OmnisearchServerHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        // Initialize our providers when listing tools
        validate_config().map_err(|e| RpcError::internal_error().with_message(e.to_string()))?;
        initialize_providers();

        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: vec![],
        })
    }

    async fn handle_call_tool_request(
        &self,
        _request: CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        Err(CallToolError::new(
            RpcError::method_not_found().with_message("Tool not found".to_string()),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Define server details and capabilities
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "omnisearch-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Omnisearch MCP Server".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("Omnisearch MCP Server - Unified search and AI tools".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // Create std transport with default options
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| eyre::eyre!("Failed to create transport: {}", e))?;

    // Instantiate our custom handler for handling MCP messages
    let handler = OmnisearchServerHandler {};

    // Create a MCP server
    let server: ServerRuntime = server_runtime::create_server(server_details, transport, handler);

    println!("Omnisearch MCP server running on stdio");

    // Start the server
    if let Err(start_error) = server.start().await {
        eprintln!(
            "{}",
            start_error
                .rpc_error_message()
                .unwrap_or(&start_error.to_string())
        );
    }

    Ok(())
}
