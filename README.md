# web-access-mcp

**The "Ultimate Browsing Extension" for AI Coding Agents.**

`web-access-mcp` is a high-precision, ultra-lightweight Model Context Protocol (MCP) server designed to give AI agents (Claude Desktop, Cursor, etc.) direct, real-time access to the web. It is purpose-built to solve the "noise and token bloat" problem inherent in traditional web scraping.

## 🚀 Key Value Propositions

*   **🎯 High Precision (Markdown-First):** Automatically converts noisy HTML into clean, structured Markdown. No more wading through ads, navbars, or footers.
*   **💎 Zero Cost:** No API keys required. No Google Search API costs. No OpenAI/Anthropic scraping costs. Uses a real browser instance to fetch what you need for free.
*   **⚡ Token Efficiency:** Built-in "Smart Token Cutter" and GitHub-specific fallbacks ensure you never hit LLM context limits with massive, irrelevant HTML.
*   **⚡ Fast Search:** `web_search`/`smart_search` resolve queries via a lightweight HTTP request against Bing's RSS search feed — no browser startup cost, no CSS-selector scraping.
*   **🛡️ Reliable Fetching:** `web_fetch` uses `chromiumoxide` to drive a persistent, shared browser session for pages that need real rendering, with a realistic User-Agent and CAPTCHA/block detection.
*   **📦 Single Binary (Mostly):** A high-performance Rust implementation that works out of the box in any environment with a browser (Chrome/Edge/Chromium) installed.

---

## 🛠️ Available Tools

### `smart_search` (Recommended)
One-stop web research tool that performs search and concurrently fetches extracted Markdown content from top result pages in a single call.
*   **Input:** `query` (string, required), `max_pages` (integer, optional, default 3, max 5)
*   **Output:** Aggregated JSON with search snippets and full extracted Markdown content for top result pages. If content couldn't be fetched for a given result, `content` is omitted and an `error` field explains why (e.g. blocked, timed out) with a hint for what to try next.

### `web_search`
Perform web searches without an API key.
*   **Input:** `query` (string)
*   **Output:** A structured list of search results (Title, URL, and Snippet) in JSON format, parsed from Bing's `format=rss` search feed (typically ~10 results per query).
*   **Optimization:** Uses an in-memory cache to prevent redundant requests and avoid being flagged.

### `web_fetch`
The "Content Extractor" for deep-diving into documentation.
*   **Input:** `url` (string, required)
*   **Output:** The core content of the page in clean Markdown, automatically truncated to ~10,000 characters at the nearest line boundary to protect context windows.
*   **Special Features:**
    *   **SSRF Protection:** Before making any request, the target host is resolved via DNS and every resolved IP is checked against loopback/private/link-local/reserved ranges (including cloud metadata endpoints like `169.254.169.254`); matches are rejected outright. Checking the *resolved* IP rather than the literal URL closes the DNS-rebinding gap where a normal-looking hostname resolves to an internal address.
    *   **GitHub Optimization:** If the URL is a GitHub file/blob URL (contains `github.com` and `/blob/`), it bypasses the browser and pulls the raw source code directly for maximum speed and clarity. Other `github.com` pages (repo home, issues, PRs, etc.) go through the normal browser fetch path.
    *   **PDF Support:** URLs that report (or look like) `application/pdf` are downloaded directly and their text extracted with `pdf-extract`, bypassing the browser entirely (Chromium's built-in PDF viewer renders a viewer UI, not extractable text). Scanned/image-only PDFs with no text layer aren't supported.
    *   **Readability Pipeline:** 
        - **Extraction:** Uses `readabilityrs` to isolate the main article content (removing noise like ads, nav, and sidebars).
        - **Conversion:** Uses `html-to-markdown-rs` to transform the clean HTML into high-fidelity Markdown.
    *   **Smart Truncation:** Safely cuts content at the last newline to prevent context overflow.
    *   **Caching:** Results are cached per-URL for 10 minutes, so re-fetching a URL an agent already visited (including via `smart_search`) skips the browser entirely.

---

## ⚠️ Error Handling

When a tool call fails (bad URL, timeout, CAPTCHA block, missing argument, etc.), the response is a normal MCP tool error (`isError: true`) containing a descriptive message plus a `Hint:` clause telling the calling agent what to try next (e.g. retry once, try a different URL, use `web_search` to find an alternative source). A single failed call never crashes the server or the rest of the session.

---

## 📦 Installation & Setup

### Prerequisites
*   Rust (for building from source)
*   A modern web browser (Google Chrome, Microsoft Edge, or Chromium) installed on your system.

### Building from Source
1. Clone the repository:
   ```bash
   git clone https://github.com/akahmys/web-access-mcp.git
   cd web-access-mcp
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```
3. The binary will be located in `target/release/web-access-mcp`.

### Development
Enable the repo's pre-commit hook (scans staged changes for secrets and absolute paths via `scripts/check_secrets.sh`) once per clone:
```bash
git config core.hooksPath .githooks
```
The same script (in `--all` mode) plus `cargo build`/`test`/`clippy --all-targets -- -D warnings` also run in CI on every push/PR (see `.github/workflows/ci.yml`).

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
*   **Search Parsing:** `quick-xml` (Bing's RSS search feed)
*   **Markdown:** `html-to-markdown-rs`
*   **Extraction:** `readabilityrs`
*   **PDF Text Extraction:** `pdf-extract`
*   **SSRF Protection:** `ipnet` (CIDR range checks against resolved IPs)
*   **Serialization:** `serde`
*   **Error Handling:** `thiserror` (typed domain errors carrying agent-facing hints) + `anyhow` (top-level propagation)
*   **Networking:** `reqwest` (search HTTP requests, GitHub raw fetch, PDF download)

---

## ⚖️ License
This project is released under the MIT License.
