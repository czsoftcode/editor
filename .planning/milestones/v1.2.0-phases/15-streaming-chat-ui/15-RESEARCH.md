# Phase 15: Streaming Chat UI - Research

**Researched:** 2026-03-06
**Domain:** egui streaming chat UI, Ollama NDJSON streaming, markdown rendering, settings migration
**Confidence:** HIGH

## Summary

Phase 15 transforms the existing WASM-plugin-backed AI chat into a native streaming chat using OllamaProvider (Phase 13) and the consolidated AiState (Phase 14). The existing codebase already has all major building blocks: `StandardTerminalWindow` for the floating window, `AiChatWidget` with input/conversation/monologue/settings sub-widgets, `OllamaProvider.stream_chat()` returning `mpsc::Receiver<StreamEvent>`, `egui_commonmark` for markdown rendering, and `cancellation_token` for Escape-to-cancel.

The core work is: (1) rewiring `send_query_to_agent` from WASM plugin calls to native `OllamaProvider.stream_chat()`, (2) adding a `stream_rx: Option<mpsc::Receiver<StreamEvent>>` field to ChatState and polling it in `process_background_events`, (3) replacing hardcoded `Color32::from_rgb(...)` colors throughout chat widgets with theme-derived `ui.visuals()` colors, (4) adding an "AI" section to the Settings modal with Ollama URL/key/model/expertise/reasoning fields, and (5) migrating plugin settings to the new AI settings on startup.

**Primary recommendation:** Rewire the existing chat architecture (floating window, widget, input) to use native OllamaProvider streaming via mpsc polling in background.rs. Replace all hardcoded colors with visuals() derivatives. Add AI settings section to the existing Settings modal.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Chat is a floating window (StandardTerminalWindow) -- NOT docked into right panel
- Right panel stays for terminal tabs (Claude CLI terminals)
- Possible to open as separate viewport/window (different monitor)
- Keyboard shortcut for opening (dedicated shortcut, e.g. Ctrl+Shift+I)
- Resizable window (StandardTerminalWindow already supports it)
- Colored blocks -- user message has one background, AI response has another
- Metadata per message: timestamp (hh:mm), role label (You / model name), token count on AI responses, copy button
- Code blocks with syntax highlighting (use syntect, already in codebase) + copy button on code block
- Theme-aware colors derived from ui.visuals() -- automatically works in dark and light mode
- No hardcoded colors (replace existing Color32::from_rgb(...) in render.rs)
- Incremental markdown rendering during stream -- buffer text, re-render egui_commonmark every N tokens
- Stop button in prompt bar: during streaming Send button changes to Stop. Plus Escape as shortcut (already implemented via cancellation_token)
- After interrupting stream: preserve partial response in conversation + '[interrupted]' label
- Auto-scroll with ability to stop: auto-scroll down during generation, if user manually scrolls up autoscroll stops. 'Scroll to bottom' button to resume
- New 'AI' section in Settings dialog (modal_dialogs/settings.rs)
- Settings items: Ollama Base URL, Ollama API Key, Expertise role, Reasoning depth, default model + all settings from plugin settings
- Changes apply on Save, when URL changes immediately trigger new check on new address
- Automatic migration from plugin settings on startup -- read Ollama values from plugin settings into new AI section. Keep old values (WASM plugin not removed until Phase 17)

