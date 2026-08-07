# web-access-mcp

`web-access-mcp` is a Model Context Protocol (MCP) server written in Rust that provides web search and page content extraction tools for AI agents.

## Overview

* **HTML to Markdown Conversion:** Transforms HTML content into structured Markdown using Readability and Markdown conversion libraries.
* **Web Search:** Queries Bing's RSS search feed over HTTP without launching a browser instance.
* **Browser Automation:** Uses `chromiumoxide` to manage a headless Chromium instance for pages requiring JavaScript execution.
* **Security Controls:** Validates URLs against internal/private IP ranges (SSRF protection) and evaluates `robots.txt` disallow rules.
* **Content Safety Limits:** Enforces a 10MB download limit on HTTP/PDF fetches and truncates output content (10,000 characters for `web_fetch`, 2,500 characters per item for `smart_search` and `batch_fetch`).

## Available Tools

### `smart_search`
Performs a web search and fetches extracted Markdown content from top result pages in a single call.
* **Input:** `query` (string, required), `max_pages` (integer, optional, default: 3, max: 5)
* **Output:** JSON object containing search snippets and extracted page Markdown (truncated to 2,500 characters per item). If a page fetch fails, an `error` field is returned for that item instead of `content`.

### `web_search`
Queries Bing's RSS search feed and returns structured search results.
* **Input:** `query` (string, required)
* **Output:** JSON array of search results (title, URL, snippet). Results are cached in memory (1 hour TTL).

### `web_fetch`
Fetches content from a single URL and converts it to Markdown.
* **Input:** `url` (string, required), `actions` (array, optional) — ordered browser actions (`click`, `scroll`) executed before content extraction.
* **Output:** JSON object (`{"title": "...", "content": "..."}`) containing page title and extracted Markdown text truncated to a maximum of 10,000 characters at UTF-8 character and line boundaries.
* **Behavior & Features:**
  * **SSRF Protection:** Resolves host IP via DNS and rejects loopback, RFC1918 private, link-local, and cloud metadata (`169.254.169.254`) addresses.
  * **robots.txt Compliance:** Fetches and parses `robots.txt` disallow rules before request. Fails open on missing or unparseable rules. Can be disabled via `WEB_FETCH_IGNORE_ROBOTS=1`.
  * **Fast Paths:** Direct `reqwest` download for GitHub raw file URLs (`github.com/.../blob/...`) and PDF text extraction via `pdf-extract`. Both enforce a 10MB download size cap.
  * **Browser Navigation:** Launches headless Chromium via CDP when rendering is required. Includes self-healing auto-restart on CDP connection failure.
  * **Caching:** Results are cached per-URL for 10 minutes. Calls with custom `actions` bypass the cache.

### `batch_fetch`
Fetches multiple specified URLs concurrently without a search step.
* **Input:** `urls` (array of strings, required, max: 10)
* **Output:** JSON array of extracted content items (`url`, `content` truncated to 2,500 characters per item, or `error`).

## Error Handling

Failed tool calls return an MCP error response (`isError: true`) containing error details and a `Hint:` clause specifying potential next actions. Errors are isolated per request and do not interrupt the MCP server session.

## Installation & Setup

### Prerequisites
* Rust toolchain (for building from source)
* Chromium, Google Chrome, or Microsoft Edge installed on the host system

### Building from Source
```bash
git clone https://github.com/akahmys/web-access-mcp.git
cd web-access-mcp
cargo build --release
```
The compiled binary is output to `target/release/web-access-mcp`.

### Development & Pre-Commit Hook
The local git pre-commit hook scans staged changes for secrets, personal names, and absolute paths via `betterleaks`:
```bash
# Configure pre-commit hook
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit # if creating manually
# Hook content:
# betterleaks git --pre-commit
```


### Configuration
Add `web-access-mcp` to your client's MCP configuration (e.g., `mcp_config.json`):

```json
{
  "mcpServers": {
    "web-access-mcp": {
      "command": "/path/to/target/release/web-access-mcp",
      "env": {
        "CHROME_PATH": "/path/to/chrome" 
      }
    }
  }
}
```

### Environment Variables
* `CHROME_PATH`: Optional path to the Chromium/Chrome executable.
* `HTTP_PROXY` / `HTTPS_PROXY`: Honored by `reqwest` and passed to Chromium via `--proxy-server`.
* `WEB_FETCH_IGNORE_ROBOTS`: Set to any non-empty value to bypass `robots.txt` checks.

## Technical Stack

* **Language:** Rust
* **Async Runtime:** `tokio`
* **Browser Automation:** `chromiumoxide` (CDP with self-healing recovery)
* **Search Abstraction:** `SearchProvider` trait (`BingSearchProvider` implementation)
* **HTML Parsing & Markdown:** `readabilityrs`, `html-to-markdown-rs`
* **PDF Processing:** `pdf-extract`
* **Security & Network Boundaries:** `ipnet`, `texting_robots`
* **MCP Protocol:** `rust-mcp-schema`

## License

This project is licensed under the MIT License.
