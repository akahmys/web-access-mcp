# web-access-mcp

**The "Ultimate Browsing Extension" for AI Coding Agents.**

`web-access-mcp` is a high-precision, ultra-lightweight Model Context Protocol (MCP) server designed to give AI agents (Claude Desktop, Cursor, etc.) direct, real-time access to the web. It is purpose-built to solve the "noise and token bloat" problem inherent in traditional web scraping.

## 🚀 Key Value Propositions

*   **🎯 High Precision (Markdown-First):** Automatically converts noisy HTML into clean, structured Markdown. No more wading through ads, navbars, or footers.
*   **💎 Zero Cost:** No API keys required. No Google Search API costs. No OpenAI/Anthropic scraping costs. Uses a real browser instance to fetch what you need for free.
*   **⚡ Token Efficiency:** Built-in "Smart Token Cutter" and GitHub-specific fallbacks ensure you never hit LLM context limits with massive, irrelevant HTML.
*   **🛡️ Stealth & Reliability:** Uses `chromiumoxide` to maintain a persistent, shared browser session, making your searches appear like natural human browsing to bypass CAPTCHAs.
*   **📦 Single Binary (Mostly):** A high-performance Rust implementation that works out of the box in any environment with a browser (Chrome/Edge/Chromium) installed.

---

## 🛠️ Available Tools

### `smart_search` (Recommended)
One-stop web research tool that performs search and concurrently fetches extracted Markdown content from top result pages in a single call.
*   **Input:** `query` (string, required), `max_pages` (integer, optional, default 3, max 5)
*   **Output:** Aggregated JSON with search snippets and full extracted Markdown content for top result pages.

### `google_search`
Perform web searches without an API key.
*   **Input:** `query` (string)
*   **Output:** A structured list of the top 5 search results (Title, URL, and Snippet) in JSON format.
*   **Optimization:** Uses an in-memory cache to prevent redundant requests and avoid being flagged.

### `web_fetch`
The "Content Extractor" for deep-diving into documentation.
*   **Input:** `url` (string, required), `max_length` (integer, optional, default 8000)
*   **Output:** The core content of the page in clean Markdown.
*   **Special Features:**
    *   **GitHub Optimization:** If the URL is `github.com`, it bypasses the browser and pulls the raw source code directly for maximum speed and clarity.
    *   **Readability Pipeline:** 
        - **Extraction:** Uses `readabilityrs` to isolate the main article content (removing noise like ads, nav, and sidebars).
        - **Conversion:** Uses `html-to-markdown-rs` to transform the clean HTML into high-fidelity Markdown.
    *   **Smart Truncation:** Safely cuts content at the last newline to prevent context overflow.

---

## 📦 Installation & Setup

### Prerequisites
*   Rust (for building from source)
*   A modern web browser (Google Chrome, Microsoft Edge, or Chromium) installed on your system.

### Building from Source
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/web-access-mcp.git
   cd web-access-mcp
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```
3. The binary will be located in `target/release/web-access-mcp`.

### Integrating with MCP (e.g., Claude Desktop)
Add the following to your `mcp_config.json`:

```json
{
  "mcpServers": {
    "web-access-mcp": {
      "command": "/path/to/your/target/release/web-access-mcp",
      "env": {
        "CHROME_PATH": "/path/to/your/chrome" 
      }
    }
  }
}
```
*(Note: `CHROME_PATH` is optional if Chrome is in your default system path.)*

---

## 🏗️ Technical Stack

*   **Language:** Rust (for memory safety and speed)
*   **Async Runtime:** `tokio`
*   **Browser Automation:** `chromiumoxide` (CDP)
*   **Parsing:** `scraper` (CSS selectors)
*   **Markdown:** `html-to-markdown-rs`
*   **Extraction:** `readabilityrs`
*   **Serialization:** `serde`

---

## ⚖️ License
This project is released under the MIT License.