### Claude's Discretion
- Exact interval of re-rendering markdown during stream (every how many tokens)
- Specific theme colors for user vs AI blocks (which visuals() fields to use)
- Layout details of info bar and footer bar
- Exact keyboard shortcut for opening chat
- Migration logic -- how to detect that plugin settings haven't been migrated yet

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CHAT-01 | Hybrid CLI layout -- prompt at bottom, responses above with visual separation | Existing layout in render.rs already implements this pattern; extend with colored blocks |
| CHAT-02 | Streaming rendering -- progressive display token by token | OllamaProvider.stream_chat() returns mpsc::Receiver<StreamEvent>; poll in background.rs, append tokens to buffer |
| CHAT-03 | Dark/light mode -- theme-aware colors from ui.visuals() instead of hardcoded | Replace all Color32::from_rgb(...) in conversation.rs, render.rs with visuals() derivatives |
| CHAT-04 | Markdown in responses -- code blocks, inline code, bold/italic | egui_commonmark 0.20 with better_syntax_highlighting feature already in Cargo.toml; CommonMarkCache in AiState |
| CHAT-05 | Conversation history -- multi-turn chat with session persistence | ChatState.conversation already stores Vec<(String,String)>; extend to use AiConversation with Vec<AiMessage> |
| CHAT-06 | Input with prompt history (up/down arrows) | Already implemented in input.rs with history/history_index |
| CHAT-07 | Cancel/Stop button to interrupt generation | cancellation_token (Arc<AtomicBool>) exists; Escape handler in render.rs; add Send/Stop toggle |
| PROV-04 | Model picker -- ComboBox with available Ollama models | ai_bar.rs already has working ComboBox with ws.ai.ollama.models; reuse in chat header |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| eframe/egui | 0.31 | GUI framework | Already the project's UI framework |
| egui_commonmark | 0.20 | Markdown rendering | Already used, has better_syntax_highlighting feature |
| syntect | 5 | Syntax highlighting for code blocks | Already used for editor; egui_commonmark uses it via feature flag |
| ureq | 2 | HTTP client for Ollama API | Already used by OllamaProvider |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde/serde_json | existing | Serialization of settings/messages | For NDJSON parsing, settings persistence |
| toml | existing | Settings file format | For settings.toml persistence |
| chrono (or std::time) | std | Timestamps for messages | Use std::time::SystemTime for hh:mm display |

### No New Dependencies Needed

All required libraries are already in Cargo.toml. No new crates are needed for this phase.

## Architecture Patterns

### Recommended Changes to Existing Structure
```
src/app/
  ai/
    state.rs            -- ADD: stream_rx, streaming_buffer, auto_scroll fields to ChatState
    ollama.rs           -- EXISTING: OllamaProvider.stream_chat() already works
    provider.rs         -- EXISTING: StreamEvent enum ready
    types.rs            -- EXISTING: AiMessage, AiConversation ready
  ui/
    terminal/
      ai_chat/
        mod.rs          -- MODIFY: AiChatAction::Send -> trigger stream_chat
        logic.rs        -- REWRITE: send_query_to_agent -> use OllamaProvider directly
        render.rs       -- MODIFY: add streaming indicator, Stop button, theme colors
    widgets/ai/chat/
      conversation.rs   -- MODIFY: replace hardcoded colors with visuals()
      render.rs         -- MODIFY: replace hardcoded colors with visuals()
      input.rs          -- EXISTING: works as-is
    workspace/
      modal_dialogs/
        settings.rs     -- ADD: new "AI" category section
    background.rs       -- ADD: stream_rx polling section
  settings.rs           -- ADD: AI-specific fields (ollama_base_url, ollama_api_key, etc.)
```

### Pattern 1: Stream Polling via mpsc in background.rs
**What:** Add a new polling section in `process_background_events` that checks `ws.ai.chat.stream_rx` for incoming `StreamEvent`s each frame.
**When to use:** Every frame while streaming is active.
**Example:**
```rust
// In process_background_events, after section 4b (Ollama polling):
// --- 4c. Chat streaming ---
if let Some(ref rx) = ws.ai.chat.stream_rx {
    let mut events_this_frame = 0;
    loop {
        match rx.try_recv() {
            Ok(StreamEvent::Token(text)) => {
                ws.ai.chat.streaming_buffer.push_str(&text);
                events_this_frame += 1;
                // Re-render markdown every ~5 tokens for smooth display
                if events_this_frame % 5 == 0 {
                    // Update the last conversation entry
                    if let Some(last) = ws.ai.chat.conversation.last_mut() {
                        last.1 = ws.ai.chat.streaming_buffer.clone();
                    }
                }
            }
            Ok(StreamEvent::Done { model, prompt_tokens, completion_tokens }) => {
                ws.ai.chat.in_tokens += prompt_tokens as u32;
                ws.ai.chat.out_tokens += completion_tokens as u32;
                if let Some(last) = ws.ai.chat.conversation.last_mut() {
                    last.1 = ws.ai.chat.streaming_buffer.clone();
                }
                ws.ai.chat.streaming_buffer.clear();
                ws.ai.chat.loading = false;
                ws.ai.chat.stream_rx = None;
                break;
            }
            Ok(StreamEvent::Error(msg)) => {
                ws.toasts.push(Toast::error(format!("AI error: {}", msg)));
                ws.ai.chat.loading = false;
                ws.ai.chat.stream_rx = None;
                break;
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                ws.ai.chat.loading = false;
                ws.ai.chat.stream_rx = None;
                break;
            }
        }
    }
}
```

