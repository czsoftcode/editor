# Phase 16: Tool Execution - Research

**Researched:** 2026-03-06
**Domain:** Ollama native tool calling, file I/O security, command execution, approval workflow
**Confidence:** HIGH

## Summary

Phase 16 transforms the AI chat from a text-only assistant into an agentic tool-calling system. The codebase already contains complete tool declarations (14 tools in `tools.rs`), approval UI (`approval.rs` with approve/deny/always + ask_user), WASM-based tool executors (`host/fs.rs`, `host/sys.rs`) with security (`security.rs`), and the `StreamEvent::ToolCall` variant is defined but unprocessed (line 285 in `background.rs`: `/* Phase 16 */`).

The core work is: (1) enable Ollama tools API in `stream_chat()`, (2) parse tool_calls from NDJSON streaming, (3) build a native tool executor (extracting logic from WASM host functions), (4) wire the tool call -> approval -> execute -> result loop in `background.rs`, (5) enhance context injection with terminal output and LSP diagnostics, (6) add security layers (path sandbox, secrets scrubbing, command blacklist, rate limiting, audit log).

**Primary recommendation:** Extract tool execution logic from existing WASM `host/fs.rs` and `host/sys.rs` into standalone functions that take `(&Path, &serde_json::Value) -> Result<String, String>`, then wire them through a new `ToolExecutor` struct that handles approval, security, rate limiting, and audit logging.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Kontext se pripoji jako system message pri KAZDE zprave -- AI vzdy vidi aktualni stav
- Stavajici AiContextPayload + PRIDAT: terminal output (poslednich N radku z aktivniho terminalu), LSP diagnostiky
- Obsah aktivniho souboru: pouzit RSG indexaci -- +-50 radku, fallback po 200 radcich
- write_file celeho souboru jen u noveho souboru -- preferovat replace
- Inline unified diff (git diff -U3) -- existujici render_diff_or_markdown v approval.rs
- Po schvaleni: okamzity zapis na disk + watcher detekuje zmenu a reloada tab
- Novy soubor = nizsi riziko, jednodussi approval; prepsani existujiciho = varovani + diff preview
- VZDY vyzadovat approval pro exec + blacklist zakazanych prikazu
- Blacklist: rm -rf /, sudo, shutdown, reboot, mkfs, dd, format
- Sitove prikazy (curl, wget, nc, ssh, scp, rsync, telnet) = approval s extra varovanim
- Timeout: 120 sekund pro exec
- Vystup inline v chatu jako code block
- Working directory: vzdy project root (ws.root_path)
- Nativni Ollama tools API -- tools parametr v /api/chat
- Jeden tool call na zpravu -- AI musi cekat na schvaleni pred dalsim
- Automaticky (bez approval): read_project_file, list_project_files, search_project, semantic_search, store_scratch, retrieve_scratch, store_fact, retrieve_fact, list_facts, delete_fact
- S approval: write_file, replace, exec
- Vlastni UI: ask_user, announce_completion
- Respektovat .gitignore -- ignorovane soubory se NEPOSILAJI do kontextu a AI je NESMI cist
- Extra blacklist v AI Settings (globalni): glob patterny pro citlive soubory
- Vychozi patterny: .env*, *.pem, *.key, id_rsa*, credentials.*, secrets.*, *.pfx, *.p12
- Vsechny cesty se kanonizuji a MUSI zacinat pod ws.root_path
- Path traversal (../, symlink ven, absolutni cesta mimo projekt) = okamzite zamitnuti + audit log
- Secrets scrubbing: regex detekce KEY=, PASSWORD=, SECRET=, TOKEN=, API_KEY= atd. -> [REDACTED]
- Rate limiting: Max 50 write/replace + 20 exec za konverzaci
- Audit log: .polycredo/ai-audit.log s casovou znackou

