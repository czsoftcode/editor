# Architecture Patterns

**Domain:** AI Chat Rewrite -- Native Providers, Streaming, Tool Execution for eframe/egui editor
**Researched:** 2026-03-06
**Overall confidence:** HIGH (direct codebase analysis of all integration points)

## Current Architecture (Baseline)

### How the WASM Plugin System Works Today

Current flow: `User prompt` -> `send_query_to_agent()` -> `PluginManager.call()` -> `WASM plugin (extism)` -> HTTP to API -> response -> `AppAction::PluginResponse` -> UI update.

Key participants:
- **WorkspaceState** (~30 AI fields): `ai_prompt`, `ai_conversation`, `ai_monologue`, `ai_loading`, `ai_cancellation_token`, `ai_selected_provider`, `pending_plugin_approval`, `pending_ask_user`, etc.
- **AppAction enum**: 7 plugin-related variants (`PluginResponse`, `PluginMonologue`, `PluginUsage`, `PluginPayload`, `PluginApprovalRequest`, `PluginAskUser`, `PluginCompleted`)
- **PluginManager (WASM)**: Loads `.wasm` files, manages extism `Plugin` instances, routes `call()` to WASM functions
- **HostContext**: Shared context (active file, project index, semantic index, agent memory, scratch, cancellation token)
- **AI Chat UI** (`terminal/ai_chat/`): Floating `StandardTerminalWindow` with conversation history, prompt input, approval/ask_user inline UI
- **Right Panel** (`terminal/right/`): Docked/float/viewport terminal tabs running CLI agents (Gemini CLI, Claude CLI)
- **AiManager**: Generates context payload (`AiContextPayload`), system mandates, logo

### Data Flow Pattern (Existing)

```
[UI Thread]                    [Background Thread]
    |                               |
    |-- send_query_to_agent() ----->|
    |   (sets ai_loading=true,      |-- PluginManager.call()
    |    clears monologue,          |   (WASM execution, HTTP inside WASM)
    |    pushes to conversation)    |
    |                               |-- host functions callback:
    |<-- AppAction::PluginMonologue-|     log_monologue -> action_sender
    |<-- AppAction::PluginUsage ----|     log_usage -> action_sender
    |<-- AppAction::PluginPayload --|     log_payload -> action_sender
    |<-- AppAction::PluginApproval -|     request_approval -> mpsc::channel (blocks WASM thread)
    |<-- AppAction::PluginAskUser --|     ask_user -> mpsc::channel (blocks WASM thread)
    |                               |
    |<-- AppAction::PluginResponse -| (final result)
    |   (sets ai_loading=false,     |
    |    updates conversation)      |
```

The background thread sends `AppAction` variants via `action_sender` (mpsc). The UI thread picks them up in `EditorApp::update()` via `action_rx.try_recv()`.

## Recommended Architecture for v1.2.0

### New Component: `AiProvider` Trait

Replace WASM plugin indirection with a native Rust trait. The trait runs in a background thread and communicates via the **existing** `AppAction` channel.

```rust
// src/app/ai/provider.rs (NEW)

pub trait AiProvider: Send + Sync {
    /// Human-readable name for the UI picker.
    fn name(&self) -> &str;

    /// Unique identifier (e.g., "ollama", "claude", "gemini").
    fn id(&self) -> &str;

    /// Execute a chat request. Called on a background thread.
    /// Must send streaming tokens via `event_tx` as they arrive.
    /// Returns the final complete response text.
    fn chat(
        &self,
        request: AiChatRequest,
        event_tx: std::sync::mpsc::Sender<AiStreamEvent>,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<String, AiProviderError>;

    /// List available models (optional, for UI picker).
    fn available_models(&self) -> Vec<String> { vec![] }

    /// Current model name for display.
    fn model_name(&self) -> &str;
}
```

### New Component: `AiStreamEvent` Enum

Replaces the scattered `PluginMonologue`/`PluginUsage`/`PluginPayload` actions with a typed enum that maps cleanly to existing `AppAction` variants.

