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

### Phase 6: Security, Capability & Reliability Hardening (long-term, in progress)
Identified via a 2026-07-28 review of "what does this MCP still need as a web-access tool for agents." Ordered by priority: security first, then capability gaps, then general scraping reliability. See the detailed AWU 6.x entries in the Short-Term Plan for Objective/Scope/DoD on each.
*   **AWU 6.1: SSRF Protection for `web_fetch`** (security, highest priority) -- done
*   **AWU 6.2: PDF/Binary Content Support for `web_fetch`** -- done
*   **AWU 6.3: `web_fetch` Result Caching** -- done
*   **AWU 6.4: Batch-Fetch Tool (concurrent fetch of N given URLs)** -- done
*   **AWU 6.5: Page Interaction Primitives (click/scroll/form-fill) for `web_fetch`** -- done (click/scroll only, no form-fill/login)
*   **AWU 6.6: Scraping Reliability Hardening (robots.txt, UA rotation, proxy, retry backoff)** -- done
*   **AWU 6.7: Migrate MCP Payload Types to `rust-mcp-schema`** -- done

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
**Current Phase:** Post-v0.1 Reliability & Polish
**Current Objective:** v0.1 is tagged and pushed. Search backend is Bing-only (`web_search`). Phase 6 (SSRF protection, PDF support, `web_fetch` caching, `batch_fetch`, page-interaction primitives, robots.txt/UA/proxy/retry hardening, `rust-mcp-schema` migration) is complete -- all seven AWUs (6.1-6.7) landed. No open items on the roadmap right now; next work should come from a fresh review or a new user request.

---

## 🚀 Short-Term Plan

*   **[✅] AWU 8.1: Comprehensive System Architecture Refactoring**
    *   **Objective**: Execute 6 core architecture improvements: (1) Explicit Chromium page teardown (`page.close().await`) to prevent target leaks, (2) Readability fallback to noise-stripped HTML body Markdown conversion, (3) Bounded capacity enforcement on `TtlCache`, (4) Semaphore-bounded page creation concurrency in `BrowserState`, (5) User-Agent rotation per browser launch, and (6) System guidance in MCP `handle_initialize` `instructions`.
    *   **Scope**: `src/fetch.rs`, `src/fetch/navigate.rs`, `src/fetch/tests.rs`, `src/cache.rs`, `src/cache/tests.rs`, `src/user_agent.rs`, `src/browser.rs`, `src/context.rs`, `src/main.rs`.
    *   **DoD**: All 6 architectural improvements implemented cleanly; all unit and integration tests compile and pass with 0 errors/warnings (`cargo test`, `cargo clippy --all-targets -- -D warnings`).
    *   **Result**: Implemented all 6 architectural improvements: (1) Added `page.close().await` teardown in `fetch_url_uncached` (`src/fetch.rs`) to prevent target/tab memory leaks; (2) Added fallback path in `html_to_markdown` (`src/fetch.rs`) to convert raw HTML directly when Readability parse fails; (3) Added `max_capacity: 500` enforcement in `TtlCache` (`src/cache.rs`); (4) Added 5-permit `Semaphore` concurrency limiting in `BrowserState::new_page` (`src/browser.rs`); (5) Added `random_user_agent()` per browser launch in `src/user_agent.rs`; and (6) Added tool usage `instructions` to MCP `handle_initialize` (`src/main.rs`). All 19 unit tests and 3 integration tests passed cleanly; `cargo clippy --all-targets -- -D warnings` passed with 0 errors/warnings.

*   **[✅] AWU 7.3: Fix Compilation Error & Documentation Discrepancies**
    *   **Objective**: (1) Fix `Duration::from_hours`/`from_mins` compile error in `src/main.rs`, (2) Update `README.md` to accurately describe `web_fetch` output format (JSON object with `title` and `content`), and (3) Document per-item content limits (2,500 characters) for `smart_search` and `batch_fetch` in `README.md`.
    *   **Scope**: `src/main.rs`, `README.md`, `PLANS.md`.
    *   **DoD**: `cargo check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` compile and pass with 0 errors/warnings; `README.md` accurately describes output formats and content limits.
    *   **Result**: Fixed standard `Duration` initialization in `src/main.rs` using explicit minute calculations (`Duration::from_secs(60 * 60)` and `Duration::from_secs(10 * 60)`). Updated `README.md` tool section to specify that `web_fetch` returns a JSON object `{"title": "...", "content": "..."}` rather than plain text, and added explicit 2,500-character per-item truncation limits for `smart_search` and `batch_fetch`. `cargo test` (17 unit + 3 integration tests) and `cargo clippy --all-targets -- -D warnings` compiled and passed cleanly with 0 errors/warnings.

*   **[✅] AWU 7.2: Architectural Hardening & Pluggable SearchProvider**
    *   **Objective**: Enhance system resilience and architecture: (1) Enforce a 10MB download size cap on HTTP/PDF fetches to prevent OOM attacks, (2) Add self-healing browser recovery when Chromium CDP connection dies, (3) Add background/active eviction (`evict_expired`) for `TtlCache`, and (4) Abstract search into a unified `SearchProvider` trait for flexible search engine switching.
    *   **Scope**: `src/search.rs`, `src/browser.rs`, `src/cache.rs`, `src/fetch/pdf.rs`, `src/fetch/fast_path.rs`, `src/context.rs`.
    *   **DoD**:
        1. `SearchProvider` trait defined and implemented by `BingSearchProvider`, allowing easy search engine replacement.
        2. `BrowserState` detects dead browser connections and re-launches Chromium automatically.
        3. `TtlCache::evict_expired` removes stale entries.
        4. PDF and HTTP downloads cap body reads at 10MB limit.
        5. `cargo test` and `cargo clippy --all-targets -- -D warnings` pass clean with 0 warnings.
    *   **Result**: Defined `SearchProvider` trait in `src/search.rs` and implemented `BingSearchProvider`, registering it in `AppContext` so any search engine can be swapped seamlessly. Added self-healing recovery in `BrowserState::new_page` (`src/browser.rs`) to auto-restart Chromium if CDP page creation fails. Added `evict_expired()` to `TtlCache` (`src/cache.rs`) and `evict_expired_caches()` to `AppContext`, wired to pings/messages in `main.rs` to prevent long-term memory growth. Enforced a 10MB `MAX_DOWNLOAD_SIZE` limit in `src/fetch/pdf.rs` and `src/fetch/fast_path.rs` for PDF and GitHub raw fetches to protect against OOM memory exhaustion. Added unit test `test_ttl_cache_evict_expired` in `src/cache/tests.rs`. All 17 unit tests and 3 integration tests passed cleanly; `cargo clippy --all-targets -- -D warnings` passed with 0 warnings.

