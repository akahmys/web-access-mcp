# Project Roadmap & Execution Log

## 🗺️ Roadmap

### Phase 1: Core Implementation (Foundation)
*   **AWU 1.1: Project Scaffolding & Dependency Setup**
    *   Configure `Cargo.toml` with `tokio`, `chromiumoxide`, `mcp-core`, etc.
    *   Initialize error handling and logging.
*   **AWU 1.2: MCP Layer & Stdio Communication**
    *   Implement `mcp-core` integration.
    *   Define `ListTools` and `CallTool` handlers.
*   **AWU 1.3: Browser Manager & Shared State**
    *   Implement `BrowserState` using `Arc<RwLock<...>>`.
    *   Initialize `chromiumoxide` and session management.

### Phase 2: Tool Implementation (Functionality)
*   **AWU 2.1: `google_search` Implementation**
    *   Implement `scraper`-based extraction for search results.
    *   Add in-memory TTL cache (DashMap).
*   **AWU 2.2: `web_fetch` Core Logic**
    *   Implement basic HTML fetching via `chromiumoxide`.
    *   Implement `html2md` conversion logic.
*   **AWU 2.3: Advanced Fetching Features**
    *   Implement GitHub fallback (Raw URL).
    *   Implement "Smart Token Cutter" (Safety boundary).
    *   Implement SPA/Networkidle waiting logic.

### Phase 3: Polishing & Audit (Reliability)
*   **AWU 3.1: Integration Testing**
    *   End-to-end tests for `google_search` $\rightarrow$ `web_fetch` flow.
*   **AWU 3.2: Error Handling & Edge Cases**
    *   Handle CAPTCHA/Block detection gracefully.
    *   Handle network timeouts and connection errors.
*   **AWU 3.3: Final Audit & Release**
    *   Audit code for memory leaks and thread safety.
    *   Final documentation and binary release.

---

## 📝 Execution Log

| AWU ID | Task Name | Status | Result / Notes |
| :--- | :--- | :--- | :--- |
| **0.1** | **Rewrite Phase 0: Documentation** | `[✅] Completed` | Updated ARCHITECTURE.md, README.md, PLANS.md for the new design. |
| **1.2.1** | **Define MCP Protocol Models** | `[✅] Completed` | Created `src/mcp.rs` with JSON-RPC 2.0 and MCP models. |
| **1.2.2** | **Implement MCP Stdio Transport** | `[✅] Completed` | Implemented `StdioTransport` in `src/transport.rs`. |
| **1.2.3** | **Implement ListTools Handler** | `[✅] Completed` | Implemented `list_tools_handler` in `src/main.rs` with dummy tools. |
| **1.2.4** | **Implement CallTool Handler** | `[✅] Completed` | Implemented `call_tool_handler` in `src/main.rs`. |
| **1.2** | **MCP Layer & Stdio Communication** | `[✅] Completed` | Implemented all sub-tasks for MCP communication. |
| **1.3** | **Browser Manager & Shared State** | `[✅] Completed` | Implemented `BrowserState` and `BrowserError` for `chromiumoxide` management. |
| **2.1** | **`google_search` Implementation** | `[✅] Completed` | Implemented search tool with scraper and DashMap cache. |
| **2.2** | **`web_fetch` Core Logic** | `[✅] Completed` | Implemented basic HTML fetching and markdown conversion. |
| **2.3.1** | **GitHub Fallback** | `[✅] Completed` | Added logic to fetch raw content from GitHub instead of full browser rendering. |
| **2.3.2** | **Smart Token Cutter** | `[✅] Completed` | Implemented content truncation at meaningful boundaries to prevent context overflow. |
| **2.3.3** | **Robust Content Extraction (Readability)** | `[✅] Completed` | Integrated `readabilityrs` for better content extraction from HTML. |
| **3.1** | **Integration Testing** | `[✅] Completed` | Created `tests/integration_test.rs` to verify full stdio MCP communication flow. |
| **3.2** | **Error Handling & CAPTCHA** | `[✅] Completed` | Implemented timeout wrapping on page fetch and robust CAPTCHA/Block detection logic. |
| **3.3** | **Final Audit & Release** | `[✅] Completed` | Audited codebase and verified compilation/tests run successfully. |
---

## 🎯 Current Status
**Current Phase:** Release & Push (v0.1)
**Current Objective:** Tag version v0.1 and push main branch and tags to GitHub.

---

## 🚀 Short-Term Plan