### Claude's Discretion
- Presny regex patterny pro secrets scrubbing
- Format audit logu (JSON vs. plain text)
- Jak zobrazit rate limit warning v UI
- Presna ikonografie pro tool blok typy (read/write/exec)
- Jak zobrazit LSP diagnostiky v kontextu (format)
- Jak extractovat terminal output (kolik radku, ktery terminal)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TOOL-01 | Automaticky editor kontext -- otevrene soubory, git stav, build errory | Existing `AiManager::generate_context()` already provides this; extend AiContextPayload with terminal_output + lsp_diagnostics fields; inject as system message every call |
| TOOL-02 | File read tool -- AI cte soubory s approval | Existing `host_read_file` logic in fs.rs provides complete implementation; read tools are auto-approved per CONTEXT.md |
| TOOL-03 | File write/replace tool -- AI upravuje soubory s approval a diff preview | Existing `host_write_file` + `host_replace_file` logic + `render_diff_or_markdown` in approval.rs; needs approval flow via pending_plugin_approval |
| TOOL-04 | Command execution tool -- AI spousti prikazy s approval | Existing `host_exec` logic in sys.rs; add blacklist, network warning, 120s timeout |
| TOOL-05 | Approval UI -- Approve/Deny/Always workflow | Existing `render_approval_ui` + `PluginApprovalResponse` enum fully functional; adapt for native tool flow |
| TOOL-06 | Ask-user tool -- AI se muze zeptat uzivatele na upresneni | Existing `render_ask_user_ui` + `host_ask_user` logic fully functional; adapt for native flow |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde_json | existing | Tool argument parsing, Ollama request/response | Already in codebase |
| similar | 2.7.0 | Diff generation for replace preview | Already in codebase, used in host_replace_file |
| regex | 1 | Secrets scrubbing, blacklist patterns, .gitignore | Already in codebase |
| walkdir | 2 | File listing for list_project_files | Already in codebase |
| ureq | existing | Ollama HTTP API with tools parameter | Already in codebase |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| globset | 0.4 | .gitignore parsing + AI blacklist glob matching | NEW -- needed for proper .gitignore and glob blacklist pattern matching |
| chrono | existing check | Audit log timestamps | Use if already in Cargo.toml, otherwise std::time::SystemTime |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| globset for .gitignore | ignore crate | `ignore` is more complete but heavier; globset is sufficient for pattern matching |
| chrono for timestamps | std::time | std::time is enough for ISO timestamps in audit log -- no new dependency needed |

**Installation:**
```bash
cargo add globset
```

## Architecture Patterns

### Recommended Project Structure
```
src/app/
  ai/
    tools.rs                 # Tool declarations (EXISTS)
    types.rs                 # AiContextPayload, AiMessage (EXISTS, EXTEND)
    provider.rs              # StreamEvent::ToolCall (EXISTS)
    ollama.rs                # OllamaProvider (EXISTS, MODIFY for tools)
    mod.rs                   # AiManager::generate_context (EXISTS, EXTEND)
    executor.rs              # NEW: ToolExecutor -- native tool execution
    security.rs              # NEW: PathSandbox, SecretsFilter, CommandBlacklist
    audit.rs                 # NEW: AuditLogger
  ui/
    terminal/ai_chat/
      approval.rs            # EXISTS -- approval + ask_user UI
      render.rs              # EXISTS -- tool block rendering additions
    background.rs            # EXISTS -- StreamEvent::ToolCall processing
    workspace/state/
      mod.rs                 # EXISTS -- add tool rate counters + approval state
```

### Pattern 1: Tool Executor with Approval Loop
**What:** A `ToolExecutor` struct that takes a tool call, checks security, dispatches to handler, manages approval flow via mpsc channels.
**When to use:** Every tool call from Ollama.
**Example:**
```rust
// Source: Derived from existing host/fs.rs + host/sys.rs patterns
pub struct ToolExecutor {
    project_root: PathBuf,
    sandbox: PathSandbox,
    secrets_filter: SecretsFilter,
    cmd_blacklist: CommandBlacklist,
    rate_limiter: RateLimiter,
    audit: AuditLogger,
    auto_approved: HashSet<String>,
}

impl ToolExecutor {
    pub fn execute(
        &mut self,
        tool_name: &str,
        args: &serde_json::Value,
        approval_tx: &mpsc::Sender<ApprovalRequest>,
        approval_rx: &mpsc::Receiver<ApprovalResponse>,
    ) -> ToolResult {
        // 1. Rate limit check
        // 2. Security checks (path sandbox, command blacklist)
        // 3. Auto-approve or request approval
        // 4. Execute tool
        // 5. Scrub secrets from output
        // 6. Audit log
        // 7. Return result
    }
}
```

### Pattern 2: Streaming Tool Call Detection in NDJSON Parser
**What:** Extend `parse_ndjson_line()` to detect `message.tool_calls` array in streaming responses.
**When to use:** In `ollama.rs` NDJSON parser.
**Example:**
```rust
// Ollama streaming tool call format (from official docs):
// {"message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"read_project_file","arguments":{"path":"src/main.rs"}}}]},"done":false}
pub fn parse_ndjson_line(line: &str) -> Option<StreamEvent> {
    // ... existing logic ...

    // Check for tool_calls in message
    if let Some(tool_calls) = parsed["message"]["tool_calls"].as_array() {
        if let Some(tc) = tool_calls.first() {
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            let arguments = tc["function"]["arguments"].clone();
            let id = format!("tc_{}", uuid_or_counter);
            return Some(StreamEvent::ToolCall { id, name, arguments });
        }
    }

    // ... existing token/done logic ...
}
```

