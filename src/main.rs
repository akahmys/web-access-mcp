mod error;
mod mcp;
mod transport;

use tracing::{info, error};
use error::AppResult;
use crate::mcp::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, ListToolsResult, McpTool, CallToolResult, McpContent};
use crate::transport::StdioTransport;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

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

    while let Some(message) = transport.read_message().await? {
        if let Err(e) = handle_message(&mut transport, message).await {
            error!("Error handling message: {:?}", e);
            break;
        }
    }
    Ok(())
}

async fn handle_message(
    transport: &mut StdioTransport,
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
                                    match call_tool_handler(&call_req.name, &call_req.arguments).await {
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
    name: &str,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    info!("Calling tool: {} with arguments: {:?}", name, arguments);

    Ok(CallToolResult {
        content: vec![McpContent {
            content_type: "text".to_string(),
            text: Some(format!("Tool '{}' called successfully with arguments: {}", name, arguments)),
            image: None,
        }],
        is_error: Some(false),
    })
}