*   **[✅] AWU 5.1: Implement `smart_search` Aggregated Search Tool**
    *   **Objective**: Add `smart_search` MCP tool that executes search, concurrently fetches top N web page contents, extracts clean Markdown, and returns an aggregated result in 1 tool call.
    *   **Scope**: Create `src/smart_search.rs`, update `src/main.rs`, `src/mcp.rs`, and `README.md`.
    *   **DoD**: `smart_search` tool is registered and returns aggregated search results with extracted page Markdown in 1 call.
    *   **Result**: Created `src/smart_search.rs`, registered `smart_search` tool, updated `README.md`, passed integration tests, and installed binary to `~/.cargo/bin/web-access-mcp`.

*   **[✅] AWU 4.7: Fast & Reliable HTTP Search (Reqwest + Fallback)**
    *   **Objective**: Replace headless browser Google search with `reqwest` HTTP fetch (with fallback to DuckDuckGo HTML search) for `google_search`. This eliminates CDP WebSocket errors, prevents CAPTCHA blocks, and speeds up search response time from 4s to 200ms.
    *   **Scope**: Update `src/search.rs`.
    *   **DoD**: `google_search` returns instant search results without CAPTCHA or CDP errors.
    *   **Result**: Implemented reqwest HTTP search with DuckDuckGo fallback in `src/search.rs`. Verified 200ms response time and accurate search result parsing.

*   **[✅] AWU 4.6: Upgrade chromiumoxide & Add User-Agent Stealth**
    *   **Objective**: Upgrade `chromiumoxide` to `0.9` (fix CDP websocket deserialization errors) and configure realistic User-Agent and headless bypass settings to avoid Google CAPTCHA blocks.
    *   **Scope**: Update `Cargo.toml`, `src/browser.rs`, `src/search.rs`.
    *   **DoD**: `google_search` executes without CDP deserialization errors and successfully returns search results.
    *   **Result**: Upgraded `chromiumoxide` to 0.9.1, added realistic macOS Chrome User-Agent string to BrowserConfig.

*   **[✅] AWU 4.5: Isolate Chromium User Data Dir (Fix SingletonLock error)**
    *   **Objective**: Fix Chromium launch failure (`SingletonLock: File exists (17)`) by assigning a unique/process-isolated user data directory for Chromium in `src/browser.rs`.
    *   **Scope**: Update `src/browser.rs` to set user-data-dir per process.
    *   **DoD**: `google_search` and `web_fetch` launch Chromium cleanly without SingletonLock error.

*   **[✅] AWU 4.4: Lazy Browser Initialization & Instant MCP Connection**
    *   **Objective**: Make browser initialization lazy (on-demand or background) so MCP server responds immediately (0ms delay) to `initialize` and `tools/list` connection requests from MCP clients like `pi`. Also honor `CHROME_PATH` environment variable and add robust launch flags (`--no-sandbox`, `--disable-dev-shm-usage`).
    *   **Scope**: Update `src/browser.rs`, `src/main.rs`, `src/fetch.rs`, `src/search.rs`.
    *   **DoD**: `web-access-mcp` connects instantly to MCP clients without blocking on Chromium launch; `CHROME_PATH` environment variable is properly respected.
    *   **Result**: Implemented lazy browser launch (`get_or_start_browser`), added `--no-sandbox`, `--disable-dev-shm-usage`, and `CHROME_PATH` handling. MCP server now listens to stdio with zero startup delay. Reinstalled binary to `~/.cargo/bin/web-access-mcp`.

*   **[✅] AWU 4.3: Standard MCP Protocol Compliance (`initialize`, `tools/list`, `tools/call`)**
    *   **Objective**: Add standard MCP protocol handlers (`initialize`, `notifications/initialized`, `tools/list`, `tools/call`) to support standard MCP clients like `pi`, Claude Desktop, and Cursor, while preserving backwards compatibility (`list_tools`, `call_tool`).
    *   **Scope**: Update `src/mcp.rs`, `src/main.rs`, and integration tests.
    *   **DoD**: Server responds properly to `initialize`, `tools/list`, and `tools/call` JSON-RPC requests without hanging or timing out.
    *   **Result**: Added `initialize`, `ping`, `tools/list`, and `tools/call` standard handlers to `main.rs` & `mcp.rs`. Passed integration tests and installed updated binary to `~/.cargo/bin/web-access-mcp`.

*   **[✅] AWU 4.2: Tag v0.1 & Push to GitHub**
    *   **Objective**: Create git tag `v0.1` and push `main` branch along with tags to GitHub remote.
    *   **Scope**: Tag `v0.1`, push `main` and tags to remote repositories.
    *   **DoD**: `main` branch and `v0.1` tag are published on GitHub.