### Pattern 3: Tool Call -> Approval -> Execute -> Resume Loop
**What:** In `background.rs`, when `StreamEvent::ToolCall` is received, pause streaming, show approval UI, execute tool, send result back to Ollama as tool message, resume streaming.
**When to use:** Core agentic loop.
**Example:**
```rust
// In background.rs chat streaming section:
StreamEvent::ToolCall { id, name, arguments } => {
    // 1. Display compact tool block in chat
    // 2. Check if auto-approved
    // 3. If needs approval: set ws.pending_tool_approval = Some(...)
    //    UI renders approval; user responds; continue
    // 4. Execute via ToolExecutor
    // 5. Build tool result message
    // 6. Append to conversation history
    // 7. Send new request to Ollama with tool result
    // 8. New stream_rx replaces current one
}
```

### Pattern 4: Ollama Tool Result Message Format
**What:** After tool execution, send result back as a "tool" role message.
**When to use:** After each tool call is executed.
**Example:**
```rust
// Ollama expects tool results as:
// {"role":"tool","content":"file content here..."}
// The full conversation must include the assistant's tool_call message too
let tool_result_msg = AiMessage {
    role: "tool".to_string(),
    content: result_text,
    tool_call_name: Some(tool_name),
    tool_result_for_id: Some(tool_call_id),
    ..Default::default()
};
```

### Anti-Patterns to Avoid
- **Running tool executor on UI thread:** Tool execution (especially exec with 120s timeout) MUST run on a background thread. Use the existing `spawn_task` pattern.
- **Sending full file content for every message:** Use RSG indexing to send only relevant +-50 lines around cursor; fallback to 200-line chunks.
- **Allowing parallel tool calls:** CONTEXT.md explicitly says one tool call per message -- AI must wait for approval and execution before next tool call.
- **Hardcoding approval categories:** Use a HashSet of auto-approved tool names so it's easy to change.
- **Blocking UI during approval wait:** The existing pattern uses `pending_plugin_approval` as an Option on WorkspaceState -- UI renders approval when Some, tool thread blocks on mpsc::Receiver.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Diff generation | Custom line-by-line diff | `similar` crate (already in Cargo.toml) | Edge cases with Unicode, whitespace, empty lines |
| Glob pattern matching | Custom wildcard matcher | `globset` crate | .gitignore patterns have complex semantics (negation, directory vs file) |
| Path canonicalization | Manual `..` stripping | `std::fs::canonicalize()` + `starts_with()` check | Symlink resolution, platform-specific edge cases |
| JSON parsing | Manual string matching | `serde_json::Value` traversal | Already used throughout codebase |
| Fuzzy string matching for replace | New matcher | Existing `find_fuzzy_match()` in `host/fs.rs` | Already battle-tested, handles whitespace normalization |

**Key insight:** Almost all tool execution logic already exists in the WASM host functions (`host/fs.rs`, `host/sys.rs`). The work is extracting it from the `extism::CurrentPlugin` API into standalone functions, then wiring through native channels instead of WASM memory.

## Common Pitfalls

### Pitfall 1: Ollama Tools + Streaming Incompatibility
**What goes wrong:** Historically, Ollama required `stream: false` when using tools. Recent versions (post-2025) support streaming with tools, but behavior varies by model.
**Why it happens:** Tool calling was added to Ollama later; some models don't support it; streaming tool calls is newer.
**How to avoid:** Start with `stream: false` for tool-enabled requests (simpler), then optimize to streaming later. Check `supports_tools` in `ProviderCapabilities`. When sending tools, if model doesn't support them, fall back to non-tool mode.
**Warning signs:** Empty tool_calls arrays, model ignoring tools entirely, malformed JSON in tool call arguments.

### Pitfall 2: Race Condition in Approval Flow
**What goes wrong:** Tool executor thread blocks waiting for approval; if user cancels the chat, the thread hangs forever.
**Why it happens:** `mpsc::Receiver::recv()` blocks indefinitely.
**How to avoid:** Use `recv_timeout()` with periodic cancellation_token checks (existing pattern in `host_ask_user`). Set a 60-second timeout for approval with auto-deny.
**Warning signs:** Chat panel freezes, "Stop" button doesn't work.