```rust
// src/app/ai/provider.rs (NEW)

pub enum AiStreamEvent {
    /// Streaming text token (partial response).
    Token(String),
    /// Agent's internal monologue / thinking step.
    Monologue(String),
    /// Token usage update (input, output).
    Usage(u32, u32),
    /// Tool call request from the model.
    ToolCall(AiToolCall),
    /// Raw JSON payload for inspector.
    RawPayload(String),
}

pub struct AiToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}
```

### New Component: `OllamaProvider`

```rust
// src/app/ai/providers/ollama.rs (NEW)

pub struct OllamaProvider {
    base_url: String,  // default: "http://localhost:11434"
    model: String,     // e.g., "llama3.1"
}
```

Uses `ureq` (blocking HTTP, no async runtime needed) with streaming response parsing. Runs on the background thread spawned by `send_query_to_provider()`.

Ollama API: `POST /api/chat` with `"stream": true` returns newline-delimited JSON objects, each containing a `message.content` delta.

### Modified Component: `send_query_to_provider()` (Replaces `send_query_to_agent()`)

```rust
// src/app/ui/terminal/ai_chat/logic.rs (MODIFIED)

pub fn send_query_to_provider(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
    // 1. Build AiChatRequest from ws state (reuse AiManager::generate_context)
    // 2. Clone the provider Arc from ProviderRegistry
    // 3. Spawn background thread:
    //    - Call provider.chat(request, event_tx, cancel_token)
    //    - Bridge AiStreamEvent -> AppAction via action_sender
    //    - On completion: send AppAction::AiResponse (new variant)
}
```

### Modified Component: `AppAction` Enum

Add new variants, keep old Plugin* variants during transition:

```rust
// src/app/types.rs (MODIFIED)

pub(crate) enum AppAction {
    // ... existing variants ...

    /// Streaming token from native AI provider
    AiToken(String),
    /// Final response from native AI provider
    AiResponse(Result<String, String>),
    /// Tool call requiring execution and approval
    AiToolCall(AiToolCall),
    /// Tool execution result to feed back to provider
    AiToolResult(String, Result<String, String>),
}
```

### Modified Component: `WorkspaceState` AI Fields

Consolidate the ~30 scattered fields into a sub-struct:

```rust
// src/app/ai/chat_state.rs (NEW)

pub struct AiChatState {
    pub prompt: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub conversation: AiConversation,  // reuse existing type
    pub monologue: Vec<String>,
    pub streaming_buffer: String,      // NEW: accumulates streaming tokens
    pub system_prompt: String,
    pub language: String,
    pub expertise: AiExpertiseRole,
    pub reasoning_depth: AiReasoningDepth,
    pub in_tokens: u32,
    pub out_tokens: u32,
    pub loading: bool,
    pub cancellation_token: Arc<AtomicBool>,
    pub selected_provider: String,
    pub font_scale: u32,
    pub show_settings: bool,
    pub inspector_open: bool,
    pub focus_requested: bool,
    pub last_payload: String,
    pub response: Option<String>,
    pub pending_approval: Option<PendingPluginApproval>,
    pub pending_ask_user: Option<PendingAskUser>,
}
```

This is a **refactor**, not a functional change. `WorkspaceState` keeps a single `pub ai: AiChatState` field. All existing access paths (`ws.ai_prompt` -> `ws.ai.prompt`) are mechanical renames.

### New Component: `ProviderRegistry`

```rust
// src/app/ai/provider.rs (NEW)

pub struct ProviderRegistry {
    providers: Vec<Box<dyn AiProvider>>,
    by_id: HashMap<String, usize>,
}
```

Lives in `AppShared` (alongside, not replacing, `PluginManager` during transition). Initialized at startup from settings.

### Modified Component: AI Chat UI (Streaming Support)

The existing `render_chat_content()` already uses `stick_to_bottom(true)` ScrollArea. For streaming:

1. `AiToken` events append to `ws.ai.streaming_buffer`
2. Conversation display renders `streaming_buffer` as the in-progress assistant message
3. On `AiResponse`, `streaming_buffer` is finalized into the conversation history
4. `ctx.request_repaint()` on every token event (existing pattern from `PluginMonologue`)