### Pattern 2: Theme-Aware Colors from visuals()
**What:** Derive chat block colors from `ui.visuals()` instead of hardcoding RGB values.
**When to use:** All chat UI rendering.
**Example:**
```rust
// User message block
let user_bg = ui.visuals().faint_bg_color;
let user_text = ui.visuals().text_color();

// AI response block -- slightly different shade
let ai_bg = ui.visuals().extreme_bg_color;
let ai_text = ui.visuals().text_color();

// Code block background
let code_bg = ui.visuals().code_bg_color;

// Separator/accent
let accent = ui.visuals().selection.stroke.color;

// Weak/metadata text
let meta_text = ui.visuals().weak_text_color();
```

### Pattern 3: Send/Stop Button Toggle
**What:** The Send button changes to Stop during streaming.
**Example:**
```rust
if ws.ai.chat.loading {
    if ui.button("Stop").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ws.ai.cancellation_token.store(true, std::sync::atomic::Ordering::Relaxed);
        // Mark partial response as interrupted
        if let Some(last) = ws.ai.chat.conversation.last_mut() {
            last.1 = format!("{}\n\n*[interrupted]*", ws.ai.chat.streaming_buffer);
        }
        ws.ai.chat.streaming_buffer.clear();
        ws.ai.chat.loading = false;
        ws.ai.chat.stream_rx = None;
    }
} else {
    if send_pressed {
        // Trigger streaming...
    }
}
```

### Pattern 4: Auto-scroll with User Override
**What:** Auto-scroll to bottom during streaming, stop when user scrolls up, show "scroll to bottom" button.
**Example:**
```rust
// Add to ChatState:
pub auto_scroll: bool, // default true

// In conversation scroll area:
let scroll = egui::ScrollArea::vertical()
    .id_salt("ai_chat_history")
    .stick_to_bottom(ws.ai.chat.auto_scroll)
    .show(ui, |ui| { /* render conversation */ });

// Detect manual scroll up
if scroll.state.offset.y < scroll.content_size.y - scroll.inner_rect.height() - 20.0 {
    ws.ai.chat.auto_scroll = false;
}

// Show "scroll to bottom" button when not auto-scrolling and streaming
if !ws.ai.chat.auto_scroll && ws.ai.chat.loading {
    if ui.button("v Scroll to bottom").clicked() {
        ws.ai.chat.auto_scroll = true;
    }
}

// Re-enable auto-scroll when new message is sent
// (in send logic: ws.ai.chat.auto_scroll = true;)
```

### Anti-Patterns to Avoid
- **Hardcoded colors:** Every `Color32::from_rgb(...)` in chat code must be replaced with `visuals()` derivatives. Dark mode/light mode must "just work".
- **Blocking UI thread with HTTP:** Never call `stream_chat()` on the UI thread. The existing pattern spawns a thread that sends to mpsc channel -- follow this exactly.
- **Re-rendering markdown on every token:** Too expensive. Buffer tokens and update the display every ~5 tokens or every ~50ms.
- **Storing full message history as `Vec<(String, String)>`:** The current tuple format loses metadata. Transition to `Vec<AiMessage>` internally while keeping backward-compatible display.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown rendering | Custom parser | egui_commonmark::CommonMarkViewer | Already integrated, handles code blocks, bold, italic |
| Syntax highlighting in code blocks | Custom lexer | egui_commonmark's better_syntax_highlighting feature (uses syntect) | Already enabled in Cargo.toml |
| Streaming HTTP | Custom async runtime | ureq + std::thread + BufReader::lines() | Already working in OllamaProvider.stream_chat() |
| Theme colors | Color palette constants | ui.visuals() fields | Automatic dark/light mode support |
| Floating window | Custom egui::Window wrapper | StandardTerminalWindow | Already handles resize, drag, head/body/footer |
| Input history | Custom ring buffer | Vec<String> + index | Already implemented in input.rs |

**Key insight:** Almost all infrastructure exists. This phase is primarily about wiring existing components together and improving visual quality.

## Common Pitfalls