### Pitfall 3: Path Traversal via Symlinks
**What goes wrong:** AI requests `src/utils.rs` which is a symlink to `/etc/passwd`.
**Why it happens:** Simple string prefix check doesn't catch symlinks.
**How to avoid:** Always `canonicalize()` the resolved path AND verify it `starts_with()` the canonicalized project root. Existing `HostState::is_allowed()` does string-level checks but NOT symlink resolution.
**Warning signs:** File reads returning unexpected content.

### Pitfall 4: Secrets Leaking Through Build Errors
**What goes wrong:** Build error messages might contain secret values from environment variables or config files.
**Why it happens:** Compiler output can include expanded macro content or literal values.
**How to avoid:** Apply secrets scrubbing filter to ALL content sent to AI: context payload, tool results, exec output.
**Warning signs:** API keys appearing in AI conversation.

### Pitfall 5: Tool Call ID Collision
**What goes wrong:** Ollama doesn't generate unique tool call IDs -- the response only has `function.name` and `function.arguments`.
**Why it happens:** Ollama tool_calls don't have an explicit `id` field (unlike OpenAI API).
**How to avoid:** Generate IDs locally: `format!("tc_{}_{}", tool_name, counter)` with a per-conversation counter.
**Warning signs:** Tool results mapped to wrong tool calls in conversation history.

### Pitfall 6: Conversation History Explosion
**What goes wrong:** After many tool calls, the conversation history becomes huge and exceeds model context window.
**Why it happens:** Each tool call adds assistant message + tool result message to history.
**How to avoid:** Truncate tool results in history (keep first 500 chars + "[truncated]"). Consider dropping old tool exchanges after 10 rounds. Monitor token count against num_ctx.
**Warning signs:** Ollama returning errors about context length, slow responses.

## Code Examples

### Ollama Request with Tools
```rust
// Source: https://github.com/ollama/ollama/blob/main/docs/api.md
let tools_json: Vec<serde_json::Value> = get_standard_tools()
    .iter()
    .map(|t| serde_json::json!({
        "type": "function",
        "function": {
            "name": t.name,
            "description": t.description,
            "parameters": t.parameters,
        }
    }))
    .collect();

let body = serde_json::json!({
    "model": config.model,
    "messages": msgs,
    "stream": true,
    "tools": tools_json,
    "options": {
        "temperature": config.temperature,
        "num_ctx": config.num_ctx,
    }
});
```

### Sending Tool Result Back to Ollama
```rust
// Source: https://github.com/ollama/ollama/blob/main/docs/api.md
// After executing tool, append to messages:
// 1. The assistant's tool_call message:
msgs.push(serde_json::json!({
    "role": "assistant",
    "content": "",
    "tool_calls": [{
        "function": {
            "name": tool_name,
            "arguments": tool_arguments
        }
    }]
}));
// 2. The tool result:
msgs.push(serde_json::json!({
    "role": "tool",
    "content": tool_result_content
}));
// Then send new /api/chat request with full conversation
```

### Secrets Scrubbing Regex
```rust
// Recommendation for secrets detection
use regex::Regex;

lazy_static::lazy_static! {
    static ref SECRETS_RE: Regex = Regex::new(
        r"(?i)(API_KEY|SECRET|TOKEN|PASSWORD|PASSWD|DB_PASSWORD|PRIVATE_KEY|ACCESS_KEY|AUTH)\s*[=:]\s*['\"]?([^\s'\"]+)"
    ).unwrap();
}

pub fn scrub_secrets(text: &str) -> String {
    SECRETS_RE.replace_all(text, "$1=[REDACTED]").to_string()
}
```

### Path Sandbox Check
```rust
pub fn validate_path(project_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let requested = project_root.join(relative_path);
    let canonical = requested.canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;
    let root_canonical = project_root.canonicalize()
        .map_err(|e| format!("Cannot resolve root: {}", e))?;

    if !canonical.starts_with(&root_canonical) {
        return Err(format!("Path traversal blocked: {}", relative_path));
    }
    Ok(canonical)
}
```

