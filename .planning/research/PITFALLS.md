# Domain Pitfalls

**Domain:** Native AI Chat with streaming, tool execution, and provider abstraction in Rust/egui editor
**Researched:** 2026-03-06

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Blocking the egui UI thread with HTTP/streaming calls

**What goes wrong:** Synchronous HTTP requests (reqwest blocking) or waiting for streaming tokens inside `update()` freezes the entire editor. The UI becomes unresponsive -- no scrolling, no tab switching, no cancel button works.

**Why it happens:** egui is an immediate-mode GUI -- `update()` runs every frame. Any blocking call inside it stops the entire render loop. The current codebase already handles this correctly for WASM plugin calls (spawns `std::thread::spawn` in `logic.rs:95`), but the new native HTTP client introduces a different pattern: streaming requires *continuous* data flow, not a single fire-and-forget thread.

**Consequences:** Complete UI freeze during AI responses. Users can't cancel, can't switch tabs, can't even close the window. On Linux/Wayland the window manager may flag the app as "not responding."

**Prevention:**
1. Use `std::thread::spawn` + `mpsc::channel` for the HTTP streaming thread (project already uses this pattern)
2. The streaming thread reads NDJSON lines from reqwest and sends individual tokens/chunks via `mpsc::Sender<StreamEvent>`
3. In `update()`, use `try_recv()` in a loop (drain all available tokens per frame) -- never `recv()` (blocking)
4. Call `ctx.request_repaint()` from the streaming thread after sending each chunk (egui Context is `Send + Sync`)
5. Do NOT introduce tokio -- the project uses std threads and mpsc channels consistently. Adding an async runtime would create a parallel execution model that conflicts with existing patterns.

**Detection:** If `update()` takes >16ms, the frame budget is blown. Monitor with `egui::Frame` profiling. Any `recv()` call (not `try_recv()`) in UI code is a red flag.

**Phase:** Must be correct from the very first streaming implementation (Phase 1).

---

### Pitfall 2: Approval state machine race conditions

**What goes wrong:** The approval UI shows stale data, processes responses for the wrong action, or gets stuck in a "waiting for approval" state that never resolves. The agent thread blocks on `mpsc::Receiver::recv()` waiting for user response, but the UI either never renders the approval dialog or renders it then drops the sender.

**Why it happens:** The current approval mechanism (`PluginApprovalRequest` in `types.rs:149`) uses a one-shot `mpsc::Sender` passed through `AppAction`. The approval dialog takes ownership of the sender (`ws.pending_plugin_approval.take()`). If the dialog is re-rendered before the user clicks (which happens every frame in egui), the sender gets moved back into the `Option` on the "not handled" path (`approval.rs:81`). This is a fragile pattern:
- If the AI Chat window is closed while approval is pending, the sender is dropped, and the agent thread gets a `RecvError` -- which may or may not be handled gracefully
- If the workspace switches or the window is minimized, the dialog doesn't render, the sender stays in `pending_plugin_approval` but no UI drives it
- Multiple rapid tool calls can queue approvals in `AppAction`, but `pending_plugin_approval` is a single `Option` -- second approval overwrites the first

**Consequences:** Agent thread hangs forever (deadlock). Or approval is sent for wrong action. Or approval dialog appears for 1 frame then vanishes. Users lose trust in the tool execution system.

**Prevention:**
1. Model approval as an explicit state machine enum: `Idle | WaitingForApproval { id, details, sender } | Approved | Denied`
2. Queue approvals -- use `VecDeque<PendingApproval>` instead of `Option<PendingApproval>`
3. Handle sender drop gracefully in the agent thread: treat `RecvError` as "Deny" and clean up
4. Ensure the approval UI renders regardless of panel visibility -- if there's a pending approval, force the AI Chat visible or show a modal overlay
5. Add a timeout on the agent side: if no response in 5 minutes, treat as Deny
6. Test the "close window while approval pending" scenario explicitly

**Detection:** Agent thread stack trace shows it blocked on `recv()`. Or `pending_plugin_approval` is `Some(...)` but no approval dialog is rendering (invisible window, wrong tab, etc.).