*   **[✅] AWU 4.1: Repository Unification to `main`**
    *   **Objective**: Clean up temporary files, commit pending changes, and switch/merge into `main`.
    *   **Scope**: Clean untracked scratch files (`test_*.rs`, etc.), stage/commit valid codebase changes, switch to `main`, merge, and delete temporary feature branch.
    *   **DoD**: Repository is on `main` branch with clean working directory and all code intact.


*   **[✅] AWU 1.2: MCP Layer & Stdio Communication**
    *   **Objective**: Implement `mcp-core` integration and stdio communication.
    *   **Scope**: Define protocol models, implement transport, and handlers for `ListTools` and `CallTool`.
    *   **DoD**: Successfully exchange JSON-RPC messages over stdio.
    *   **Sub-tasks**:
        *   [✅] AWU 1.2.1: Define MCP Protocol Models (`src/mcp.rs`)
        *   [✅] AWU 1.2.2: Implement MCP Stdio Transport
        *   [✅] AWU 1.2.3: Implement ListTools Handler
        *   [✅] AWU 1.2.4: Implement CallTool Handler
*   **[✅] AWU 1.3: Browser Manager & Shared State**
    *   **Objective**: Implement `BrowserState` to manage `chromiumoxide` lifecycle.
    *   **Scope**: Define browser/session errors, implement `BrowserState` with `Arc<RwLock>`, and initialize in `main`.
    *   **DoD**: Browser starts and shuts down gracefully; `main` holds `BrowserState`.
    *   **Sub-tasks**:
        *   [✅] AWU 1.3.1: Define `BrowserError` and `BrowserState` structures
        *   [✅] AWU 1.3.2: Implement browser lifecycle management (start/stop)
        *   [✅] AWU 1.3.3: Integrate `BrowserState` into main application loop
*   **[✅] AWU 2.1: `google_search` Implementation**
    *   **Objective**: Implement `google_search` tool with search result extraction and caching.
    *   **Scope**: Implement `scraper`-based extraction for search results and an in-memory TTL cache using `DashMap`.
    *   **DoD**: Successful execution of `google_search` tool returning structured results; cache correctly stores/expires results.
    *   **Sub-tasks**:
        *   [✅] Implement search query processing and `chromiumoxide` navigation.
        *   [✅] Implement `scraper`-based HTML parsing for search results.
        *   [✅] Implement `DashMap` based TTL cache.
*   **[✅] AWU 2.2: `web_fetch` Core Logic**
    *   **Objective**: Implement basic HTML fetching via `chromiumoxide`.
    *   **Scope**: Implement `html2md` conversion logic.
    *   **DoD**: Successfully fetch and convert web pages to markdown.
    *   **Sub-tasks**:
        *   [✅] Implement HTML fetching logic.
        *   [✅] Implement markdown conversion logic.
*   **[✅] AWU 2.3: Advanced Fetching Features**
    *   **Objective**: Implement enhanced fetching capabilities.
    *   **Scope**: Implement GitHub fallback, smart token cutter, and SPA support.
    *   **DoD**: Robust handling of different content sources and large pages.
    *   **Sub-tasks**:
        *   [✅] Implement GitHub fallback.
        *   [✅] Implement "Smart Token Cutter".
        *   [✅] Implement SPA/Networkidle waiting logic.
        *   [✅] AWU 2.3.3: Robust Content Extraction (Readability)

* **[✅] AWU 2.2.1: Refactor Markdown Conversion**
    * **Objective**: Replace manual HTML-to-Markdown logic with `html-to-markdown-rs` for improved reliability and feature completeness.
    * **Scope**: Add `html-to-markdown-rs` dependency and update `src/fetch.rs`.
    * **DoD**: High-fidelity Markdown conversion using the library, covering tables, images, and nested lists.
    * **Result**: Successfully refactored `src/fetch.rs` to use the new pipeline and resolved API mismatch errors.

* **[✅] Phase 3: Polishing & Audit**
    * **Objective**: Ensure robustness via error handling, timeouts, and integration tests.
    * **Scope**: Implement timeout wrapping for browser requests, CAPTCHA/Block detection for search and fetch, and full end-to-end integration tests.
    * **DoD**: Compilation succeeds with warnings reviewed, all unit/integration tests pass cleanly.
    * **Result**: Implemented CAPTCHA detection, timeout handling, and end-to-end stdio testing. All tests passing.

