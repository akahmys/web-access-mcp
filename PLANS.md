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