### Command Blacklist
```rust
const BLACKLISTED_COMMANDS: &[&str] = &[
    "rm -rf /", "sudo", "shutdown", "reboot", "mkfs", "dd ", "format",
];

const NETWORK_COMMANDS: &[&str] = &[
    "curl", "wget", "nc", "ssh", "scp", "rsync", "telnet",
];

pub fn classify_command(cmd: &str) -> CommandClassification {
    let lower = cmd.to_lowercase();
    for pattern in BLACKLISTED_COMMANDS {
        if lower.contains(pattern) {
            return CommandClassification::Blocked;
        }
    }
    for pattern in NETWORK_COMMANDS {
        if lower.contains(pattern) {
            return CommandClassification::NetworkWarning;
        }
    }
    CommandClassification::NeedsApproval
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WASM plugin host functions | Native Rust tool executor | Phase 16 | No extism dependency for tool execution |
| No tools in Ollama streaming | Streaming tool calls supported | Ollama late 2024/2025 | Can stream content and detect tool calls in same response |
| `supports_tools: false` | `supports_tools: true` | Phase 16 | OllamaProvider advertises tool support |

**Deprecated/outdated:**
- WASM host functions in `registry/plugins/host/`: Will be removed in Phase 17 (CLEN-02), but logic must be extracted first
- `stream: false` requirement for tools: Ollama now supports streaming with tools

## Open Questions

1. **RSG Indexing for Active File Context**
   - What we know: CONTEXT.md mentions RSG indexation for sending relevant context. The ProjectIndex and SemanticIndex exist.
   - What's unclear: Exact API for getting +-50 lines around relevant context from the index. Need to check `semantic_index.rs` and `index.rs` implementations.
   - Recommendation: Start with simple cursor-based approach (send +-50 lines around cursor from active file), use semantic index as enhancement.

2. **Terminal Output Extraction**
   - What we know: `egui_term` manages terminal instances. Need to extract last N lines.
   - What's unclear: Whether `egui_term` exposes terminal buffer content programmatically.
   - Recommendation: Check egui_term API; if not accessible, skip terminal output for initial implementation and add as follow-up.

3. **Ollama Streaming + Tools Reliability**
   - What we know: Recent Ollama versions support streaming with tools. Format uses tool_calls array in message.
   - What's unclear: Whether all models (especially smaller ones like 7B) reliably produce well-formed tool_calls JSON in streaming mode.
   - Recommendation: Implement non-streaming tool mode first (`stream: false` when tools present), add streaming tool support as optimization.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TOOL-01 | Context payload includes all required fields | unit | `cargo test ai::types::tests -x` | Needs extension |
| TOOL-02 | File read respects path sandbox + blacklist | unit | `cargo test ai::executor::tests::test_read -x` | Wave 0 |
| TOOL-03 | File write/replace generates correct diff, respects sandbox | unit | `cargo test ai::executor::tests::test_write -x` | Wave 0 |
| TOOL-04 | Command blacklist blocks dangerous commands | unit | `cargo test ai::security::tests::test_blacklist -x` | Wave 0 |
| TOOL-05 | Approval flow approve/deny/always works correctly | unit | `cargo test ai::executor::tests::test_approval -x` | Wave 0 |
| TOOL-06 | Ask-user returns user input to tool executor | unit | `cargo test ai::executor::tests::test_ask_user -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/app/ai/executor.rs` -- new module with tests for tool execution logic
- [ ] `src/app/ai/security.rs` -- new module with tests for path sandbox, secrets scrubbing, command blacklist
- [ ] `src/app/ai/audit.rs` -- new module with tests for audit log formatting
- [ ] Extend `src/app/ai/ollama.rs` tests -- parse_ndjson_line with tool_calls
- [ ] Extend `src/app/ai/types.rs` tests -- AiContextPayload new fields

## Sources

### Primary (HIGH confidence)
- Ollama API documentation (https://github.com/ollama/ollama/blob/main/docs/api.md) -- tool call request/response format, tool result message format
- Ollama blog: Tool Support (https://ollama.com/blog/tool-support) -- tool declaration format
- Ollama blog: Streaming Tool Calls (https://ollama.com/blog/streaming-tool) -- streaming NDJSON format with tool_calls
- Codebase analysis: `src/app/ai/` -- existing tool declarations, provider, types, state
- Codebase analysis: `src/app/registry/plugins/host/` -- existing WASM tool execution logic (reusable)
- Codebase analysis: `src/app/ui/terminal/ai_chat/approval.rs` -- existing approval UI
- Codebase analysis: `src/app/ui/background.rs` -- existing streaming processing with Phase 16 placeholder

### Secondary (MEDIUM confidence)
- Ollama streaming tool calls behavior -- verified across multiple sources but model-dependent

### Tertiary (LOW confidence)
- egui_term terminal buffer API -- not verified, may need investigation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all key libraries already in Cargo.toml except globset
- Architecture: HIGH -- existing code provides clear patterns to follow; tool execution logic exists in WASM host
- Pitfalls: HIGH -- based on direct code analysis and official Ollama docs
- Ollama tool streaming: MEDIUM -- format confirmed but model reliability varies

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (stable domain, Ollama API unlikely to change drastically)