### Tool Execution Flow (Approval Workflow)

The existing approval pattern (`pending_plugin_approval`, `PluginApprovalResponse`) maps directly:

```
[Background Thread]              [UI Thread]
    |                                |
    |-- provider.chat() returns      |
    |   ToolCall in stream           |
    |                                |
    |-- AiStreamEvent::ToolCall ---->|
    |   (via event_tx)               |-- AppAction::AiToolCall
    |                                |   maps to pending_approval
    |                                |   (existing approval UI)
    |                                |
    |<---- mpsc response channel ----|-- User approves/denies
    |                                |
    |-- Execute tool locally ------->|
    |   (read_file, write_file,      |
    |    replace, exec, search)      |
    |                                |
    |-- Feed result back to provider |
    |   (next chat() call or         |
    |    multi-turn within same call)|
```

**Key difference from current**: Tool execution happens in the background thread (native Rust), not inside WASM. The approval channel pattern is identical.

### Context Payload (Reuse Existing)

`AiManager::generate_context()` and `AiContextPayload` are **kept as-is**. They already produce everything needed:
- Open files with content
- Build errors
- Cursor position and selection
- Git branch and status
- Cargo.toml summary
- Memory keys

The new `AiChatRequest` wraps this:

```rust
pub struct AiChatRequest {
    pub messages: Vec<AiMessage>,    // conversation history
    pub context: AiContextPayload,   // editor context
    pub tools: Vec<AiToolDeclaration>, // available tools
    pub system_prompt: String,
    pub model: Option<String>,
}
```

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `AiProvider` trait | HTTP to AI API, streaming parse | Background thread -> `event_tx` |
| `OllamaProvider` | Ollama-specific HTTP (`/api/chat`) | `ureq` HTTP client |
| `ProviderRegistry` | Stores available providers | `AppShared` (read at send time) |
| `AiChatState` | All AI chat UI state | `WorkspaceState.ai` field |
| `AiManager` | Context generation, system prompts | `WorkspaceState` (read-only) |
| `send_query_to_provider()` | Spawns background thread, bridges events | `action_sender` channel |
| `ai_chat/render.rs` | Chat UI rendering (streaming display) | `WorkspaceState.ai` fields |
| `ai_chat/approval.rs` | Tool approval UI (existing, reused) | `pending_approval` / mpsc |
| `tools.rs` | Tool declarations (existing, reused) | `AiToolDeclaration` |
| `NativeToolExecutor` | Executes tools in background thread | File system, project index |
| `right/mod.rs` | CLI terminal panel (unchanged) | `Terminal` instances |

## Data Flow Diagram (New System)

```
[User types prompt]
       |
       v
[send_query_to_provider()]
       |
       |-- Clone provider from ProviderRegistry
       |-- Build AiChatRequest (context + history + tools)
       |-- Set ws.ai.loading = true
       |-- Spawn thread:
       |
       |   [Background Thread]
       |   |
       |   |-- provider.chat(request, event_tx, cancel)
       |   |   |
       |   |   |-- HTTP POST /api/chat (stream=true)
       |   |   |-- For each NDJSON chunk:
       |   |   |   |-- Parse delta
       |   |   |   |-- event_tx.send(AiStreamEvent::Token(delta))
       |   |   |
       |   |   |-- On tool_call in response:
       |   |   |   |-- event_tx.send(AiStreamEvent::ToolCall(call))
       |   |   |   |-- Wait for approval via mpsc
       |   |   |   |-- Execute tool (native Rust)
       |   |   |   |-- Feed result back to API (multi-turn)
       |   |   |
       |   |   |-- On completion:
       |   |       |-- event_tx.send(AiStreamEvent::Usage(in, out))
       |   |
       |   |-- Bridge loop: event_tx -> action_sender.send(AppAction::Ai*)
       |   |-- Final: action_sender.send(AppAction::AiResponse(result))
       |
       v
[EditorApp::update() -- action_rx.try_recv()]
       |
       |-- AppAction::AiToken(t):
       |   ws.ai.streaming_buffer += t
       |   ctx.request_repaint()
       |
       |-- AppAction::AiToolCall(call):
       |   ws.ai.pending_approval = Some(...)
       |   (renders approval UI)
       |
       |-- AppAction::AiResponse(result):
       |   ws.ai.loading = false
       |   finalize conversation
```