**Phase:** Must be redesigned when building the new native tool execution (Phase 2-3).

---

### Pitfall 3: Context payload size explosion

**What goes wrong:** The `AiContextPayload` (defined in `ai/types.rs`) includes full file contents of the active file, all open file paths, git status, Cargo.toml summary, build errors, and memory keys. As users work with large files or many tabs, this payload grows to hundreds of KB or even MB. Ollama local models have limited context windows (4K-128K tokens depending on model), and the context payload alone can consume the entire window.

**Why it happens:** `AiManager::generate_context()` includes `tab.content.clone()` for the active file (`ai/mod.rs:78`). No truncation, no size check, no relevance filtering. A 5000-line Rust file is ~150KB of text, which is ~40K tokens. Combined with conversation history (`ai_conversation: Vec<(String, String)>`) that grows unbounded, the payload quickly exceeds model limits.

**Consequences:** Ollama returns truncated or garbage responses. API calls fail with context length errors. Token costs explode for paid providers. Response quality degrades as irrelevant context drowns the actual question.

**Prevention:**
1. Implement a token budget system: allocate fixed portions (e.g., 30% for context, 50% for conversation, 20% for response)
2. Truncate file content to a window around the cursor position (e.g., +/- 100 lines)
3. Only include content for the active file, not all open files
4. Implement conversation pruning: keep system prompt + last N messages, summarize older ones
5. Add a context size indicator in the UI (like the current token counter `ai_in_tokens`)
6. Make context components opt-in: let users toggle what gets included

**Detection:** Log the serialized payload size before each API call. Alert if > 50KB. Monitor Ollama responses for truncation indicators.

**Phase:** Phase 1 for basic truncation, Phase 2 for smart windowing.

---

### Pitfall 4: WASM-to-native migration breaking existing functionality

**What goes wrong:** During the transition from WASM plugins (extism) to native Rust code, the old and new systems interfere with each other. The WASM plugin system (`PluginManager`, `extism::Plugin`) has deep integration points: `HostContext`, `action_sender`, `egui_ctx`, 20+ host functions. Removing or modifying any of these while the new system isn't fully functional leaves users with no working AI.

**Why it happens:** The WASM plugin system is load-bearing:
- `logic.rs` sends queries through `plugin_manager.call()`
- `render.rs` renders responses from `PluginResponse` AppActions
- `approval.rs` handles approvals via `PluginApprovalRequest` AppActions
- Host functions (`host/*.rs`) implement file read/write/search/exec
- Settings store plugin configs in `settings.plugins` HashMap keyed by plugin ID

Replacing this incrementally is hard because the interface between UI and provider is tightly coupled through the AppAction/AppShared channel.

**Consequences:** Users lose AI functionality during migration. Old and new systems fight over shared state. "It worked yesterday" regression bugs. Half-migrated state where some features use old path, others new.

**Prevention:**
1. Build the new provider trait and Ollama implementation as a completely separate module (`src/app/ai/providers/`)
2. Keep the existing WASM path functional throughout -- new code runs in parallel, behind a feature flag or UI toggle
3. Map the new provider's output to the SAME `AppAction` variants initially -- reuse `PluginResponse`, `PluginMonologue`, `PluginUsage`
4. Only after the new provider works end-to-end, create a single "switch" commit that removes the WASM dependency
5. Extract host functions into provider-agnostic tool implementations first, then wire them to both WASM and native callers
6. Keep `extism` in Cargo.toml until the final removal phase -- don't fight dependency conflicts mid-migration

**Detection:** Run both old and new AI providers in the same build. If the old one breaks, the migration is leaking.

**Phase:** Spans the entire milestone. Phase 1 builds new alongside old. Final phase removes old.

## Moderate Pitfalls

### Pitfall 5: egui repaint storm from streaming tokens

**What goes wrong:** Each streaming token triggers `ctx.request_repaint()`, causing the editor to repaint at maximum speed (potentially 1000+ fps) during AI responses. This wastes CPU, increases power consumption, and contradicts the project's core value: "Editor nesmi zahrivat notebook v klidovem stavu."