*   **[✅] AWU 5.13: Configure Betterleaks Pre-Commit Hook & Add "Jun Kato" Rule**
    *   **Objective**: Transition pre-commit hook to `betterleaks`, remove custom `check_secrets.sh` script and `.githooks` directory, and add rule to detect personal name "Jun Kato".
    *   **Scope**: `.betterleaks.toml`, `.git/hooks/pre-commit`, `.github/workflows/ci.yml`, `README.md`, `check_secrets.sh` (deleted), `.githooks/` (deleted).
    *   **DoD**: Custom scripts deleted; pre-commit hook uses `betterleaks`; personal name "Jun Kato" rule configured in `.betterleaks.toml`; verified leak detection and clean scan.
    *   **Result**: Removed `scripts/check_secrets.sh` and `.githooks/`. Configured `.git/hooks/pre-commit` to stream staged diffs into `betterleaks stdin`. Added `personal-name-jun-kato` rule to `.betterleaks.toml`. Verified that staging "Jun Kato" or "jun kato" is detected and blocked by the pre-commit hook, while the workspace passes clean scans.

*   **[✅] AWU 5.12: Migrate Secret Check to `betterleaks`**
    *   **Objective**: Replace hand-written bash grep secret/path regexes in `scripts/check_secrets.sh` with `betterleaks` scanning engine and `.betterleaks.toml` configuration.
    *   **Scope**: `.betterleaks.toml` (new), `scripts/check_secrets.sh`.
    *   **DoD**: `.betterleaks.toml` rules configured for absolute path and secret detection; `scripts/check_secrets.sh` delegates staged and full-repo scans to `betterleaks`; verified execution with `--all` and staged diffs.
    *   **Result**: Created `.betterleaks.toml` defining custom absolute path rules (`/Users/`, `/home/`) with allowlists for markdown/tests/agent rules. Refactored `scripts/check_secrets.sh` to execute `betterleaks git` for full scans and `betterleaks stdin` for staged diffs. Verified both `--all` mode and staged diff scans pass cleanly.

*   **[✅] AWU 5.11: Migrate License Check to `cargo-deny`**
    *   **Objective**: Replace custom Python parser logic in `scripts/check_licenses.py` with `cargo-deny` configuration and invocation, and integrate license checking into CI.
    *   **Scope**: `deny.toml` (new), `scripts/check_licenses.py`, `.github/workflows/ci.yml`.
    *   **DoD**: `deny.toml` configured with allowed license whitelist; `scripts/check_licenses.py` delegates execution to `cargo deny check licenses`; CI includes a `license-scan` job using `EmbarkStudios/cargo-deny-action@v2`.
    *   **Result**: Created `deny.toml` with allowed license list matching existing policy. Updated `scripts/check_licenses.py` to invoke `cargo-deny`. Added `license-scan` job to `ci.yml`. Verified execution locally via `python3 scripts/check_licenses.py` and confirmed `cargo test` passes.

*   **[✅] AWU 7.1: Comprehensive Codebase Refactoring & Quality Hardening**
    *   **Objective**: Comprehensive refactoring across architecture, data structures, data operations, error handling, and potential logical bugs (e.g. UTF-8 slice panic risk in `truncate_content`, redundant `BrowserState` dependencies in search, argument extraction boilerplate in handlers, and `AppContext` encapsulation).
    *   **Scope**: `src/main.rs`, `src/handlers.rs`, `src/search.rs`, `src/fetch.rs`, `src/browser.rs`, `src/error.rs`, `src/smart_search.rs`, `src/batch_fetch.rs`, `src/cache.rs`, `src/context.rs`, `tests/integration_test.rs`.
    *   **DoD**: 
        1. Fix potential UTF-8 multi-byte slicing panic in `truncate_content`.
        2. Introduce `AppContext` to encapsulate shared state (`BrowserState`, `SearchCache`, `FetchCache`) and clean up function signatures (remove `_browser_state` from `web_search`).
        3. Clean up and unify error handling: consolidate `AppError`/`BrowserError`/`AppResult` inconsistencies in `src/error.rs`.
        4. Eliminate argument parsing boilerplate in `src/handlers.rs` with concise helpers.
        5. Use `reqwest::Url::parse_with_params` in `src/search.rs` for type-safe query construction.
        6. All unit & integration tests pass cleanly with `cargo test` and `cargo clippy --all-targets -- -D warnings`.
    *   **Result**: Created `src/context.rs` containing `AppContext` and updated `main.rs`, `handlers.rs`, `smart_search.rs`, and `batch_fetch.rs` to use it, eliminating repetitive state parameter passing. Fixed potential runtime panic in `truncate_content` (`src/fetch.rs`) by replacing raw byte slicing with UTF-8 character boundary checking (`content.is_char_boundary`). Added unit test `test_content_truncation_multibyte` in `src/fetch/tests.rs` to verify Japanese/multibyte string truncation. Removed unused `_browser_state` argument from `perform_web_search` and refactored Bing search URL generation to use `reqwest::Url::parse_with_params`. Extracted `get_str_arg` and `get_usize_arg` helpers in `src/handlers.rs` to eliminate argument parsing boilerplate. Added `test_mcp_invalid_tool_error` in `tests/integration_test.rs` to verify error tool call responses. All 16 unit tests and 3 integration tests passed cleanly, and `cargo clippy --all-targets -- -D warnings` passed with 0 warnings.

