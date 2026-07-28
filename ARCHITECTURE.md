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
- **`BrowserState` (`Arc<RwLock<Option<Arc<Mutex<Browser>>>>>`)**: Lazily starts one shared `chromiumoxide` browser instance; the lock is only held briefly to open a new page, so concurrent fetches run in parallel rather than serializing on page load. Launch args include a User-Agent (`src/user_agent.rs`: picked once at random from a small pool of realistic strings, then reused for the process's lifetime -- varies which UA a given server instance presents while staying internally consistent for a session) and, if `HTTP_PROXY`/`HTTPS_PROXY` is set, `--proxy-server` (Chromium doesn't read those env vars itself, unlike `reqwest`, which honors them for every HTTP client -- search, PDF, GitHub raw, robots.txt -- by default with no extra code).
- **`TtlCache<V>`** (`src/cache.rs`): A generic `DashMap`-backed in-memory cache with a fixed TTL per entry, shared by two instantiations -- `SearchCache` (`TtlCache<Vec<SearchResult>>`, 1 hour TTL, keyed by query) and `FetchCache` (`TtlCache<WebFetchResult>`, 10 minute TTL, keyed by URL). The shorter fetch TTL reflects that page content changes more often than search rankings. `fetch_url` checks/populates the cache before any GitHub/PDF/browser dispatch, so a repeat fetch of the same URL (including from `smart_search`'s per-item fetches) skips the browser entirely.

### 3. Extraction & Transformation Engine
- **`web_search`**: Does *not* use the browser. Issues a plain `reqwest` HTTP GET against Bing's `format=rss` search feed -- a documented, purpose-built machine-readable output mode, not HTML/CSS-selector scraping -- and parses the response as structured XML with `quick-xml` into a list (Title, URL, Snippet), typically ~10 results per query.
- **`smart_search`**: Runs `web_search` internally, then concurrently calls `web_fetch` (via `join_all`) on the top `max_pages` results to attach extracted Markdown content to each item in a single tool call. Per-item fetch failures don't fail the whole call — that item just carries an `error` explanation instead of `content`.
- **`batch_fetch`** (`src/batch_fetch.rs`): Same "concurrent fetch, per-item error isolation" shape as `smart_search`, minus the search step -- takes a caller-supplied `urls` list (capped at `MAX_URLS = 10`) instead of deriving it from a query. Both share the `fetch_content_or_error` helper (`src/fetch/multi.rs`) rather than duplicating the fetch-then-truncate-then-stringify-errors logic.
- **`web_fetch`**:
    - **SSRF Protection** (`src/fetch/ssrf.rs`): The very first step for every `url`, before GitHub/PDF/browser dispatch. Parses the URL, rejects non-http(s) schemes, resolves the host via DNS, and checks *every* resolved `IpAddr` (including the IPv4 address inside an IPv4-mapped IPv6 literal) against loopback/RFC1918-private/CGNAT/link-local/IPv6-unique-local ranges using `ipnet`. Resolving before checking (rather than pattern-matching the literal URL) closes the DNS-rebinding gap, and link-local coverage includes the `169.254.169.254` cloud metadata endpoint.
    - **robots.txt** (`src/fetch/robots.rs`): Checked right after SSRF validation, before dispatch. Fetches `robots.txt` for the URL's host via `texting_robots`, evaluated against the fixed `"web-access-mcp"` user-agent string; fails *open* (allows the fetch) on any missing/unreachable/unparseable `robots.txt`, matching standard crawler convention. `WEB_FETCH_IGNORE_ROBOTS` env var skips the check entirely.
    - **Fast Paths** (`src/fetch/fast_path.rs`): GitHub -- detects `github.com/.../blob/...` URLs and switches to a plain `reqwest` fetch of the raw file content; and PDF (`src/fetch/pdf.rs`) -- sniffs `Content-Type` via a `HEAD` request (falling back to a `.pdf` URL-extension heuristic if `HEAD` is inconclusive) and, on a match, downloads with `reqwest` and extracts text with `pdf-extract`. Both bypass the browser entirely, and both are skipped when `actions` are given (neither has anything for an action to click/scroll). No OCR -- scanned/image-only PDFs aren't supported.
    - **Page Actions** (`src/fetch/actions.rs`): An optional, ordered list of whitelisted browser interactions -- `click` (by CSS selector) or `scroll` (`top`/`bottom`) -- applied after page load and before extraction, each followed by a short settle delay. Deliberately no raw JS `eval` primitive and no form-fill/login support: `eval` would make `web_fetch` an RCE vector against whatever the browser can reach, compounding the SSRF surface AWU 6.1 closed off. Using `actions` forces the browser path (fast paths skipped) and disables the `FetchCache` read/write for that call, since the same URL can yield different content depending on the actions applied.
    - **Browser Fallback** (`src/fetch/navigate.rs`): Otherwise opens a page in the shared `chromiumoxide` browser, waits for the DOM to stabilize, and checks for Cloudflare/CAPTCHA block pages before extracting content. Navigation gets one automatic retry with a short backoff on transient-looking failures (timeout, navigation, page-creation errors) -- not retried when it's known to be pointless, like a CAPTCHA block.
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
| **SSRF Protection** | `ipnet` | CIDR containment checks against DNS-resolved IPs before any fetch. |
| **robots.txt** | `texting_robots` | Parses and evaluates `robots.txt` disallow rules. |
| **Serialization**| `serde` | Industry standard for high-speed JSON processing. |
| **Networking** | `reqwest` | For lightweight, non-browser HTTP requests (search, GitHub raw, PDF, robots.txt); honors `HTTP_PROXY`/`HTTPS_PROXY` by default. |
| **Error Handling** | `thiserror` + `anyhow` | `thiserror` models typed, agent-facing domain errors (`FetchError`, `SearchError`) in `fetch.rs`/`search.rs`; `anyhow` propagates them (plus top-level ad hoc errors) up through `handlers.rs`/`main.rs` to the MCP response. |
