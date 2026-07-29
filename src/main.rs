#![deny(clippy::pedantic)]

mod fetch;
mod error;
mod mcp;
mod transport;
mod browser;
mod search;
mod smart_search;
mod batch_fetch;
mod handlers;
mod cache;
mod user_agent;

use tracing::{info, error};
use error::AppResult;
use crate::mcp::{
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError, ErrorDetails,
};
use crate::transport::StdioTransport;
use crate::browser::BrowserState;
use crate::handlers::{call_tool_handler, list_tools_handler};
use rust_mcp_schema::{
    CallToolRequestParams, CallToolResult, ContentBlock, Implementation, InitializeResult,
    ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools, TextContent,
};
use serde_json::{json, Value};
use std::time::Duration;
use crate::search::SearchCache;
use crate::fetch::FetchCache;

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
    let search_cache = SearchCache::new(Duration::from_hours(1));
    let fetch_cache = FetchCache::new(Duration::from_mins(10));

    while let Some(message) = transport.read_message().await? {
        if let Err(e) = handle_message(&mut transport, &browser_state, &search_cache, &fetch_cache, message).await {
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
    fetch_cache: &FetchCache,
    message: JsonRpcMessage,
) -> AppResult<()> {
    let req = match message {
        JsonRpcMessage::Request(req) => req,
        JsonRpcMessage::Response(res) => {
            info!("Received response: {:?}", res);
            return Ok(());
        }
        JsonRpcMessage::Error(err) => {
            info!("Received error message: {:?}", err);
            return Ok(());
        }
        JsonRpcMessage::Notification(notif) => {
            info!("Received notification: {:?}", notif.method);
            return Ok(());
        }
    };

    match req.method.as_str() {
        "initialize" => handle_initialize(transport, req.id).await,
        "ping" => handle_ping(transport, req.id).await,
        "tools/list" | "list_tools" => handle_tools_list(transport, req.id).await,
        "tools/call" | "call_tool" => handle_tools_call(transport, browser_state, search_cache, fetch_cache, req).await,
        _ => handle_unknown_method(transport, &req.method, req.id).await,
    }
}

/// Serializes `result` into a JSON-RPC success response and writes it.
/// Shared by every method handler below to avoid repeating the
/// response-envelope boilerplate at each call site.
async fn write_response<T: serde::Serialize>(
    transport: &mut StdioTransport,
    result: T,
    id: Value,
) -> AppResult<()> {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: serde_json::to_value(result)?,
        id,
    };
    transport.write_message(&JsonRpcMessage::Response(response)).await
}

async fn handle_initialize(transport: &mut StdioTransport, id: Value) -> AppResult<()> {
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        instructions: None,
        meta: None,
        protocol_version: ProtocolVersion::latest().to_string(),
        server_info: Implementation {
            name: "web-access-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: None,
            description: None,
            icons: Vec::new(),
            website_url: None,
        },
    };
    write_response(transport, result, id).await
}

async fn handle_ping(transport: &mut StdioTransport, id: Value) -> AppResult<()> {
    write_response(transport, json!({}), id).await
}

async fn handle_tools_list(transport: &mut StdioTransport, id: Value) -> AppResult<()> {
    let result = list_tools_handler().await?;
    write_response(transport, result, id).await
}

async fn handle_tools_call(
    transport: &mut StdioTransport,
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    fetch_cache: &FetchCache,
    req: JsonRpcRequest,
) -> AppResult<()> {
    let call_result: AppResult<CallToolResult> = match &req.params {
        Some(params) => match serde_json::from_value::<CallToolRequestParams>(params.clone()) {
            Ok(call_req) => {
                let arguments = Value::Object(call_req.arguments.unwrap_or_default());
                call_tool_handler(browser_state, search_cache, fetch_cache, &call_req.name, &arguments).await
            }
            Err(e) => Err(anyhow::anyhow!(
                "Invalid CallToolRequest: {e}. Hint: 'arguments' must be a JSON object matching the target tool's inputSchema (see tools/list) -- check for missing fields or wrong types."
            )),
        },
        None => Err(anyhow::anyhow!(
            "Missing params for call_tool. Hint: include a 'params' object with 'name' (the tool name) and 'arguments' (matching that tool's inputSchema)."
        )),
    };

    // A failed tool call (bad URL, timeout, blocked page, bad arguments,
    // etc.) is reported to the client as a normal MCP tool error response,
    // not a fatal transport error -- one failed web_fetch/search shouldn't
    // take down the whole server for the rest of the session.
    let result = call_result.unwrap_or_else(|e| error_to_call_result(&e));
    write_response(transport, result, req.id).await
}

/// Wraps a tool-call failure into an MCP `isError: true` result. `{:#}`
/// renders the full anyhow context chain (not just the outermost message),
/// so any underlying cause and hint text baked into it reach the model.
fn error_to_call_result(e: &anyhow::Error) -> CallToolResult {
    error!("CallTool error: {:?}", e);
    CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent::new(format!("{e:#}"), None, None))],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

async fn handle_unknown_method(transport: &mut StdioTransport, method: &str, id: Value) -> AppResult<()> {
    error!("Unknown method: {}", method);
    let response = JsonRpcError {
        jsonrpc: "2.0".to_string(),
        error: ErrorDetails {
            code: -32601,
            message: format!("Method not found: {method}"),
            data: None,
        },
        id,
    };
    transport.write_message(&JsonRpcMessage::Error(response)).await
}