*   **[✅] AWU 5.10: Wire `check_secrets.sh` into a Pre-Commit Hook and CI**
    *   **Objective**: `scripts/check_secrets.sh` existed but wasn't actually run by anything -- neither at commit time nor in CI. Connect it to both.
    *   **Scope**: `.githooks/pre-commit` (new), `.github/workflows/ci.yml` (new -- no CI existed at all before this), `README.md`.
    *   **DoD**: A `git commit` with staged secrets/absolute paths gets blocked locally (once `core.hooksPath` is configured); CI runs the same script in `--all` mode on every push/PR, alongside `cargo build`/`test`/`clippy --all-targets -- -D warnings`.
    *   **Result**: Git hooks aren't tracked/shared by git itself, so used the `core.hooksPath` mechanism instead: added a tracked `.githooks/pre-commit` (thin wrapper that `exec`s `scripts/check_secrets.sh` in its default staged-diff mode) and set `git config core.hooksPath .githooks` locally; documented the one-time opt-in command in README for other clones, since core.hooksPath is per-clone config, not something a commit can silently turn on for everyone. Since no `.github/workflows/` existed at all, added `ci.yml` with two jobs: `build-test` (build/test/clippy, matching this session's local verification standard) and `secret-scan` (`check_secrets.sh --all`). Confirmed the existing integration tests don't need a real browser in CI -- they only exercise `initialize`/`tools/list`, never a tool call that would lazily launch Chromium. Verified both the hook (direct invocation) and `--all` mode locally before committing.
    *   **Note**: `check_licenses.py` (the other audit script) was *not* wired into either the hook or CI -- the user's request named `check_secrets.sh` specifically; flagging this as a natural follow-up rather than doing it unprompted.