**Why it happens:** The obvious implementation calls `request_repaint()` after every token. egui's default behavior is to only repaint on input events. Streaming breaks this by producing continuous "events."

**Prevention:**
1. Batch token updates: accumulate tokens for 50-100ms, then send a single update + repaint request
2. Use `request_repaint_after(Duration::from_millis(50))` instead of immediate `request_repaint()`
3. In `update()`, drain all pending tokens from the channel in one pass, then render once
4. After streaming ends, stop requesting repaints -- return to event-driven mode
5. Consider a `streaming_active: bool` flag that switches between event-driven and periodic repaint modes

**Detection:** Monitor CPU usage during AI streaming. If it's >20% on a modern CPU just for text rendering, there's a repaint storm.

**Phase:** Phase 1, alongside streaming implementation.

---

### Pitfall 6: Provider trait over-engineering

**What goes wrong:** The provider abstraction tries to be generic enough for Ollama, Claude API, Gemini, OpenAI, and hypothetical future providers. This leads to a trait with 15+ methods, complex associated types, and abstraction layers that make simple things hard. The trait becomes the bottleneck for every feature addition.

**Why it happens:** "We might need Claude support later" drives premature abstraction. Each provider has different authentication, streaming formats (NDJSON vs SSE vs WebSocket), tool calling schemas, and context window semantics.

**Prevention:**
1. Start with a minimal trait: `fn send_message(&self, messages: Vec<Message>, tools: Vec<Tool>) -> Receiver<StreamEvent>` -- that's it
2. Provider-specific configuration goes in the provider struct, not the trait
3. `StreamEvent` enum: `Token(String) | ToolCall { name, args } | Done { usage } | Error(String)` -- covers all providers
4. Don't abstract authentication -- each provider handles it internally
5. Add methods to the trait only when the second provider actually needs them
6. The current `AiMessage` and `AiConversation` types are already provider-agnostic -- reuse them

**Detection:** If adding Ollama support requires implementing more than 3-4 trait methods, the trait is over-engineered.

**Phase:** Phase 1 design decision. Get it right before implementing.

---

### Pitfall 7: Ollama NDJSON streaming parser fragility

**What goes wrong:** The Ollama API streams responses as newline-delimited JSON (NDJSON, `application/x-ndjson`). Each line is a complete JSON object. But network buffering can split JSON objects across TCP packets, or combine multiple objects in one read. A naive line-by-line parser breaks on partial reads.

