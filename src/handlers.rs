use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use tracing::{error, info};

use crate::batch_fetch::{self, MAX_URLS};
use crate::context::AppContext;
use crate::error::AppResult;
use crate::fetch::{fetch_url, PageAction};
use crate::search::perform_web_search;
use crate::smart_search::perform_smart_search;
use rust_mcp_schema::{CallToolResult, ContentBlock, ListToolsResult, TextContent, Tool, ToolInputSchema};

pub async fn list_tools_handler() -> AppResult<ListToolsResult> {
    let tools = vec![
        smart_search_tool(),
        web_search_tool(),
        web_fetch_tool(),
        batch_fetch_tool(),
    ];

    Ok(ListToolsResult { meta: None, next_cursor: None, tools })
}

/// Builds a `Tool` from just the parts that vary per tool, filling in the
/// rest (annotations, execution hints, icons, output schema, title) with
/// their empty/`None` defaults -- none of our tools use those fields.
fn tool(name: &str, description: &str, input_schema: ToolInputSchema) -> Tool {
    Tool {
        name: name.to_string(),
        description: Some(description.to_string()),
        input_schema,
        annotations: None,
        execution: None,
        icons: Vec::new(),
        meta: None,
        output_schema: None,
        title: None,
    }
}

/// Converts a list of `(property name, JSON Schema object)` pairs into the
/// `BTreeMap<String, Map<String, Value>>` shape `ToolInputSchema::properties`
/// requires.
fn schema_properties(fields: &[(&str, Value)]) -> BTreeMap<String, Map<String, Value>> {
    fields
        .iter()
        .filter_map(|(name, schema)| schema.as_object().map(|obj| ((*name).to_string(), obj.clone())))
        .collect()
}

fn smart_search_tool() -> Tool {
    let input_schema = ToolInputSchema::new(
        vec!["query".to_string()],
        Some(schema_properties(&[
            ("query", json!({ "type": "string", "description": "The search query" })),
            ("max_pages", json!({ "type": "integer", "description": "Number of top pages to fetch content from (default: 3, max: 5)" })),
        ])),
        None,
    );
    tool(
        "smart_search",
        "Perform web search and automatically fetch extracted Markdown content from top result pages in a single call.",
        input_schema,
    )
}

fn web_search_tool() -> Tool {
    let input_schema = ToolInputSchema::new(
        vec!["query".to_string()],
        Some(schema_properties(&[("query", json!({ "type": "string" }))])),
        None,
    );
    tool("web_search", "Search the web (Bing) and return raw search result snippets", input_schema)
}

fn web_fetch_tool() -> Tool {
    let actions_description = "Optional ordered browser actions to apply before extracting content, e.g. [{\"type\":\"click\",\"selector\":\"#load-more\"},{\"type\":\"scroll\",\"target\":\"bottom\"}]. Supported: 'click' (needs 'selector'), 'scroll' (needs 'target': 'top'|'bottom'). No form-fill/login support. Using actions disables result caching for that call.";
    let input_schema = ToolInputSchema::new(
        vec!["url".to_string()],
        Some(schema_properties(&[
            ("url", json!({ "type": "string" })),
            ("actions", json!({ "type": "array", "description": actions_description, "items": { "type": "object" } })),
        ])),
        None,
    );
    tool("web_fetch", "Fetch content from a single URL and convert to Markdown", input_schema)
}

fn batch_fetch_tool() -> Tool {
    let input_schema = ToolInputSchema::new(
        vec!["urls".to_string()],
        Some(schema_properties(&[(
            "urls",
            json!({ "type": "array", "items": { "type": "string" }, "description": format!("URLs to fetch (max {MAX_URLS})") }),
        )])),
        None,
    );
    tool(
        "batch_fetch",
        "Fetch multiple URLs concurrently and convert each to Markdown in a single call. For when you already have the URLs; use smart_search instead if you need to search first.",
        input_schema,
    )
}

pub async fn call_tool_handler(
    ctx: &AppContext,
    name: &str,
    arguments: &Value,
) -> AppResult<CallToolResult> {
    info!("Calling tool: {} with arguments: {:?}", name, arguments);

    match name {
        "smart_search" => call_smart_search(ctx, arguments).await,
        "web_search" => call_web_search(ctx, arguments).await,
        "web_fetch" => call_web_fetch(ctx, arguments).await,
        "batch_fetch" => call_batch_fetch(ctx, arguments).await,
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
        content: vec![ContentBlock::TextContent(TextContent::new(text, None, None))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    }
}

fn get_str_arg<'a>(args: &'a Value, key: &str, tool_name: &str, hint: &str) -> AppResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Missing '{key}' argument in {tool_name}. Hint: {hint}"))
}

fn get_usize_arg(args: &Value, key: &str, default: usize, max: usize) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(default)
        .min(max)
}

async fn call_smart_search(ctx: &AppContext, arguments: &Value) -> AppResult<CallToolResult> {
    let query = get_str_arg(
        arguments,
        "query",
        "smart_search",
        "pass a non-empty 'query' string describing what to search for.",
    )?;

    let max_pages = get_usize_arg(arguments, "max_pages", 3, 5);

    let results = perform_smart_search(ctx, query, max_pages).await?;
    Ok(text_result(serde_json::to_string_pretty(&results)?))
}

async fn call_web_search(ctx: &AppContext, arguments: &Value) -> AppResult<CallToolResult> {
    let query = get_str_arg(
        arguments,
        "query",
        "web_search",
        "pass a non-empty 'query' string describing what to search for.",
    )?;

    let results = perform_web_search(ctx.search_provider.as_ref(), &ctx.search_cache, query).await?;
    Ok(text_result(serde_json::to_string(&results)?))
}

async fn call_web_fetch(ctx: &AppContext, arguments: &Value) -> AppResult<CallToolResult> {
    let url = get_str_arg(
        arguments,
        "url",
        "web_fetch",
        "pass an absolute URL including the scheme, e.g. 'https://example.com/page'.",
    )?;

    let actions: Vec<PageAction> = match arguments.get("actions") {
        Some(value) => serde_json::from_value(value.clone()).map_err(|e| anyhow::anyhow!(
            "Invalid 'actions' argument in web_fetch: {e}. Hint: each action needs a 'type' of 'click' (with 'selector') or 'scroll' (with 'target': 'top' or 'bottom')."
        ))?,
        None => Vec::new(),
    };

    let result = fetch_url(&ctx.browser, &ctx.fetch_cache, url, &actions).await?;
    Ok(text_result(serde_json::to_string(&result)?))
}

async fn call_batch_fetch(ctx: &AppContext, arguments: &Value) -> AppResult<CallToolResult> {
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

    let results = batch_fetch::fetch_many(ctx, &urls).await;
    Ok(text_result(serde_json::to_string(&results)?))
}