### Pitfall 1: egui_commonmark Cache Invalidation During Streaming
**What goes wrong:** CommonMarkCache caches rendered markdown by content hash. During streaming, content changes every few tokens. If you use the same cache ID for the streaming message, the cache may thrash or show stale content.
**Why it happens:** egui_commonmark caches by ID, not content.
**How to avoid:** Use a unique ID per message for CommonMarkViewer, and consider calling `cache.clear()` or using a separate viewer ID for the currently-streaming message.
**Warning signs:** Streaming text appears frozen, then jumps to final state.

### Pitfall 2: Too Frequent UI Redraws During Streaming
**What goes wrong:** Calling `ctx.request_repaint()` on every single token causes excessive CPU usage, especially with large responses.
**Why it happens:** Each repaint triggers full UI layout including markdown re-parsing.
**How to avoid:** Batch token processing (already shown in Pattern 1). Only update the visible conversation entry every ~5 tokens. The egui event loop will naturally repaint when the mpsc channel has data.
**Warning signs:** CPU spikes to 100% during streaming, UI becomes laggy.

### Pitfall 3: Cancellation Token Not Checked in Stream Thread
**What goes wrong:** User presses Stop/Escape, token is set to true, but the streaming thread keeps reading from the HTTP response until the model finishes.
**Why it happens:** The current `stream_chat()` implementation doesn't check `cancellation_token`.
**How to avoid:** Either: (a) drop the `mpsc::Receiver` (sender thread will get `SendError` and exit), or (b) pass `Arc<AtomicBool>` to stream_chat and check it in the BufReader loop. Option (a) is simpler since dropping the Receiver on the UI side naturally stops the sender.
**Warning signs:** Stop button appears to work (UI resets) but Ollama server keeps generating, wasting GPU.

### Pitfall 4: Settings Migration Race Condition
**What goes wrong:** Plugin settings and new AI settings both exist, changes to one don't propagate to the other.
**Why it happens:** During the coexistence period (Phase 15-16), both WASM plugin settings and native AI settings exist.
**How to avoid:** One-time migration at startup: check if new AI settings fields are at defaults AND plugin settings have non-default Ollama values. Copy once, set a `migrated_ai_settings = true` flag.
**Warning signs:** User changes Ollama URL in Settings but chat uses old URL.

### Pitfall 5: ScrollArea stick_to_bottom Doesn't Update During Streaming
**What goes wrong:** Using `stick_to_bottom(true)` on ScrollArea but it only scrolls to bottom on first render, not on content updates.
**Why it happens:** egui's stick_to_bottom behavior depends on content size changes between frames.
**How to avoid:** Ensure the scroll area content actually changes size between frames (which it will if you update conversation text). Also use `scroll_area.scroll_to_bottom()` explicitly when auto_scroll is true and content changed.
**Warning signs:** User sees responses accumulating but has to manually scroll down.

## Code Examples

### Rewired send_query_to_agent (Replacing WASM Plugin)
```rust
// In logic.rs - replace the WASM plugin call with native OllamaProvider
pub fn send_query_to_agent(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if ws.ai.chat.prompt.trim().is_empty() {
        return;
    }
    if ws.ai.ollama.status != OllamaConnectionStatus::Connected {
        ws.toasts.push(Toast::error("Ollama is not connected"));
        return;
    }

    let prompt = ws.ai.chat.prompt.clone();

    // Build AiMessage list for multi-turn
    let mut messages = Vec::new();
    if !ws.ai.chat.system_prompt.is_empty() {
        messages.push(AiMessage {
            role: "system".to_string(),
            content: ws.ai.chat.system_prompt.clone(),
            ..Default::default()
        });
    }
    // Add conversation history
    for (q, a) in &ws.ai.chat.conversation {
        if !q.is_empty() {
            messages.push(AiMessage { role: "user".to_string(), content: q.clone(), ..Default::default() });
        }
        if !a.is_empty() {
            messages.push(AiMessage { role: "assistant".to_string(), content: a.clone(), ..Default::default() });
        }
    }
    // Add current prompt
    messages.push(AiMessage { role: "user".to_string(), content: prompt.clone(), ..Default::default() });

    // Push empty response slot
    ws.ai.chat.conversation.push((prompt.clone(), String::new()));
    ws.ai.chat.prompt.clear();
    ws.ai.chat.loading = true;
    ws.ai.chat.auto_scroll = true;
    ws.ai.chat.streaming_buffer.clear();
    ws.ai.cancellation_token = Arc::new(AtomicBool::new(false));

    // History
    if ws.ai.chat.history.last() != Some(&prompt) {
        ws.ai.chat.history.push(prompt);
    }
    ws.ai.chat.history_index = None;

    // Create provider and start streaming
    let config = ProviderConfig {
        base_url: ws.ai.ollama.base_url.clone(),
        model: ws.ai.ollama.selected_model.clone(),
        temperature: 0.7,
        num_ctx: 4096,
        api_key: ws.ai.ollama.api_key.clone(),
    };
    let provider = OllamaProvider::new(
        config.base_url.clone(),
        config.model.clone(),
        config.api_key.clone(),
    );
    ws.ai.chat.stream_rx = Some(provider.stream_chat(messages, config));
}
```

