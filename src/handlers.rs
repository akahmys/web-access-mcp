use serde_json::{json, Value};
use tracing::{error, info};

use crate::batch_fetch::{self, MAX_URLS};
use crate::browser::BrowserState;
use crate::error::AppResult;
use crate::fetch::{fetch_url, FetchCache, PageAction};
use crate::mcp::{CallToolResult, ListToolsResult, McpContent, McpTool};
use crate::search::{perform_web_search, SearchCache};
use crate::smart_search::perform_smart_search;

pub async fn list_tools_handler() -> AppResult<ListToolsResult> {
    let tools = vec![
        smart_search_tool(),
        web_search_tool(),
        web_fetch_tool(),
        batch_fetch_tool(),
    ];

    Ok(ListToolsResult { tools })
}

fn smart_search_tool() -> McpTool {
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
    }
}

fn web_search_tool() -> McpTool {
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
    }
}

fn web_fetch_tool() -> McpTool {
    McpTool {
        name: "web_fetch".to_string(),
        description: "Fetch content from a single URL and convert to Markdown".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" },
                "actions": {
                    "type": "array",
                    "description": "Optional ordered browser actions to apply before extracting content, e.g. [{\"type\":\"click\",\"selector\":\"#load-more\"},{\"type\":\"scroll\",\"target\":\"bottom\"}]. Supported: 'click' (needs 'selector'), 'scroll' (needs 'target': 'top'|'bottom'). No form-fill/login support. Using actions disables result caching for that call.",
                    "items": { "type": "object" }
                }
            },
            "required": ["url"]
        }),
    }
}

fn batch_fetch_tool() -> McpTool {
    McpTool {
        name: "batch_fetch".to_string(),
        description: "Fetch multiple URLs concurrently and convert each to Markdown in a single call. For when you already have the URLs; use smart_search instead if you need to search first.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "urls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": format!("URLs to fetch (max {MAX_URLS})")
                }
            },
            "required": ["urls"]
        }),
    }
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
        "batch_fetch" => call_batch_fetch(browser_state, fetch_cache, arguments).await,
        _ => {
            error!("Unknown tool: {}", name);
            Err(anyhow::anyhow!(
                "Unknown tool: '{name}'. Hint: call tools/list to see available tools; valid names are 'smart_search', 'web_search', 'web_fetch', and 'batch_fetch'."
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

    let actions: Vec<PageAction> = match arguments.get("actions") {
        Some(value) => serde_json::from_value(value.clone()).map_err(|e| anyhow::anyhow!(
            "Invalid 'actions' argument in web_fetch: {e}. Hint: each action needs a 'type' of 'click' (with 'selector') or 'scroll' (with 'target': 'top' or 'bottom')."
        ))?,
        None => Vec::new(),
    };

    let result = fetch_url(browser_state, fetch_cache, url, &actions).await?;
    Ok(text_result(serde_json::to_string(&result)?))
}

async fn call_batch_fetch(browser_state: &BrowserState, fetch_cache: &FetchCache, arguments: &Value) -> AppResult<CallToolResult> {
    let urls: Vec<String> = arguments
        .get("urls")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!(
            "Missing 'urls' argument in batch_fetch. Hint: pass a non-empty array of absolute URLs, e.g. [\"https://example.com/a\", \"https://example.com/b\"]."
        ))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    if urls.is_empty() {
        return Err(anyhow::anyhow!(
            "'urls' argument in batch_fetch is empty. Hint: pass at least one absolute URL."
        ));
    }

    let results = batch_fetch::fetch_many(browser_state, fetch_cache, &urls).await;
    Ok(text_result(serde_json::to_string(&results)?))
}
