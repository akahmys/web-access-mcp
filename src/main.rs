mod fetch;
mod error;
mod mcp;
mod transport;
mod browser;
mod search;

use tracing::{info, error};
use error::AppResult;
use crate::mcp::{JsonRpcMessage, JsonRpcResponse, ListToolsResult, McpTool, CallToolResult, McpContent};
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

    // Initialize browser
    if let Err(e) = browser_state.start().await {
        error!("Failed to start browser: {:?}", e);
        // We might want to continue even if browser fails if we want to allow non-browser tools, 
        // but for now, let's treat it as fatal or just log it.
        error!("Continuing without browser support for now.");
    }

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
                "list_tools" => {
                    let result = list_tools_handler().await?;
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: serde_json::to_value(result)?,
                        id: req.id,
                    };
                    transport.write_message(&JsonRpcMessage::Response(response)).await?;
                }
                "call_tool" => {
                    let (result, id) = match &req.params {
                        Some(params) => {
                            match serde_json::from_value::<crate::mcp::CallToolRequest>(params.clone()) {
                                Ok(call_req) => {
                                    match call_tool_handler(browser_state, search_cache, &call_req.name, &call_req.arguments).await {
                                        Ok(res) => (Ok(serde_json::to_value(res).unwrap_or_default()), req.id.clone()),
                                        Err(e) => (Err(e), req.id.clone()),
                                    }
                                }
                                Err(e) => (Err(anyhow::anyhow!("Invalid CallToolRequest: {}", e)), req.id.clone()),
                            }
                        }
                        None => (Err(anyhow::anyhow!("Missing params for call_tool")), req.id.clone()),
                    };

                    match result {
                        Ok(res_val) => {
                            let response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: res_val,
                                id,
                            };
                            transport.write_message(&JsonRpcMessage::Response(response)).await?;
                        }
                        Err(e) => {
                            error!("CallTool error: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                _ => {
                    error!("Unknown method: {}", req.method);
                }
            }
        }
        JsonRpcMessage::Response(res) => {
            info!("Received response: {:?}", res);
        }
        JsonRpcMessage::Notification(notif) => {
            info!("Received notification: {:?}", notif);
        }
    }
    Ok(())
}

async fn list_tools_handler() -> AppResult<ListToolsResult> {
    let tools = vec![
        McpTool {
            name: "google_search".to_string(),
            description: "Search Google for queries".to_string(),
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
            description: "Fetch content from a URL".to_string(),
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

            let browser_arc = browser_state
                .get_browser()
                .await
                .ok_or_else(|| anyhow::anyhow!("Browser is not running"))?;

            let browser = browser_arc.lock().await;

            match fetch_url(&browser, url).await {
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