*   **[✅] AWU 6.7: Migrate MCP Payload Types to `rust-mcp-schema`**
    *   **Objective**: `src/mcp.rs` hand-rolled every MCP-specific payload type (`InitializeResult`, `ServerCapabilities`, `ImplementationInfo`, `ListToolsResult`, `McpTool`, `CallToolRequest`, `CallToolResult`, `McpContent`), and `handle_initialize` hardcoded `protocol_version: "2024-11-05"` -- two spec revisions behind current. This project's client counterpart (`rad`'s `mcp-tool-provider` extension) had already migrated to the `rust-mcp-schema` crate and `ProtocolVersion::latest()`; bring the server side in line with the same crate and pattern.
    *   **Scope**: `Cargo.toml` (new `rust-mcp-schema = "0.10.3"` dependency), `src/mcp.rs`, `src/main.rs`, `src/handlers.rs`. Transport-envelope types (`JsonRpcMessage`/`JsonRpcRequest`/`JsonRpcResponse`/`JsonRpcNotification`/`JsonRpcError`) explicitly out of scope -- kept hand-written, since they're generic JSON-RPC 2.0, not MCP-specific.
    *   **DoD**: `handle_initialize` reports `ProtocolVersion::latest()` instead of a hardcoded string; `tools/list`/`tools/call` are built from `rust-mcp-schema` types instead of the hand-rolled equivalents; `cargo build`/`test`/`clippy --all-targets -- -D warnings` clean (including the existing `#![deny(clippy::pedantic)]`); verified end-to-end against the real `rad` client, not just direct JSON-RPC probing.
    *   **Result**: Removed all MCP-specific structs from `src/mcp.rs` (105 -> 47 lines), keeping only the JSON-RPC envelope. `handle_initialize` now builds `rust_mcp_schema::InitializeResult`/`Implementation`/`ServerCapabilities` directly, with `protocol_version: ProtocolVersion::latest().to_string()` (resolves to `"2025-11-25"` as of crate 0.10.3). `handlers.rs`'s tool builders switched to `Tool`/`ToolInputSchema::new(required, properties, schema)` (the latter has a private discriminator field, so it requires its constructor rather than a struct literal); added a small `schema_properties` helper to convert this codebase's existing `json!({...})` schema literals into `ToolInputSchema`'s required `BTreeMap<String, serde_json::Map<String, Value>>` shape. `call_tool_handler`'s result path switched to `CallToolResult`/`ContentBlock::TextContent(TextContent::new(...))`. `CallToolRequestParams.arguments` (a `serde_json::Map`) is converted to `Value::Object(...)` once at the `main.rs::handle_tools_call` boundary, so the five existing `&Value`-based argument-parsing functions in `handlers.rs` needed no changes. Verified the real API shapes against the vendored crate source directly (not docs) before writing any code, per the user's explicit instruction, since e.g. `ToolInputSchema`'s constructor signature and `ServerCapabilities`'s `Default` impl aren't obvious from a summary. `cargo build`/`test`/`clippy --all-targets -- -D warnings` all clean on the first attempt. End-to-end verified via the actual `rad` binary (not just direct JSON-RPC piping): `rad`'s `mcp-tool-provider` extension successfully completes the handshake against the newly-built binary and lists all 4 tools, with `protocolVersion: "2025-11-25"` in the response -- matching `rad`'s own `ProtocolVersion::latest()` on the client side. (One detour during verification: the first several `rad` e2e attempts timed out on the handshake; root cause turned out to be the host machine's own memory pressure intermittently SIGKILLing freshly-spawned processes, unrelated to this change -- confirmed by reproducing the same SIGKILL against a plain foreground `web-access-mcp` invocation outside `rad` entirely, and resolved by freeing memory rather than any code change.)

*   **[✅] AWU 6.1: SSRF Protection for `web_fetch`** (security, do first)
    *   **Objective**: `web_fetch` currently navigates the shared browser to *any* URL an agent passes it, including `localhost`, RFC1918 private ranges, link-local addresses, and cloud metadata endpoints (`169.254.169.254`). If an agent is fed a malicious URL (e.g. via prompt injection from a page it's summarizing), this is a Server-Side Request Forgery vector into the host's internal network. Block it before the browser ever navigates.
    *   **Scope**: `src/fetch.rs` (new validation step ahead of `open_and_load_page`), `src/error.rs` or a new `FetchError` variant, unit tests in `src/fetch/tests.rs`.
    *   **DoD**: URLs resolving to loopback (`127.0.0.0/8`, `::1`), private ranges (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), link-local (`169.254.0.0/16`, `fe80::/10`), and other non-routable ranges are rejected with a clear `Hint:`-bearing error *before* any network/browser call is made (resolve the hostname first, then check the IP -- reject on DNS resolution too, not just literal-IP URLs, to close the DNS-rebinding gap). Legit public URLs are unaffected; add tests covering both.
    *   **Result**: Added `validate_public_url` in new `src/fetch/ssrf.rs` (mirroring the `pdf.rs`/`tests.rs` submodule pattern), called as literal step 0 of `fetch_url` -- before the GitHub-raw shortcut, the PDF check, and the browser path, so it protects all three uniformly. Parses the URL (`reqwest::Url`, no new dependency needed since reqwest already re-exports it), rejects non-http(s) schemes, resolves the host via `tokio::net::lookup_host`, then checks every resolved `IpAddr` -- including the embedded IPv4 inside an IPv4-mapped IPv6 literal (`::ffff:169.254.169.254`), a known bypass trick -- against a blocklist built with the new `ipnet` dependency: IPv4 `0.0.0.0/8`, `10.0.0.0/8`, `100.64.0.0/10` (CGNAT), `127.0.0.0/8`, `169.254.0.0/16`, `172.16.0.0/12`, `192.168.0.0/16`; IPv6 `::/128`, `::1/128`, `fc00::/7` (unique-local), `fe80::/10`. Added `FetchError::InvalidUrl` and `FetchError::SsrfBlocked` (the latter's hint explicitly says "don't retry" since it's not a transient failure). 6 new unit tests in `src/fetch/ssrf/tests.rs` cover the pure `is_blocked` range-checking logic (no DNS I/O in tests); the full resolve-then-check flow was verified live against the compiled binary instead -- `127.0.0.1`, `169.254.169.254`, `localhost` (proving the DNS-then-check ordering actually works, not just literal-IP matching), and `192.168.1.1` were all rejected, while `https://example.com` passed through to a normal browser fetch. Didn't extend this same check to the GitHub-raw-URL rewrite path (`get_github_raw_url`'s naive full-string `.replace()`) -- it always targets the fixed, non-attacker-arbitrary host `raw.githubusercontent.com`, so it's a much narrower, lower-severity pre-existing fragility than the "fetch any URL" surface this AWU targets; noted as a possible small follow-up, not folded into this change.
    *   **Result**: _not started_

*   **[✅] AWU 6.2: PDF/Binary Content Support for `web_fetch`**
    *   **Objective**: `web_fetch` assumes HTML; a PDF URL (common for papers/specs/docs) currently either fails or gets mangled through the Readability/Markdown pipeline. Detect PDF responses and extract text instead.
    *   **Scope**: `src/fetch.rs`, `src/fetch/pdf.rs` (new), `Cargo.toml`, `README.md`, `ARCHITECTURE.md`.
    *   **DoD**: `web_fetch` on a PDF URL (detected via `Content-Type` or `.pdf` extension) returns extracted text content instead of an HTML-pipeline error; non-PDF binary content still fails with a clear `Hint:` (not silently mangled).
    *   **Result**: Chose `pdf-extract` (simple `extract_text_from_mem(&[u8]) -> Result<String, OutputError>` API, sufficient for readable-text extraction) over `lopdf` (lower-level, would need manual text-layer walking). Detection: a best-effort `HEAD` request checks `Content-Type` for `application/pdf`; if `HEAD` fails or is inconclusive, falls back to a `.pdf`-suffix URL heuristic. On a PDF match, downloads via `reqwest::get` and extracts text -- the browser is never launched (Chromium's built-in PDF viewer renders a viewer UI, not text this pipeline could read). Added `FetchError::PdfExtraction` for genuine extraction failures (encrypted/corrupted/scanned-image PDFs). Extracted the new logic into `src/fetch/pdf.rs` (mirroring the existing `src/fetch/tests.rs` submodule pattern) to keep `fetch.rs` under CODING.md's 300-line limit -- it would've been 303 lines inline. Verified live against two real PDFs: a small W3C test PDF (~14 bytes of body text) and the 15-page "Attention Is All You Need" arXiv paper, both returning clean extracted text with no browser launched in either case.

*   **[✅] AWU 6.3: `web_fetch` Result Caching**
    *   **Objective**: `google_search`/`web_search` already has a `SearchCache`; `web_fetch` has none, so repeated fetches of the same URL (e.g. an agent re-reading a doc page across turns, or `smart_search` re-visiting a URL already seen in a prior call) always pay full browser-launch + page-load cost.
    *   **Scope**: `src/fetch.rs` (new TTL cache, likely mirroring `search::SearchCache`'s `DashMap` design), `src/main.rs`/`src/handlers.rs` (thread the cache through like `SearchCache` is).
    *   **DoD**: A second `web_fetch` call to the same URL within the cache TTL returns cached content without launching a browser page. TTL should be shorter than search's 1 hour (pages change more often than search rankings) -- pick something like 10-15 minutes, but this is a judgment call worth confirming before implementing.
    *   **Result**: Rather than duplicating `SearchCache`'s `DashMap`+TTL logic for a second time, extracted it into a generic `TtlCache<V: Clone>` in new `src/cache.rs`; `SearchCache` became a one-line type alias (`TtlCache<Vec<SearchResult>>`) with zero call-site changes, and `FetchCache` (`TtlCache<WebFetchResult>`, 10-minute TTL -- picked without a separate check-in since it's a small, easily-adjustable parameter, not a risky one) followed the same pattern. `fetch_url` now checks the cache first and, on a miss, delegates to a renamed `fetch_url_uncached` (the prior body verbatim) and caches its result before returning -- one cache-set point instead of one per return path. Threaded `&FetchCache` through every layer between `main.rs::run()` and `fetch_url` (`handle_message`, `handle_tools_call`, `call_tool_handler`, `call_smart_search`/`call_web_fetch`, `perform_smart_search`, `fetch_one_item`) alongside the existing `&SearchCache`/`&BrowserState` params, matching the codebase's existing individual-parameter style rather than introducing an `AppState`-type bundling struct, which would have been a larger, unrequested refactor. Verified live: fetching `https://example.com` twice back-to-back shows `Chromium browser initialized successfully` only once in the logs, with the second call returning identical content instantly.

*   **[✅] AWU 6.4: Batch-Fetch Tool**
    *   **Objective**: `smart_search` always couples "search" with "fetch top N results"; there's no tool for the simpler case of "fetch these N URLs I already have, concurrently." Add one.
    *   **Scope**: New function (likely `src/fetch.rs` or a small new module) reusing `fetch_url` + `join_all` the same way `smart_search::fetch_one_item` does; new tool registration in `src/handlers.rs` and the `McpTool` schema; `README.md`/`ARCHITECTURE.md`.
    *   **DoD**: New tool accepts `urls: string[]` (with a sane max count, e.g. 5-10, mirroring `smart_search`'s `max_pages` cap) and returns per-URL results (content or a per-item `error` with hint, same pattern as `SmartSearchItem`) in one call.
    *   **Result**: Added `batch_fetch` tool (new `src/batch_fetch.rs`, one module per tool matching the existing `search.rs`/`smart_search.rs`/`fetch.rs` convention), capped at `MAX_URLS = 10`. Noticed `smart_search::fetch_one_item`'s "fetch, truncate to 2500 chars, stringify the error" logic was about to get duplicated a second time, so extracted it into `fetch_content_or_error` in new `src/fetch/multi.rs` (mirroring the `pdf.rs`/`ssrf.rs` submodule split) and had both `smart_search` and `batch_fetch` call it -- `smart_search.rs` shrank from 78 to 70 lines in the process. Also split `handlers.rs::list_tools_handler` (which had grown to 55 lines, over the 40-line limit, once `batch_fetch`'s schema was the 4th tool literal) into one small `*_tool() -> McpTool` function per tool plus a 6-line function that just collects them -- a natural split along existing per-tool boundaries, not mechanical line-chopping. Verified live: a 3-URL batch (one real, one blocked by AWU 6.1's SSRF check, one real) returned two successful items and one `error`-carrying item in a single call, confirming per-item isolation and that `batch_fetch` correctly inherits SSRF protection and caching for free (it goes through the same `fetch_url`).

*   **[✅] AWU 6.5: Page Interaction Primitives for `web_fetch`**
    *   **Objective**: `web_fetch` only supports a single passive page load; it can't click, scroll, or fill forms, so login-gated pages and infinite-scroll content are out of reach. Add a minimal, safe set of pre-extraction actions.
    *   **Scope**: `src/fetch.rs` (extend the load pipeline with an optional action-execution step before extraction), MCP tool schema (new optional `actions` parameter on `web_fetch`), design doc/discussion before implementation.
    *   **DoD**: `web_fetch` accepts an optional ordered list of simple, whitelisted actions (e.g. `{"click": "<selector>"}`, `{"scroll": "bottom"}`, `{"fill": {"selector": "...", "value": "..."}}`) applied before extraction.
    *   **Note**: highest-risk item on this list, both in complexity (action vocabulary design) and security (must NOT expose raw/arbitrary JS `eval` to the calling agent -- that would turn `web_fetch` into a remote-code-execution primitive against whatever the browser can reach, compounding the SSRF risk from AWU 6.1 if that lands first as expected). Needs its own design pass before coding starts.
    *   **Result**: Per the user's explicit scoping call before implementation, cut the vocabulary down to just `click` (CSS selector) and `scroll` (`top`/`bottom`) -- dropped `fill`/form-fill entirely along with login support, since without login a form-fill primitive had little motivating use case left and only added surface area. New `PageAction` enum (`src/fetch/actions.rs`, `#[serde(tag = "type")]` so JSON looks like `{"type":"click","selector":"..."}`) deserialized straight from the `actions` argument; `run_actions` applies them in order over the already-open `chromiumoxide::Page` with a 500ms settle delay after each, using `Page::find_element(selector).click()` for clicks and a fixed (non-agent-supplied) `window.scrollTo(...)` snippet for scroll -- never raw agent-supplied JS. `fetch_url`/`fetch_url_uncached` gained an `actions: &[PageAction]` parameter: non-empty actions now force the browser path (skip the GitHub-raw/PDF fast paths, which have nothing to interact with) and skip both `FetchCache` read and write, since the same URL can yield different content depending on which actions ran and the cache doesn't key on the action list. `smart_search`/`batch_fetch` are unaffected -- both go through `fetch_content_or_error`, which always passes `&[]`. Extracted the GitHub-raw/PDF dispatch itself into new `src/fetch/fast_path.rs` (moving `get_github_raw_url`/`fetch_raw_content` out of `fetch.rs`) purely to keep `fetch.rs`/`fetch_url_uncached` under CODING.md's 300-line/40-line limits once the actions step was added; its test moved to a new `src/fetch/fast_path/tests.rs` companion. Verified live: `click` on a nonexistent selector returns a clear `Action failed: ... no matching element` error (not a hang or a silent no-op); `scroll` succeeds; confirmed by code inspection (the `if actions.is_empty()` gate around both the cache read and write) that actions correctly bypass caching.

*   **[✅] AWU 6.6: Scraping Reliability Hardening**
    *   **Objective**: No `robots.txt` awareness, a single hardcoded User-Agent, no proxy support, and no retry/backoff on transient failures. Fine at low volume; all four become real problems at scale.
    *   **Scope**: `src/fetch.rs`, `src/browser.rs`, `Cargo.toml` (robots.txt parser crate).
    *   **DoD**: `web_fetch` checks `robots.txt` disallow rules before fetching (with an env-var opt-out for operators who accept the tradeoff, since this is a policy choice, not just a technical one); User-Agent varies instead of one fixed string; `HTTP_PROXY`/`HTTPS_PROXY` env vars are honored; transient network failures get one automatic retry with backoff before surfacing as an error.
    *   **Result**: All four landed:
        - **robots.txt**: New `src/fetch/robots.rs` using `texting_robots` (chosen over `robotstxt`/`robots_txt` for its simple `Robot::new(agent, bytes).allowed(url)` API). Fetches the host's `robots.txt` with a 5s timeout and fails *open* (allows the fetch) on any missing/unreachable/unparseable robots.txt -- standard crawler convention, and the only sane default for something so easy to misconfigure into a false block. `WEB_FETCH_IGNORE_ROBOTS` env var (checked first, before any network call) skips the check entirely, since whether robots.txt should even apply to a single agent-directed fetch (vs. bulk crawling) is a policy call some operators may want to make themselves. New `FetchError::RobotsDisallowed`.
        - **User-Agent variation**: New `src/user_agent.rs` -- a small fixed pool of realistic UA strings, one picked pseudo-randomly (seeded from `SystemTime` subsec-nanos, no new RNG dependency needed for this) the first time `user_agent()` is called and cached in a `OnceLock` for the rest of the process's life. Wired into `browser.rs`'s Chromium launch args and every `reqwest` client that previously hardcoded the same literal UA string (`search.rs`, `fetch/pdf.rs`, `fetch/fast_path.rs`, and the new `fetch/robots.rs`).
        - **Proxy support**: Turned out to be half-already-done -- `reqwest` honors `HTTP_PROXY`/`HTTPS_PROXY`/lowercase variants by default with zero code changes, so every HTTP client (search, PDF, GitHub raw, robots.txt) already had this. Only Chromium needed real work, since it doesn't read those env vars itself: added `proxy_from_env()` to `browser.rs`, passing `--proxy-server=<value>` at launch when set.
        - **Retry with backoff**: New `open_and_load_page_with_retry` retries the browser navigation once, after a 2s backoff, but only on failures that look transient (`Timeout`/`Navigation`/`PageCreation`) -- not on failures where retrying the identical request is known to be pointless, like `Blocked` (CAPTCHA) or `SsrfBlocked`.
        - Extracted `open_and_load_page`/`wait_for_page_load`/`wait_for_document_ready`/`wait_for_content_stable` (previously inline in `fetch.rs`) into new `src/fetch/navigate.rs` alongside the new retry wrapper, purely to keep `fetch.rs` under CODING.md's 300-line limit once `robots.rs`'s call site and the retry wrapper were added.
        - Verified live against real sites: GitHub's actual `robots.txt` (`Disallow: /*/*/forks` under `User-agent: *`) correctly blocks `https://github.com/torvalds/linux/forks` with a clear error, and `WEB_FETCH_IGNORE_ROBOTS=1` correctly lets the identical request through to the browser. Retry wasn't separately live-tested (hard to deterministically trigger a transient failure on demand); verified by code review instead -- the match arms only cover the three error variants that are genuinely retry-worthy.

*   **[✅] AWU 5.9: Switch Search Backend to Bing-Only, Rename `google_search` → `web_search`**
    *   **Objective**: Replace the DuckDuckGo-primary/Google-fallback search implementation with a single Bing backend for better result quality and simpler code (user's explicit preference), and rename the tool to `web_search` since `google_search` was already inaccurate (it never used Google as primary) and would become more so.
    *   **Scope**: `src/search.rs`, `src/search/tests.rs`, `src/smart_search.rs`, `src/handlers.rs`, `src/fetch.rs` (one hint string), `tests/integration_test.rs`, `README.md`, `ARCHITECTURE.md`, `Cargo.toml`.
    *   **DoD**: `web_search`/`smart_search` return real Bing results end-to-end; `cargo build`/`test`/`clippy --all-targets -- -D warnings` clean; no remaining `google_search`/DuckDuckGo references outside historical `PLANS.md` log entries.
    *   **Result**: Investigated `bing.com/search`'s plain HTML response first -- it no longer server-renders results (client-side rendered, zero result links in the raw HTML) -- but discovered Bing's documented `&format=rss` output mode returns clean, stable, structured XML instead, which is more robust than HTML/CSS-selector scraping. Rewrote `search.rs` around this: dropped `SearchError::Selector`/`BothBackendsFailed`/DuckDuckGo+Google fallback logic entirely, added `SearchError::RequestFailed`, and parse the RSS via `quick-xml`'s serde integration (`BingRss`/`BingChannel`/`BingItem` structs) instead of manual XML handling. Removed the now-unused `scraper` dependency, added `quick-xml` (with `serialize` feature). Renamed `perform_google_search` → `perform_web_search`, the MCP tool `google_search` → `web_search`, and `call_google_search` → `call_web_search` in `handlers.rs`; updated the tool's `McpTool` description, `tests/integration_test.rs`'s assertions, and one `web_fetch` CAPTCHA-block hint string that recommended `google_search`. `search.rs` shrank from 236 to 138 lines. Verified live via the compiled binary: `web_search` returns ~10 real Bing results per query (vs. 5 capped on the old DuckDuckGo path), `smart_search` correctly chains search + concurrent fetch on top of it. Updated README.md/ARCHITECTURE.md's tool descriptions, diagram, and Technical Stack tables accordingly.

*   **[✅] AWU 5.8: README Accuracy Pass & MIT License**
    *   **Objective**: Re-audit `README.md` against the codebase (requested independently of the AWU 5.7 pass) and fix what's found.
    *   **Scope**: `README.md`, `LICENSE` (new), `Cargo.toml`.
    *   **DoD**: No factual claim in `README.md` contradicts `src/*.rs` behavior; the license claim is backed by an actual `LICENSE` file and `Cargo.toml` metadata.
    *   **Result**: Found and fixed three mismatches: (1) README claimed "MIT License" with no `LICENSE` file or `Cargo.toml` `license` field anywhere in the repo -- added a standard MIT `LICENSE` (copyright Jun Kato, 2026) and `license = "MIT"` to `Cargo.toml`, and fixed the still-placeholder `authors = ["Your Name"]` while touching that section; (2) README claimed `google_search` returns "the top 5" results, but that cap only exists in `parse_ddg_results` -- the Google HTML fallback path (`parse_search_results`) is uncapped, so reworded to say the cap only applies to the DuckDuckGo path; (3) README's GitHub-optimization description said "if the URL is `github.com`", but `get_github_raw_url` (`src/fetch.rs`) actually requires both `github.com` *and* `/blob/` -- reworded to be precise about which GitHub URLs take the fast path. Also fixed the README's placeholder clone URL (`your-username` -> `akahmys`, matching the `origin` remote fixed earlier this session) and added the missing `reqwest` entry to README's Technical Stack table for parity with `ARCHITECTURE.md`.

*   **[✅] AWU 5.7: Dependency Cleanup & Doc/Codebase Re-sync**
    *   **Objective**: Re-verify README.md/ARCHITECTURE.md/PLANS.md against the post-AWU-5.6 codebase (module split, typed errors), and check `Cargo.toml` for drift.
    *   **Scope**: `Cargo.toml`, `README.md`, `ARCHITECTURE.md`.
    *   **DoD**: No unused direct dependencies in `Cargo.toml`; `cargo build`/`test`/`clippy --all-targets -- -D warnings` clean after removal; Technical Stack tables reflect the current error-handling design.
    *   **Result**: README/ARCHITECTURE/PLANS text itself required no fixes -- both already matched the current `handlers.rs`/typed-error structure. Found `Cargo.toml` declared 4 unused direct dependencies (`mcp-core`, `html5ever`, `encoding_rs`, `ego-tree` -- none referenced anywhere in `src/`, confirmed via grep) left over from before the AWU 1.2 rewrite off the old `mcp-uutils`/`mcp-core`-based design; removed all four and confirmed `cargo build`/`test`/`clippy --all-targets -- -D warnings` stay clean. Added `thiserror` to both README's and ARCHITECTURE's Technical Stack tables, since AWU 5.6 made it a core part of the error-handling design rather than an incidental dependency.

*   **[✅] AWU 5.6: thiserror Domain Errors & `#![deny(clippy::pedantic)]`**
    *   **Objective**: Close the last two `CODING.md`/`AUDITING.md` gaps flagged (but deliberately deferred) in AWU 5.5: model `fetch.rs`/`search.rs` errors as typed `thiserror` enums instead of `anyhow!` strings, and enforce `clippy::pedantic` crate-wide instead of leaving it unconfigured.
    *   **Scope**: `src/fetch.rs`, `src/search.rs`, `src/smart_search.rs`, `src/handlers.rs`, `src/main.rs`, `tests/integration_test.rs`.
    *   **DoD**: `fetch_url`/`perform_google_search` return typed domain errors (`FetchError`/`SearchError`) instead of `anyhow::Result`; `#![deny(clippy::pedantic)]` present in both the binary crate root and the integration-test crate; `cargo clippy --all-targets` (pedantic now denied by default, no extra flags needed) and `cargo clippy --all-targets -- -D warnings` both clean; `cargo test` green.
    *   **Result**: Added `FetchError` (11 variants) and `SearchError` (3 variants) via `#[derive(Error)]`. Every variant embeds its `Hint:` text directly in the `#[error("...")]` template and takes pre-formatted `String`/primitive fields rather than `#[source]`/`#[from]` -- deliberate, to avoid anyhow's `{:#}` alternate-chain printer at the MCP boundary (`main.rs`) re-printing the same underlying cause a second time once the typed error is auto-boxed into `anyhow::Error` via `?`. `fetch_url`, `open_and_load_page`, `get_verified_html`, `get_page_title`, `html_to_markdown` now return `Result<_, FetchError>`; `perform_google_search`, `search_with_fallback`, `parse_ddg_results`, `parse_search_results` now return `Result<_, SearchError>`. Low-level multi-step JS-evaluation plumbing (`wait_for_document_ready`/`wait_for_content_stable`) still uses `anyhow::Result` internally and gets flattened into `FetchError::Navigation` at the `open_and_load_page` boundary, matching the CODING.md split of "thiserror for domain modules, anyhow for glue needing ad hoc context propagation." Added `#![deny(clippy::pedantic)]` to `src/main.rs` and `tests/integration_test.rs`; ran `cargo clippy --fix -- -W clippy::pedantic` for the mechanical fixes (inline format-string args, redundant closures, doc-comment backticks) and hand-fixed the rest: two similarly-named test bindings renamed for clarity, a justified `#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]` (with comment) on the one cast that's provably safe (JS string length, always non-negative and far under `usize::MAX`), a `usize::try_from` instead of `as usize` for the user-supplied `max_pages` argument, and `error_to_call_result` changed to borrow its `anyhow::Error` instead of consuming it. Zero `#[allow]` outside that one justified case, per `CODING.md`'s "no unjustified bypassing" rule.

*   **[✅] AWU 5.5: Enforce CODING.md/AUDITING.md Structural Rules**
    *   **Objective**: Bring the codebase into compliance with `CODING.md`'s file/function length limits and test-separation rule, and `AUDITING.md`'s zero-clippy-warnings mandate, after an audit found several violations (partly introduced by AWU 5.3's error-hint text).
    *   **Scope**: `src/main.rs`, `src/handlers.rs` (new), `src/fetch.rs`, `src/fetch/tests.rs` (new), `src/search.rs`, `src/search/tests.rs` (new), `src/smart_search.rs`, `src/transport.rs`, `tests/integration_test.rs`, `ARCHITECTURE.md`.
    *   **DoD**: No file over 300 lines; no function over 40 lines; no `#[cfg(test)] mod tests { ... }` embedded inline in a production file; `cargo clippy --all-targets -- -D warnings` passes with zero warnings; `cargo test` green.
    *   **Result**: Split `src/main.rs`'s tool-call handling into a new `src/handlers.rs` (list_tools_handler, call_tool_handler, and one function per tool: call_smart_search/call_google_search/call_web_fetch). Split `handle_message` into one small handler per JSON-RPC method plus a shared `write_response` helper. Split `fetch_url` into `open_and_load_page`/`get_verified_html`/`get_page_title`, and `wait_for_page_load` into `wait_for_document_ready`/`wait_for_content_stable`. Split `perform_google_search` into itself plus `search_with_fallback`, and `parse_ddg_results` into itself plus `extract_ddg_result`/`resolve_ddg_redirect`. Split `perform_smart_search` into itself plus `fetch_one_item`. Moved `fetch.rs`'s and `search.rs`'s inline `#[cfg(test)] mod tests { ... }` blocks to companion files `src/fetch/tests.rs` and `src/search/tests.rs` (Rust's standard file-module resolution for `mod tests;` inside `fetch.rs`/`search.rs`), matching `CODING.md`'s own `src/module/tests.rs` example. Fixed the 3 pre-existing clippy warnings (redundant import in `transport.rs`, needless_return in `search.rs`, zombie-process warnings in `tests/integration_test.rs` via `child.wait()` after `child.kill()`). All behavior preserved -- `cargo test` and `cargo clippy --all-targets -- -D warnings` both clean. Deliberately did *not* convert `fetch.rs`/`search.rs`/`smart_search.rs` to `thiserror`-typed domain errors (CODING.md's error-handling rule) or add a blanket `#![deny(clippy::pedantic)]`: both would be large, invasive rewrites disproportionate to this audit's scope, and were flagged to the user instead of applied unilaterally.

*   **[✅] AWU 5.4: Documentation Sync (README/ARCHITECTURE/PLANS vs. implementation)**
    *   **Objective**: Audit README.md, ARCHITECTURE.md, and PLANS.md against the current `src/*.rs` behavior and fix discrepancies.
    *   **Scope**: `README.md`, `ARCHITECTURE.md`, `PLANS.md`, `src/main.rs` (tool description text).
    *   **DoD**: No documented tool schema, parameter, or architecture description contradicts the actual code.
    *   **Result**: Removed `web_fetch`'s documented `max_length` parameter (it doesn't exist; truncation is the fixed `MAX_CONTENT_LENGTH = 10000` constant in `src/fetch.rs`). Rewrote `ARCHITECTURE.md`'s search section and diagram, which still described a `chromiumoxide`/browser-driven Google search predating the AWU 4.7 rewrite to `reqwest`-based HTTP search; added the missing `smart_search` tool to the diagram. Fixed `google_search`'s `McpTool` description in `src/main.rs`, which still said "Search Google" despite DuckDuckGo being the primary backend since AWU 4.7. Also backfilled the AWU 5.2 log entry below, which had landed in `git log` (commit `b1f8102`) without a corresponding `PLANS.md` entry. Replaced the stale "Current Status" (v0.1 was already tagged/pushed as of AWU 4.2).

*   **[✅] AWU 5.3: LLM-Actionable Error Hints**
    *   **Objective**: Ensure every tool-call failure returned to the calling model includes a concrete `Hint:` clause describing what to try next, not just a bare failure description.
    *   **Scope**: `src/main.rs`, `src/fetch.rs`, `src/search.rs`, `src/smart_search.rs`.
    *   **DoD**: `cargo build`/`test`/`clippy` clean; every user-facing error path (missing/invalid arguments, unknown tool, browser launch failure, page timeout, CAPTCHA block, Readability parse failure, search backend failure) ends with a `Hint:` clause.
    *   **Result**: Added hints across all listed error paths. Switched the `CallToolResult` error text from `e.to_string()` (only the outermost anyhow context) to `format!("{:#}", e)` so the full cause chain reaches the model. Added a `SmartSearchItem.error` field so per-page fetch failures inside `smart_search` are surfaced with a reason and hint instead of silently becoming `content: None`.

*   **[✅] AWU 5.2: Reliability & Hygiene Cleanup**
    *   **Objective**: Fix a batch of reliability and dead-code issues found in review: unanswered unknown JSON-RPC methods, opaque combined search failure, accumulating Chromium profile dirs, missing GitHub-raw-fetch timeout, and stale doc/dead-code drift.
    *   **Scope**: `README.md`, `src/browser.rs`, `src/error.rs`, `src/fetch.rs`, `src/main.rs`, `src/mcp.rs`, `src/search.rs`.
    *   **DoD**: Unknown JSON-RPC methods get a proper `-32601` response instead of hanging the client; combined DuckDuckGo/Google search failure reports the real per-backend reason; `BrowserState::stop()` removes its per-process user-data-dir; GitHub raw fetch has the same timeout as the browser path; unused `BrowserError`/`AppError` variants and `ListToolsRequest` removed.
    *   **Result**: Landed in commit `b1f8102` (previously undocumented in this file — backfilled by AWU 5.4).

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