**Why it happens:** `reqwest`'s streaming body returns chunks at TCP packet boundaries, not JSON object boundaries. A single `chunk()` call might return half a JSON line, or two complete lines, or one and a half lines. Additionally, Ollama's streaming tool call support has known inconsistencies (GitHub issue #12557) -- tool_calls may arrive as a single non-streamed response even when `stream: true`.

**Prevention:**
1. Use `reqwest::blocking::Response::bytes()` in chunks and buffer incoming data
2. Split on newlines, but keep incomplete lines in a buffer for the next chunk
3. Parse each complete line as a separate JSON object
4. Handle the case where `tool_calls` arrive in a single message (not streamed) -- don't assume every response will be chunked
5. Set `stream: true` in the request but handle both chunked and non-chunked responses
6. Add a test with simulated partial JSON chunks

**Detection:** Garbled output in the AI chat. JSON parse errors in logs. Missing tokens or duplicated text.

**Phase:** Phase 1, core streaming implementation.

---

### Pitfall 8: Conversation history serialization mismatch

**What goes wrong:** The current conversation history is stored as `Vec<(String, String)>` (prompt, response pairs) in `WorkspaceState`. The new system needs structured `AiMessage` objects (already defined in `ai/types.rs`) with roles, tool calls, tool results, and metadata. Migrating between these formats during the transition period causes data loss or rendering errors.

**Why it happens:** The old format (`ai_conversation: Vec<(String, String)>`) is a simple tuple. The new format (`AiConversation` with `Vec<AiMessage>`) has tool_call_id, tool_result_for_id, and structured content. Converting between them is lossy -- tool interactions can't be reconstructed from simple strings.

**Prevention:**
1. Switch `WorkspaceState` to use `AiConversation` from day one of the rewrite
2. Don't try to migrate old conversations -- just start fresh with new format
3. Keep the old rendering code (`AiChatWidget::ui_conversation`) working with the new data structure by implementing a compatibility adapter
4. Design the new `AiMessage` to render both old-style text and new-style structured content

**Detection:** Tool call/result messages render as raw JSON in the chat. Or conversation history is empty after switching providers.

**Phase:** Phase 1, data model change.

---

### Pitfall 9: Tool execution security regression

**What goes wrong:** The WASM plugin system has security boundaries built-in: `Blacklist` for file access, `HostState` sandboxing, and explicit host function registration. When moving to native Rust tool execution, these boundaries are bypassed because native code has full system access. A "read file" tool with a path traversal bug can read `/etc/passwd`.

**Why it happens:** WASM plugins run in a sandboxed VM (extism/wasmtime). Native Rust code runs with the same permissions as the editor process. The security model must be reimplemented at the application level.

**Prevention:**
1. Port the `Blacklist` pattern matching from `security.rs` to the new tool executor
2. All file operations must go through a path validation function that checks:
   - Path is within project root
   - Path doesn't match blacklist patterns
   - Path doesn't contain `..` that escapes project root (canonicalize first)
3. Command execution (`exec` tool) must use the same approval flow as the WASM version
4. Keep the `auto_approved_actions: HashSet` pattern for "approve always" functionality
5. Add integration tests that verify path traversal attacks are blocked

**Detection:** Tool successfully reads/writes files outside project root. `exec` runs without approval dialog.

**Phase:** Phase 2, when implementing native tool execution.

---

### Pitfall 10: Markdown rendering performance with long conversations

**What goes wrong:** The AI chat renders conversation history using `egui_commonmark::CommonMarkViewer`. As conversations grow (50+ messages with code blocks), markdown parsing and layout computation happen every frame, causing visible lag and high CPU usage.

**Why it happens:** Immediate-mode GUI re-renders everything every frame. Markdown parsing is expensive (regex, AST construction). The current code creates a new `CommonMarkViewer` for each render call (`approval.rs:208`, `render.rs`), and the `markdown_cache` helps but doesn't eliminate parsing overhead for long documents.

**Prevention:**
1. Pre-render markdown to egui layout once, cache the result, only re-render when content changes
2. Virtualize the conversation list: only render messages visible in the scroll viewport
3. For streaming messages, only re-parse the last message (the one being streamed), keep others cached
4. Limit conversation display to last N messages, with "Load more" button
5. Profile `CommonMarkViewer` performance with 100+ messages before optimizing

**Detection:** CPU usage stays elevated after streaming ends. Scrolling through long conversations is laggy. Frame time > 16ms during scroll.

**Phase:** Phase 3, after basic chat works. Optimize based on actual measurements.

## Minor Pitfalls

### Pitfall 11: Missing `request_repaint()` after background state changes

**What goes wrong:** Background threads update shared state (via `AppAction`), but the UI doesn't update until the user moves the mouse or types. AI responses appear "stuck" until interaction.

**Prevention:** Every `AppAction` push from a background thread must be followed by `ctx.request_repaint()`. The `egui_ctx` is already stored in `PluginManager` -- ensure the new native provider also has access to it.

**Phase:** Phase 1.

---

### Pitfall 12: Hardcoded Czech strings in approval UI

**What goes wrong:** The approval dialog (`approval.rs`) has hardcoded Czech strings: "Agent '{}' vyzaduje schvaleni akce", "Provest", "Schvalovat vzdy", "Zamitnout", "Odeslat", "Zrusit". The rest of the app uses fluent i18n. This will be inconsistent when the new AI chat is built.

**Prevention:** Move all strings to fluent `.ftl` files in `locales/` from day one. Add i18n keys for all approval/tool-related UI text. The test `all_lang_keys_match_english` will enforce parity across all 5 languages.

**Phase:** Phase 2, alongside approval UI rewrite.

---

### Pitfall 13: Ollama connection failure handling

**What goes wrong:** Ollama runs as a local server. If it's not running, not installed, or on a non-default port, the first API call fails silently or with a cryptic error. Users don't know what went wrong.

**Prevention:**
1. Add a connection test on provider initialization (GET `/api/tags`)
2. Show a clear error toast: "Ollama is not running. Start it with `ollama serve`"
3. Make the Ollama URL configurable in settings (default: `http://localhost:11434`)
4. The existing `ai_tool_available` HashMap and `spawn_ai_tool_check()` already check for CLI tools -- extend this to check Ollama connectivity

**Phase:** Phase 1.

---

### Pitfall 14: Multi-viewport state desynchronization

**What goes wrong:** The editor supports multiple viewports (windows) via `SecondaryWorkspace`. AI state (`ai_conversation`, `ai_loading`, `pending_plugin_approval`) lives in `WorkspaceState`, which is per-viewport. But `AppShared` (cross-viewport state) is accessed via `Arc<Mutex>`. If two viewports share a provider, one viewport's streaming can interfere with another's.

**Prevention:**
1. Each workspace gets its own streaming channel and conversation state (already the case)
2. Provider instances should be per-workspace, not shared globally
3. The `PluginManager` is currently shared via `AppShared.registry.plugins` -- the new provider should NOT be shared the same way
4. If settings change in one viewport (via `settings_version` AtomicU64), all viewports must update their provider configuration

**Phase:** Phase 1 architecture decision.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Provider trait design | Over-engineering (Pitfall 6) | Start with 1-2 methods, grow as needed |
| Ollama streaming | UI freeze (Pitfall 1), repaint storm (Pitfall 5), NDJSON parse (Pitfall 7) | Thread + channel + batched repaints |
| Native tool execution | Security regression (Pitfall 9), approval state bugs (Pitfall 2) | Port blacklist, queue approvals, explicit state machine |
| Conversation data model | History format mismatch (Pitfall 8) | Switch to AiMessage from day one |
| Context integration | Payload bloat (Pitfall 3) | Token budget, cursor-windowed file content |
| WASM removal | Breaking existing functionality (Pitfall 4) | Parallel operation, same AppAction variants |
| Long conversations | Markdown rendering perf (Pitfall 10) | Virtualize list, cache rendered output |
| i18n | Hardcoded strings (Pitfall 12) | All new strings go through fluent from start |

## Sources

- [egui + tokio discussion](https://github.com/emilk/egui/discussions/521) - Threading patterns for async in egui (HIGH confidence)
- [egui async discussions](https://github.com/emilk/egui/discussions/2010) - Community patterns for async data (HIGH confidence)
- [egui_inbox crate](https://docs.rs/egui_inbox) - Channel for async-to-egui communication (MEDIUM confidence)
- [Ollama streaming docs](https://docs.ollama.com/api/streaming) - NDJSON streaming format (HIGH confidence)
- [Ollama streaming tool calls blog](https://ollama.com/blog/streaming-tool) - Tool calling with streaming (HIGH confidence)
- [Ollama tool calling streaming issue #12557](https://github.com/ollama/ollama/issues/12557) - Known inconsistencies (HIGH confidence)
- [NDJSON parse errors in Ollama clients](https://github.com/code-yeongyu/oh-my-opencode/issues/1122) - Real-world parsing bugs (MEDIUM confidence)
- [LLM tool calling in production](https://medium.com/@komalbaparmar007/llm-tool-calling-in-production-rate-limits-retries-and-the-infinite-loop-failure-mode-you-must-2a1e2a1e84c8) - Infinite loop failure mode (MEDIUM confidence)
- [Multi-agent LLM race conditions](https://medium.com/@bhagyarana80/llm-agents-and-race-conditions-debugging-multi-tool-ai-with-langgraph-b0dcbf14fa67) - State synchronization issues (MEDIUM confidence)
- Project source code analysis: `src/app/ai/`, `src/app/registry/plugins/`, `src/app/ui/terminal/ai_chat/` (HIGH confidence)
