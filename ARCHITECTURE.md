# Architecture: rust-web-mcp

## 🎯 Mission Statement
`rust-web-mcp` is a high-precision, token-efficient Web Search & Documentation MCP Server designed specifically for AI Coding Agents. It aims to provide "Zero-cost, Zero-API-key" access to the live web, transforming complex, noisy HTML into clean, structured Markdown.

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
    
    subgraph rust-web-mcp [Rust Runtime]
        MCP -->|CallTool| HANDLER[Tool Handlers]
        
        subgraph STATE [Shared Thread-Safe State]
            HANDLER -->|Request/Update| CACHE[In-Memory TTL Cache]
            HANDLER -->|Execute| BROWSER[Browser Controller: chromiumoxide]
        end
        
        subgraph ENGINE [Extraction Engine]
            BROWSER -->|DOM Content| MD[Markdown Converter]
            BROWSER -->|Search HTML| GOOGLE[Google Parser]
            BROWSER -->|Raw Source| GITHUB[GitHub Fallback]
        end
    end

    MD -->|Clean Markdown| AI
    GOOGLE -->|Structured Results| AI
    GITHUB -->|Clean Code| AI
```

### 1. MCP Layer (`mcp-core` / `async-mcp`)
Handles the lifecycle of the MCP server, listening on `stdin` and communicating via `stdout`. It maps incoming JSON-RPC `call_tool` requests to specific asynchronous Rust functions.

### 2. Shared State Management (`BrowserState`)
To ensure the agent can "click" through a search result to read a page, the browser session is not per-request but **per-server-instance**.
- **`Arc<RwLock<BrowserState>>`**: Allows multiple concurrent tool calls to share the same browser profile/cookies.
- **In-Memory Cache**: A `DashMap` or `HashMap` with TTL (Time-To-Live) to prevent redundant searches and minimize latency.

### 3. Extraction & Transformation Engine
- **`google_search`**: Uses `scraper` to parse Google Search results into a structured list (Title, URL, Snippet).
- **`web_fetch`**:
    - **GitHub Logic**: Detects `github.com` and switches to Raw API/Git fallback to avoid heavy rendering.
    - **Markdown Logic**: Uses `html2md` or a `Readability`-style algorithm to strip navbars, footers, and ads, returning only the core technical content.
    - **Smart Cutter**: Enforces `max_length` to ensure the response fits within the agent's context window.

---

## 🛠️ Technical Stack

| Component | Technology | Reason |
| :--- | :--- | :--- |
| **Runtime** | `tokio` | High-performance asynchronous I/O. |
| **Browser** | `chromiumoxide` | Reliable CDP-based control of real Chrome/Edge. |
| **Parsing** | `scraper` | Fast and precise CSS selector-based HTML parsing. |
| **Markdown** | `html2md` | Lightweight conversion of clean HTML to Markdown. |
| **Serialization**| `serde` | Industry standard for high-speed JSON processing. |
| **Networking** | `reqwest` | For lightweight, non-browser HTTP requests (GitHub). |
