mod fetch;
mod error;
mod mcp;
mod transport;
mod browser;
mod search;
mod smart_search;

use tracing::{info, error};
use error::AppResult;
use crate::mcp::{
    JsonRpcMessage, JsonRpcResponse, JsonRpcError, ErrorDetails, ListToolsResult, McpTool,
    CallToolResult, McpContent, InitializeResult, ServerCapabilities, ImplementationInfo,
};
use crate::transport::StdioTransport;
use crate::browser::BrowserState;
use serde_json::{json, Value};
use std::time::Duration;
use crate::search::SearchCache;
use crate::fetch::fetch_url;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging to stderr so stdout is reserved for JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    info!("Starting MCP Web Agent...");

    // Basic logic to test if everything is working
    if let Err(e) = run().await {
        error!("Application error: {:?}", e);
        return Err(e);
    }

    info!("Application finished successfully.");
    Ok(())
}

async fn run() -> AppResult<()> {
    info!("Running core logic...");
    let mut transport = StdioTransport::new();
    let browser_state = BrowserState::new();
    let search_cache = SearchCache::new(Duration::from_secs(3600));

    while let Some(message) = transport.read_message().await? {
        if let Err(e) = handle_message(&mut transport, &browser_state, &search_cache, message).await {
            error!("Error handling message: {:?}", e);
            break;
        }
    }
    
    // Graceful shutdown
    if let Err(e) = browser_state.stop().await {
        error!("Error stopping browser: {:?}", e);
    }

    Ok(())
}

async fn handle_message(
    transport: &mut StdioTransport,
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    message: JsonRpcMessage,
) -> AppResult<()> {
    match message {
        JsonRpcMessage::Request(req) => {
            match req.method.as_str() {
                "initialize" => {
                    let result = InitializeResult {
                        protocol_version: "2024-11-05".to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(json!({})),
                        },
                        server_info: ImplementationInfo {
                            name: "web-access-mcp".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        },
                    };
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: serde_json::to_value(result)?,
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Response(response)).await?;
                }
                "ping" => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: json!({}),
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Response(response)).await?;
                }
                "tools/list" | "list_tools" => {
                    let result = list_tools_handler().await?;
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: serde_json::to_value(result)?,
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Response(response)).await?;
                }
                "tools/call" | "call_tool" => {
                    let call_result: AppResult<CallToolResult> = match &req.params {
                        Some(params) => {
                            match serde_json::from_value::<crate::mcp::CallToolRequest>(params.clone()) {
                                Ok(call_req) => {
                                    call_tool_handler(browser_state, search_cache, &call_req.name, &call_req.arguments).await
                                }
                                Err(e) => Err(anyhow::anyhow!("Invalid CallToolRequest: {}", e)),
                            }
                        }
                        None => Err(anyhow::anyhow!("Missing params for call_tool")),
                    };

                    // A failed tool call (bad URL, timeout, blocked page, bad
                    // arguments, etc.) is reported to the client as a normal
                    // MCP tool error response, not a fatal transport error --
                    // one failed web_fetch/search shouldn't take down the
                    // whole server for the rest of the session.
                    let result = match call_result {
                        Ok(res) => res,
                        Err(e) => {
                            error!("CallTool error: {:?}", e);
                            CallToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: Some(e.to_string()),
                                    image: None,
                                }],
                                is_error: Some(true),
                            }
                        }
                    };

                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: serde_json::to_value(result)?,
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Response(response)).await?;
                }
                _ => {
                    error!("Unknown method: {}", req.method);
                    let response = JsonRpcError {
                        jsonrpc: "2.0".to_string(),
                        error: ErrorDetails {
                            code: -32601,
                            message: format!("Method not found: {}", req.method),
                            data: None,
                        },
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Error(response)).await?;
                }
            }
        }
        JsonRpcMessage::Response(res) => {
            info!("Received response: {:?}", res);
        }
        JsonRpcMessage::Error(err) => {
            info!("Received error message: {:?}", err);
        }
        JsonRpcMessage::Notification(notif) => {
            info!("Received notification: {:?}", notif.method);
        }
    }
    Ok(())
}

async fn list_tools_handler() -> AppResult<ListToolsResult> {
    let tools = vec![
        McpTool {
            name: "smart_search".to_string(),
            description: "Perform web search and automatically fetch extracted Markdown content from top result pages in a single call.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query" },
                    "max_pages": { "type": "integer", "description": "Number of top pages to fetch content from (default: 3, max: 5)" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "google_search".to_string(),
            description: "Search Google for queries and return raw search result snippets".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "web_fetch".to_string(),
            description: "Fetch content from a single URL and convert to Markdown".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" }
                },
                "required": ["url"]
            }),
        },
    ];

    Ok(ListToolsResult { tools })
}

async fn call_tool_handler(
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    name: &str,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    info!("Calling tool: {} with arguments: {:?}", name, arguments);

    match name {
        "smart_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'query' argument in smart_search"))?;

            let max_pages = arguments
                .get("max_pages")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(3)
                .min(5);

            match crate::smart_search::perform_smart_search(browser_state, search_cache, query, max_pages).await {
                Ok(results) => Ok(CallToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: Some(serde_json::to_string_pretty(&results)?),
                        image: None,
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Err(e),
            }
        }
        "google_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'query' argument in google_search"))?;

            match crate::search::perform_google_search(browser_state, search_cache, query).await {
                Ok(results) => Ok(CallToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: Some(serde_json::to_string(&results)?),
                        image: None,
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Err(e),
            }
        }
        "web_fetch" => {
            let url = arguments
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'url' argument in web_fetch"))?;

            match fetch_url(browser_state, url).await {
                Ok(result) => Ok(CallToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: Some(serde_json::to_string(&result)?),
                        image: None,
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Err(e),
            }
        }
        _ => {
            error!("Unknown tool: {}", name);
            Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