### Settings Modal AI Section
```rust
// In modal_dialogs/settings.rs - new AI category
fn render_ai_settings(
    ui: &mut egui::Ui,
    draft: &mut Settings,
    i18n: &I18n,
) -> bool {
    let mut changed = false;

    ui.heading(i18n.get("settings-ai-title"));
    ui.add_space(8.0);

    // Ollama Base URL
    ui.horizontal(|ui| {
        ui.label("Ollama URL:");
        changed |= ui.text_edit_singleline(&mut draft.ollama_base_url).changed();
    });

    // Ollama API Key (password field)
    ui.horizontal(|ui| {
        ui.label("API Key:");
        changed |= ui.add(
            egui::TextEdit::singleline(&mut draft.ollama_api_key)
                .password(true)
        ).changed();
    });

    // Expertise Role
    ui.horizontal(|ui| {
        ui.label("Expertise:");
        egui::ComboBox::from_id_salt("settings_ai_expertise")
            .selected_text(draft.ai_expertise.as_str())
            .show_ui(ui, |ui| {
                changed |= ui.selectable_value(&mut draft.ai_expertise, AiExpertiseRole::Junior, "Junior").changed();
                changed |= ui.selectable_value(&mut draft.ai_expertise, AiExpertiseRole::Senior, "Senior").changed();
                changed |= ui.selectable_value(&mut draft.ai_expertise, AiExpertiseRole::Master, "Master").changed();
            });
    });

    // Reasoning Depth
    ui.horizontal(|ui| {
        ui.label("Reasoning:");
        egui::ComboBox::from_id_salt("settings_ai_depth")
            .selected_text(draft.ai_reasoning_depth.as_str())
            .show_ui(ui, |ui| {
                changed |= ui.selectable_value(&mut draft.ai_reasoning_depth, AiReasoningDepth::Fast, "Fast").changed();
                changed |= ui.selectable_value(&mut draft.ai_reasoning_depth, AiReasoningDepth::Balanced, "Balanced").changed();
                changed |= ui.selectable_value(&mut draft.ai_reasoning_depth, AiReasoningDepth::Deep, "Deep").changed();
            });
    });

    changed
}
```

### New Fields for Settings struct
```rust
// In settings.rs - add to Settings struct:
#[serde(default = "default_ollama_url")]
pub ollama_base_url: String,

#[serde(default)]
pub ollama_api_key: String,

#[serde(default)]
pub ai_expertise: AiExpertiseRole,

#[serde(default)]
pub ai_reasoning_depth: AiReasoningDepth,

#[serde(default)]
pub ai_default_model: String,

#[serde(default)]
pub ai_settings_migrated: bool,

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}
```

