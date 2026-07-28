use serde_json::{json, Value};
use tracing::{error, info};

use crate::browser::BrowserState;
use crate::error::AppResult;
use crate::fetch::{fetch_url, FetchCache};
use crate::mcp::{CallToolResult, ListToolsResult, McpContent, McpTool};
use crate::search::{perform_web_search, SearchCache};
use crate::smart_search::perform_smart_search;

pub async fn list_tools_handler() -> AppResult<ListToolsResult> {
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
            name: "web_search".to_string(),
            description: "Search the web (Bing) and return raw search result snippets".to_string(),
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

pub async fn call_tool_handler(
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    fetch_cache: &FetchCache,
    name: &str,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    info!("Calling tool: {} with arguments: {:?}", name, arguments);

    match name {
        "smart_search" => call_smart_search(browser_state, search_cache, fetch_cache, arguments).await,
        "web_search" => call_web_search(browser_state, search_cache, arguments).await,
        "web_fetch" => call_web_fetch(browser_state, fetch_cache, arguments).await,
        _ => {
            error!("Unknown tool: {}", name);
            Err(anyhow::anyhow!(
                "Unknown tool: '{name}'. Hint: call tools/list to see available tools; valid names are 'smart_search', 'web_search', and 'web_fetch'."
            ))
        }
    }
}

fn text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![McpContent {
            content_type: "text".to_string(),
            text: Some(text),
            image: None,
        }],
        is_error: Some(false),
    }
}

async fn call_smart_search(
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    fetch_cache: &FetchCache,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    let query = arguments
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "Missing 'query' argument in smart_search. Hint: pass a non-empty 'query' string describing what to search for."
        ))?;

    let max_pages = arguments
        .get("max_pages")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(3)
        .min(5);

    let results = perform_smart_search(browser_state, search_cache, fetch_cache, query, max_pages).await?;
    Ok(text_result(serde_json::to_string_pretty(&results)?))
}

async fn call_web_search(
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    let query = arguments
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "Missing 'query' argument in web_search. Hint: pass a non-empty 'query' string describing what to search for."
        ))?;

    let results = perform_web_search(browser_state, search_cache, query).await?;
    Ok(text_result(serde_json::to_string(&results)?))
}

async fn call_web_fetch(browser_state: &BrowserState, fetch_cache: &FetchCache, arguments: &Value) -> AppResult<CallToolResult> {
    let url = arguments
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "Missing 'url' argument in web_fetch. Hint: pass an absolute URL including the scheme, e.g. 'https://example.com/page'."
        ))?;

    let result = fetch_url(browser_state, fetch_cache, url).await?;
    Ok(text_result(serde_json::to_string(&result)?))
}
