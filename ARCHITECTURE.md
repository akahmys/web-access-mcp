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
Handles the lifecycle of the MCP server, listening on `stdin` and communicating via `stdout`. `src/main.rs` dispatches incoming JSON-RPC `initialize` / `tools/list` / `tools/call` requests to per-method handler functions; `src/handlers.rs` implements the actual tool dispatch (`list_tools_handler`, `call_tool_handler`) and per-tool argument parsing. `src/mcp.rs` only defines the generic JSON-RPC 2.0 transport envelope (`JsonRpcRequest`/`JsonRpcResponse`/`JsonRpcNotification`/`JsonRpcError`); the MCP-specific payload types (`InitializeResult`, `Implementation`, `ServerCapabilities`, `Tool`, `ToolInputSchema`, `CallToolResult`, `ContentBlock`, etc.) come from the `rust-mcp-schema` crate rather than being hand-rolled, so the server tracks the current MCP spec version (`ProtocolVersion::latest()`) and field naming without manual upkeep.

### 2. Shared State Management (`AppContext` & `BrowserState`)
To ensure clean state propagation, all shared thread-safe application resources are encapsulated inside `AppContext` (`src/context.rs`):
- **`AppContext`**: Holds `browser: BrowserState`, `search_provider: Arc<dyn SearchProvider>` (defaults to `BingSearchProvider`), `search_cache: SearchCache`, and `fetch_cache: FetchCache`.
- **`BrowserState` (`Arc<RwLock<Option<Arc<Mutex<Browser>>>>>`)**: Lazily starts one shared `chromiumoxide` browser instance. If a CDP connection failure or browser crash is detected during page creation, `BrowserState` triggers a **self-healing restart** to re-launch Chromium transparently without failing the session. Launch args include a User-Agent (`src/user_agent.rs`) and `--proxy-server` if `HTTP_PROXY`/`HTTPS_PROXY` is set.
- **`TtlCache<V>`** (`src/cache.rs`): A generic `DashMap`-backed in-memory cache with a fixed TTL per entry, shared by `SearchCache` (1 hour TTL) and `FetchCache` (10 minute TTL). Active eviction (`evict_expired`) is triggered on incoming messages to purge stale entries and prevent long-term memory growth.

### 3. Extraction & Transformation Engine
- **`SearchProvider` Trait**: Search execution is abstracted behind a pluggable `SearchProvider` trait (`src/search.rs`), allowing any search engine (Bing RSS, DuckDuckGo, Google, or custom API) to be plugged in seamlessly. `BingSearchProvider` is the default implementation, issuing a plain `reqwest` HTTP GET against Bing's `format=rss` search feed parsed with `quick-xml`.
- **`smart_search`**: Runs the configured `SearchProvider` internally, then concurrently calls `web_fetch` (via `join_all`) on the top `max_pages` results.
- **`batch_fetch`** (`src/batch_fetch.rs`): Concurrent fetch of a caller-supplied `urls` list (capped at `MAX_URLS = 10`), sharing `fetch_content_or_error` with `smart_search`.
- **`web_fetch`**:
    - **SSRF Protection** (`src/fetch/ssrf.rs`): Validates public URLs and checks DNS-resolved IPs against private/loopback/link-local/cloud-metadata ranges using `ipnet`.
    - **robots.txt** (`src/fetch/robots.rs`): Evaluates disallow rules via `texting_robots`. Fails open if missing/unreachable.
    - **Fast Paths** (`src/fetch/fast_path.rs`): Direct `reqwest` download for GitHub raw file URLs and PDF text extraction via `pdf-extract` (`src/fetch/pdf.rs`), both enforcing a strict **10MB download limit (`MAX_DOWNLOAD_SIZE`)** to prevent OOM memory exhaustion attacks.
    - **Page Actions** (`src/fetch/actions.rs`): Applies whitelisted browser interactions (`click`, `scroll`) before extraction.
    - **Browser Fallback** (`src/fetch/navigate.rs`): Opens a page in the shared Chromium browser with self-healing retry on transient failures.
    - **Readability Pipeline**: Extracts main article text via `readabilityrs` and converts clean HTML to Markdown via `html-to-markdown-rs`.
    - **Smart Cutter**: Safely truncates output to a 10,000-character limit at UTF-8 character and line boundaries (`truncate_content` in `src/fetch.rs`).
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
| **SSRF Protection** | `ipnet` | CIDR containment checks against DNS-resolved IPs before any fetch. |
| **robots.txt** | `texting_robots` | Parses and evaluates `robots.txt` disallow rules. |
| **Serialization**| `serde` | Industry standard for high-speed JSON processing. |
| **Networking** | `reqwest` | For lightweight, non-browser HTTP requests (search, GitHub raw, PDF, robots.txt); honors `HTTP_PROXY`/`HTTPS_PROXY` by default. |
| **Error Handling** | `thiserror` + `anyhow` | `thiserror` models typed, agent-facing domain errors (`FetchError`, `SearchError`) in `fetch.rs`/`search.rs`; `anyhow` propagates them (plus top-level ad hoc errors) up through `handlers.rs`/`main.rs` to the MCP response. |
| **MCP Protocol Types** | `rust-mcp-schema` | serde-based, async-runtime-agnostic type definitions for MCP payloads (`InitializeResult`, `Tool`, `CallToolResult`, `ContentBlock`, ...); tracks the current MCP spec version instead of hand-rolled structs. |