## Patterns to Follow

### Pattern 1: Background Thread + mpsc Channel (Existing)

**What:** All async work runs on `std::thread::spawn`, results come back via `mpsc::channel` to the UI thread.
**When:** Always. egui is single-threaded, no tokio/async runtime.
**Why reuse:** This is the established pattern for git status, build errors, file operations, and current WASM plugin calls. Adding a new async runtime would be unnecessary complexity.

### Pattern 2: AppAction Dispatch (Existing)

**What:** Background threads send `AppAction` variants via `action_sender`. `EditorApp::update()` processes them synchronously in the main loop.
**When:** For all cross-thread communication.
**Why reuse:** Centralizes state mutation in one place (main loop), prevents race conditions with `WorkspaceState`.

### Pattern 3: Blocking Approval Channel (Existing)

**What:** Background thread sends approval request with a `mpsc::Sender<Response>`, then blocks on `recv()`. UI thread renders approval UI, user clicks, sends response back.
**When:** Tool calls requiring user approval (file writes, exec commands).
**Why reuse:** Already proven pattern with `PluginApprovalRequest` and `PluginAskUser`. The approval UI (`approval.rs`) works and is theme-aware.

### Pattern 4: `ureq` for Blocking HTTP (New, Follows Existing Philosophy)

**What:** Use `ureq` crate for HTTP requests. Blocking API, no async runtime.
**When:** All provider HTTP calls (Ollama, future Claude/Gemini).
**Why:** Consistent with the "no tokio" philosophy. `ureq` supports streaming via `Read` trait on response body. Minimal dependency footprint.

```rust
// Streaming example with ureq
let response = ureq::post(&url)
    .set("Content-Type", "application/json")
    .send_json(&body)?;

let reader = std::io::BufReader::new(response.into_body().into_reader());
for line in reader.lines() {
    let line = line?;
    if cancel.load(Ordering::Relaxed) { break; }
    let chunk: OllamaChunk = serde_json::from_str(&line)?;
    event_tx.send(AiStreamEvent::Token(chunk.message.content))?;
}
```

### Pattern 5: NativeToolExecutor (New, Replaces WASM Host Functions)

**What:** The tool execution logic currently in `host/mod.rs` (read_project_file, write_project_file, replace_project_file, exec, search_project, etc.) is extracted into a standalone executor callable from the background thread.

**Why:** The WASM host functions contain the actual logic for file reading, writing, searching. This logic is sound and tested. Extract it into a non-WASM module so the provider background thread can call it directly.

