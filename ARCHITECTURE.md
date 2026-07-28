# Architecture: web-access-mcp

## 🎯 Mission Statement
`web-access-mcp` is a high-precision, token-efficient Web Search & Documentation MCP Server designed specifically for AI Coding Agents. It aims to provide "Zero-cost, Zero-API-key" access to the live web, transforming complex, noisy HTML into clean, structured Markdown.

---

## 📐 Design Philosophy

1. **Token Defense (Context Efficiency)**: 
   - Automatic Markdown conversion of web pages.
   - Intelligent "Smart Token Cutting" to prevent LLM context overflow.
   - GitHub-specific optimization (Raw content extraction instead of scraping).
2. **Session Persistence (Stealth Browsing)**: 
   - Uses a shared browser session via `Arc<RwLock<BrowserState>>` to maintain cookies and User-Agent consistency, effectively bypassing simple bot detection.
3. **High-Performance Async**: 
   - Built on `tokio` and `chromiumoxide` for non-blocking, concurrent browsing tasks.
4. **Deterministic Output**: 
   - Eliminates the unpredictability of raw HTML by providing strictly structured JSON/Markdown.

---

## 🏗️ System Architecture

```mermaid
graph TD
    AI[AI Agent / LLM] -->|JSON-RPC (stdio)| MCP[MCP Layer]

    subgraph web-access-mcp [Rust Runtime]
        MCP -->|CallTool| HANDLER[Tool Handlers]

        subgraph STATE [Shared Thread-Safe State]
            HANDLER -->|Request/Update| CACHE[In-Memory TTL Cache]
            HANDLER -->|Lazy-launch on first use| BROWSER[Browser Controller: chromiumoxide]
        end

        subgraph ENGINE [Extraction Engine]
            HANDLER -->|HTTP GET, format=rss| SEARCHHTTP[reqwest: Bing RSS search feed]
            SEARCHHTTP -->|RSS XML| BING[quick-xml Parser]
            BROWSER -->|DOM Content| READ[Readability (Extraction)]
            READ -->|Clean HTML| MD[Markdown Converter]
            HANDLER -->|Raw Source via reqwest| GITHUB[GitHub Raw Fallback]
        end

        SMART[smart_search: search + concurrent fetch] --> SEARCHHTTP
        SMART --> BROWSER
    end

    MD -->|Clean Markdown| AI
    BING -->|Structured Results| AI
    GITHUB -->|Clean Code| AI
    SMART -->|Aggregated snippets + Markdown| AI
```

### 1. MCP Layer (`src/mcp.rs`, `src/transport.rs`, `src/main.rs`, `src/handlers.rs`)
Handles the lifecycle of the MCP server, listening on `stdin` and communicating via `stdout`. `src/main.rs` dispatches incoming JSON-RPC `initialize` / `tools/list` / `tools/call` requests to per-method handler functions; `src/handlers.rs` implements the actual tool dispatch (`list_tools_handler`, `call_tool_handler`) and per-tool argument parsing.

### 2. Shared State Management (`BrowserState`)
To ensure the agent can fetch multiple pages within the same session, the browser is not launched per-request but lazily on first `web_fetch`/`smart_search` call, then shared for the lifetime of the server process.
- **`BrowserState` (`Arc<RwLock<Option<Arc<Mutex<Browser>>>>>`)**: Lazily starts one shared `chromiumoxide` browser instance; the lock is only held briefly to open a new page, so concurrent fetches run in parallel rather than serializing on page load.
- **`SearchCache`**: A `DashMap`-backed in-memory cache with TTL (1 hour) to prevent redundant search requests and minimize latency.

### 3. Extraction & Transformation Engine
- **`web_search`**: Does *not* use the browser. Issues a plain `reqwest` HTTP GET against Bing's `format=rss` search feed -- a documented, purpose-built machine-readable output mode, not HTML/CSS-selector scraping -- and parses the response as structured XML with `quick-xml` into a list (Title, URL, Snippet), typically ~10 results per query.
- **`smart_search`**: Runs `web_search` internally, then concurrently calls `web_fetch` (via `join_all`) on the top `max_pages` results to attach extracted Markdown content to each item in a single tool call. Per-item fetch failures don't fail the whole call — that item just carries an `error` explanation instead of `content`.
- **`web_fetch`**:
    - **GitHub Logic**: Detects `github.com/.../blob/...` URLs and switches to a plain `reqwest` fetch of the raw file content, bypassing the browser entirely.
    - **PDF Logic** (`src/fetch/pdf.rs`): Sniffs `Content-Type` via a `HEAD` request (falling back to a `.pdf` URL-extension heuristic if `HEAD` is inconclusive); if it looks like a PDF, downloads it with `reqwest` and extracts text with `pdf-extract`, bypassing the browser entirely (Chromium's built-in PDF viewer renders a viewer UI, not extractable text). No OCR -- scanned/image-only PDFs aren't supported.
    - **Browser Fallback**: Otherwise opens a page in the shared `chromiumoxide` browser, waits for the DOM to stabilize, and checks for Cloudflare/CAPTCHA block pages before extracting content.
    - **Readability Pipeline**:
        1. **Extraction**: Uses `readabilityrs` to isolate the main article content (removing noise like ads, nav, and sidebars).
        2. **Conversion**: Uses `html-to-markdown-rs` to transform the clean HTML into high-fidelity Markdown.
    - **Smart Cutter**: Truncates output to a fixed 10,000-character limit (`MAX_CONTENT_LENGTH` in `src/fetch.rs`) at the nearest line boundary to ensure the response fits within the agent's context window.
- **Error Hints**: Every failure path (timeout, block, missing content, bad arguments, unknown tool) returns a message ending in a `Hint:` clause telling the calling model what to do next, instead of a bare error string.

---

## 🛠️ Technical Stack

| Component | Technology | Reason |
| :--- | :--- | :--- |
| **Runtime** | `tokio` | High-performance asynchronous I/O. |
| **Browser** | `chromiumoxide` | Reliable CDP-based control of real Chrome/Edge. |
| **Search Parsing** | `quick-xml` | Structured XML parsing for Bing's RSS search feed. |
| **Markdown** | `html-to-markdown-rs` | Robust HTML-to-Markdown conversion. |
| **Extraction** | `readabilityrs` | Noise removal (ads, nav, sidebars) via readability algorithm. |
| **PDF** | `pdf-extract` | Text extraction from PDF byte streams, no browser involved. |
| **Serialization**| `serde` | Industry standard for high-speed JSON processing. |
| **Networking** | `reqwest` | For lightweight, non-browser HTTP requests (search, GitHub raw). |
| **Error Handling** | `thiserror` + `anyhow` | `thiserror` models typed, agent-facing domain errors (`FetchError`, `SearchError`) in `fetch.rs`/`search.rs`; `anyhow` propagates them (plus top-level ad hoc errors) up through `handlers.rs`/`main.rs` to the MCP response. |