### New Fields for ChatState
```rust
// In ai/state.rs - add to ChatState:
pub stream_rx: Option<mpsc::Receiver<StreamEvent>>,
pub streaming_buffer: String,
pub auto_scroll: bool,
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WASM plugin (extism) for AI chat | Native OllamaProvider via ureq | Phase 13 (2026-03) | Eliminates ~2000 LOC plugin overhead |
| AI state scattered in WorkspaceState | Consolidated AiState sub-struct | Phase 14 (2026-03) | Clean state access via ws.ai.* |
| Hardcoded dark-mode-only colors | Theme-aware ui.visuals() | Phase 15 (this phase) | Automatic dark/light mode support |

**Deprecated/outdated:**
- Plugin-based chat (WASM/extism): Being replaced by native OllamaProvider. Old code stays until Phase 17 removal.
- `conversation: Vec<(String, String)>`: Should transition to `Vec<AiMessage>` for richer metadata (timestamp, token count, model name).

## Open Questions

1. **Conversation data model transition**
   - What we know: Current code uses `Vec<(String, String)>` tuples. AiMessage struct exists with full metadata support.
   - What's unclear: Whether to fully migrate the conversation field or keep the tuple format and add a parallel metadata vec.
   - Recommendation: Migrate to `Vec<AiMessage>` internally. The tuple format is used in conversation.rs rendering, which will be rewritten anyway for colored blocks + metadata.

2. **egui_commonmark streaming re-render cost**
   - What we know: CommonMarkViewer renders markdown in one pass. During streaming, content changes frequently.
   - What's unclear: Exact performance characteristics of re-rendering large markdown documents every ~5 tokens.
   - Recommendation: Start with every 5 tokens, measure, adjust. For very long responses (>2000 tokens), consider rendering only the last N paragraphs during streaming.

3. **Cancellation propagation to HTTP stream**
   - What we know: Dropping the mpsc::Receiver causes the sender thread to get SendError and exit the loop. But the underlying ureq HTTP connection might still be open.
   - What's unclear: Whether ureq properly closes the connection when the BufReader is dropped mid-stream.
   - Recommendation: Rely on Receiver drop first. If Ollama keeps generating (visible in GPU usage), add explicit cancellation_token check in the stream_chat thread loop.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in #[cfg(test)] + cargo test |
| Config file | Cargo.toml (existing) |
| Quick run command | `cargo test --lib -- ai` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHAT-01 | CLI layout structure | manual-only | N/A (UI visual) | N/A |
| CHAT-02 | Streaming token processing | unit | `cargo test --lib -- ai::state` | Wave 0 |
| CHAT-03 | Theme colors from visuals | manual-only | N/A (visual verification) | N/A |
| CHAT-04 | Markdown rendering | manual-only | N/A (egui_commonmark integration) | N/A |
| CHAT-05 | Multi-turn conversation | unit | `cargo test --lib -- ai::types` | Existing |
| CHAT-06 | Input history up/down | unit | `cargo test --lib -- widgets::ai::chat::input` | Wave 0 |
| CHAT-07 | Cancel/Stop mechanism | unit | `cargo test --lib -- ai::ollama` | Existing |
| PROV-04 | Model picker from API | unit | `cargo test --lib -- ai::ollama::tests` | Existing |

### Sampling Rate
- **Per task commit:** `cargo test --lib -- ai`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `src/app/ai/state.rs` tests -- test streaming_buffer append, auto_scroll reset logic
- [ ] `src/settings.rs` tests -- test AI settings migration from plugin settings
- [ ] `src/settings.rs` tests -- test serde roundtrip for new AI fields

## Sources

### Primary (HIGH confidence)
- Project codebase direct inspection: ai/provider.rs, ai/ollama.rs, ai/state.rs, ai/types.rs
- Project codebase: terminal/ai_chat/ (mod.rs, render.rs, logic.rs) -- existing chat architecture
- Project codebase: widgets/ai/chat/ (conversation.rs, input.rs, render.rs, settings.rs)
- Project codebase: background.rs -- mpsc polling patterns
- Project codebase: settings.rs -- Settings struct and persistence
- Cargo.toml: egui 0.31, egui_commonmark 0.20, syntect 5, ureq 2

### Secondary (MEDIUM confidence)
- egui visuals() API: panel_fill, faint_bg_color, extreme_bg_color, text_color(), weak_text_color(), code_bg_color, selection.stroke.color -- standard egui 0.31 fields
- egui_commonmark CommonMarkViewer::new().show() API -- used throughout codebase

### Tertiary (LOW confidence)
- egui_commonmark streaming performance with frequent re-renders -- needs empirical testing
- ureq connection cleanup on BufReader drop mid-stream -- needs validation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, versions confirmed in Cargo.toml
- Architecture: HIGH -- extending existing patterns (mpsc polling, StandardTerminalWindow, AiState)
- Pitfalls: MEDIUM -- streaming performance and cancellation behavior need empirical validation

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (stable, no fast-moving dependencies)