```rust
// src/app/ai/tool_executor.rs (NEW)

pub struct NativeToolExecutor {
    pub project_root: PathBuf,
    pub blacklist: Arc<Mutex<Blacklist>>,
    pub project_index: Arc<ProjectIndex>,
    pub semantic_index: Arc<Mutex<SemanticIndex>>,
    pub action_sender: mpsc::Sender<AppAction>,
    pub egui_ctx: egui::Context,
    pub is_cancelled: Arc<AtomicBool>,
    pub auto_approved_actions: HashSet<String>,
    pub agent_memory: Arc<Mutex<AgentMemory>>,
    pub scratch: Arc<Mutex<HashMap<String, String>>>,
}

impl NativeToolExecutor {
    pub fn execute(&mut self, tool_call: &AiToolCall) -> Result<String, String> {
        match tool_call.name.as_str() {
            "read_project_file" => { /* existing logic from host_read_file */ }
            "write_file" => { /* request approval, then write */ }
            "replace" => { /* request approval, then replace */ }
            "exec" => { /* request approval, then execute */ }
            "search_project" => { /* existing logic */ }
            "semantic_search" => { /* existing logic */ }
            // ... etc
            _ => Err(format!("Unknown tool: {}", tool_call.name)),
        }
    }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Async Runtime (tokio/async-std)

**What:** Adding an async runtime for HTTP requests.
**Why bad:** egui runs on a synchronous main loop. Mixing async/sync creates complexity (`.block_on()`, channel bridges). The entire codebase uses `std::thread` + `mpsc`.
**Instead:** Use `ureq` (blocking HTTP) on a background `std::thread`.

### Anti-Pattern 2: Shared Mutable Provider State

**What:** Letting background threads mutate provider state while UI reads it.
**Why bad:** Race conditions, need for `Arc<Mutex<Provider>>` everywhere.
**Instead:** Provider trait is stateless per-call. Configuration comes from `Settings` (read at call time). Streaming state lives in `WorkspaceState.ai` (owned by UI thread).

### Anti-Pattern 3: Direct UI State Mutation from Background Thread

**What:** Passing `&mut WorkspaceState` to background threads.
**Why bad:** `WorkspaceState` is not `Send`. Violates ownership model.
**Instead:** Background thread sends `AppAction` events. UI thread mutates state in response.

### Anti-Pattern 4: Over-Abstracting the Provider Trait

**What:** Making the trait generic over async/sync, supporting both streaming/non-streaming, etc.
**Why bad:** YAGNI. Ollama is the first provider. Claude and Gemini can be added later with minimal trait changes.
**Instead:** Simple trait with one `chat()` method. Add complexity only when the second provider is implemented.

### Anti-Pattern 5: Breaking the Right Panel (CLI Terminal)

**What:** Replacing the right panel terminal tabs with the new AI Chat.
**Why bad:** The right panel runs CLI agents (Gemini CLI, Claude CLI). Users may want both.
**Instead:** New AI Chat is a separate modal/panel (`show_ai_chat`). Right panel terminals remain for CLI agents. Eventually the right panel may be repurposed, but that's a separate decision.

## Integration Points Summary

### Files to CREATE (New)

| File | Purpose |
|------|---------|
| `src/app/ai/provider.rs` | `AiProvider` trait, `AiStreamEvent`, `ProviderRegistry`, `AiChatRequest`, `AiProviderError` |
| `src/app/ai/providers/mod.rs` | Provider module root |
| `src/app/ai/providers/ollama.rs` | `OllamaProvider` implementation |
| `src/app/ai/chat_state.rs` | `AiChatState` sub-struct (extracted from WorkspaceState) |
| `src/app/ai/tool_executor.rs` | `NativeToolExecutor` (extracted from WASM host functions) |

### Files to MODIFY (Existing)

| File | Change | Risk |
|------|--------|------|
| `src/app/ai/mod.rs` | Add `pub mod provider; pub mod providers; pub mod chat_state; pub mod tool_executor;` | LOW -- additive |
| `src/app/types.rs` | Add `AiToken`, `AiResponse`, `AiToolCall` variants to `AppAction` | LOW -- additive |
| `src/app/mod.rs` | Handle new `AppAction` variants in `update()` match arm | LOW -- additive pattern |
| `src/app/ui/workspace/state/mod.rs` | Replace ~30 `ai_*` fields with `pub ai: AiChatState` | MEDIUM -- mechanical but widespread rename |
| `src/app/ui/terminal/ai_chat/logic.rs` | Replace `send_query_to_agent()` with `send_query_to_provider()` | MEDIUM -- core integration point |
| `src/app/ui/terminal/ai_chat/render.rs` | Add streaming buffer display, adapt to `ws.ai.*` paths | MEDIUM -- UI changes |
| `src/settings.rs` | Add `OllamaSettings` (base_url, model, enabled) to `Settings` | LOW -- additive |
| `src/app/ui/terminal/ai_chat/mod.rs` | Update `handle_action()` for new provider flow | LOW |
| `src/app/ui/workspace/state/init.rs` | Initialize `AiChatState` | LOW |

### Files UNCHANGED

| File | Why |
|------|-----|
| `src/app/ui/terminal/right/mod.rs` | CLI terminal panel stays as-is |
| `src/app/ui/terminal/right/ai_bar.rs` | Agent picker for CLI agents stays |
| `src/app/ai/tools.rs` | Tool declarations reused directly by NativeToolExecutor |
| `src/app/ai/types.rs` | `AiContextPayload`, `AiMessage`, `AiConversation` reused as-is |
| `src/app/ui/terminal/ai_chat/approval.rs` | Approval UI reused with minimal changes |
| `src/app/ui/background.rs` | No changes (AppAction processing is in mod.rs) |

### Files to DELETE Later (After WASM Removal Phase)

| File | When |
|------|------|
| `src/app/registry/plugins/` (entire directory) | After new chat fully replaces WASM |
| `src/app/registry/plugins/host/` | After new chat fully replaces WASM |
| External `.wasm` plugin files | After new chat fully replaces WASM |

## Suggested Build Order

Based on dependency analysis:

### Phase 1: Foundation (No UI changes, no breaking changes)
1. **`AiProvider` trait + `AiStreamEvent` + `AiChatRequest`** -- defines the interface
2. **`OllamaProvider`** -- concrete implementation with `ureq`
3. **`ProviderRegistry`** -- container for providers, added to `AppShared`
4. **New `AppAction` variants** -- `AiToken`, `AiResponse`, `AiToolCall`

Testable: Unit test verifying Ollama streaming works (no UI needed).

### Phase 2: State Refactor (Mechanical, no functional change)
5. **`AiChatState` extraction** -- move ~30 fields from `WorkspaceState` into sub-struct
6. **Mechanical renames** across all files that touch `ws.ai_*` -> `ws.ai.*`

Testable: Compiles, existing WASM chat still works through old path.

### Phase 3: Bridge + Streaming UI
7. **`send_query_to_provider()`** -- background thread bridge, event->AppAction mapping
8. **Handle new `AppAction` variants** in `EditorApp::update()`
9. **Streaming buffer display** in `render_chat_content()`
10. **Provider picker in AI Chat** settings (alongside existing WASM plugin picker)

Testable: Can chat with Ollama via new UI path, see streaming tokens.

### Phase 4: Tool Execution
11. **`NativeToolExecutor`** -- extract logic from WASM host functions
12. **Wire tool calls** through existing approval UI
13. **Multi-turn tool loop** -- feed tool results back to provider

Testable: Full agentic workflow with file edits, search, and approval.

### Phase 5: Cleanup
14. **Remove WASM plugin path** for AI chat (old `send_query_to_agent()`)
15. **Remove `extism` dependency** (if no other WASM plugins remain)
16. **Clean up old `AppAction::Plugin*` variants** that are no longer used

## Scalability Considerations

| Concern | Current (1 provider) | 3 providers | 10+ models |
|---------|---------------------|-------------|------------|
| Provider init | Direct struct creation | `ProviderRegistry` loop | Same -- providers are lightweight |
| Streaming | Single `streaming_buffer` | Per-conversation (already scoped) | Same |
| Tool execution | Direct function calls | Same -- tools are provider-agnostic | Same |
| Config storage | `Settings` struct field | HashMap keyed by provider ID | Same |
| Context window | Send full context each time | Same (context payload is ~2KB) | May need truncation strategy |

## Sources

- Existing codebase analysis: `app/mod.rs`, `app/types.rs`, `app/ai/`, `app/registry/plugins/`, `app/ui/terminal/ai_chat/`, `app/ui/terminal/right/`, `app/ui/workspace/state/`, `app/ui/background.rs`, `settings.rs` (HIGH confidence -- direct code inspection)
- Ollama API documentation: `/api/chat` endpoint with NDJSON streaming (MEDIUM confidence -- training data, verify before implementation)
- `ureq` crate: blocking HTTP client for Rust, streaming via `into_reader()` (MEDIUM confidence -- verify API in current version)
- eframe/egui threading model: single-threaded UI, `ctx.request_repaint()` for cross-thread updates (HIGH confidence -- verified in codebase)
