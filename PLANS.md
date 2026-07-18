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
    *   Implement GitHub fallback (Raw/Git).
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
| **1.2** | **MCP Layer & Stdio Communication** | `[✅] Completed` | Implemented all sub-tasks for MCP communication. | |
---

## 🎯 Current Status
**Current Phase:** Phase 1 (Core Implementation)
**Current Objective:** AWU 1.2: MCP Layer & Stdio Communication

---

## 🚀 Short-Term Plan

*   **[✅] AWU 1.2: MCP Layer & Stdio Communication**
    *   **Objective**: Implement `mcp-core` integration and stdio communication.
    *   **Scope**: Define protocol models, implement transport, and handlers for `ListTools` and `CallTool`.
    *   **DoD**: Successfully exchange JSON-RPC messages over stdio.
    *   **Sub-tasks**:
        *   [✅] AWU 1.2.1: Define MCP Protocol Models (`src/mcp.rs`)
        *   [✅] AWU 1.2.2: Implement MCP Stdio Transport
        *   [✅] AWU 1.2.3: Implement ListTools Handler
        *   [✅] AWU 1.2.4: Implement CallTool Handler
