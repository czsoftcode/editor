# Phase 19: Slash Command Infrastructure - Context

**Gathered:** 2026-03-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Slash command dispatch system in the chat panel. Users type `/command` in the chat prompt, the system intercepts it before the AI model, executes the command, and displays the result in the conversation. Built-in commands: /help, /clear, /new, /model, /git, /build, /settings. Error handling for unknown commands with fuzzy suggestions. GSD commands are a separate phase (Phase 20+).

</domain>

<decisions>
## Implementation Decisions

### Output presentation
- Slash command output appears as a **system message in the conversation** (inserted into `conversation` vec)
- System messages have a **distinct background color** (different from AI responses) in both dark and light mode
- Both the user's command AND the system response are visible in conversation (consistent with AI Q&A pattern)
- Output is **formatted as markdown** using the existing egui_commonmark renderer

### /build behavior
- /build is **non-blocking** — runs cargo build on a background thread
- Immediately shows "Building..." message in chat
- After completion, updates the message to show result: OK count + error/warning list
- Output stays in chat (does NOT redirect to build terminal)

### /git behavior
- /git shows a **full diff summary** similar to `git diff --stat`
- Includes: branch name, per-file change counts with +/- indicators, total summary line

### /model behavior
- `/model` (no args): lists all available models with the active one marked
- `/model <name>`: switches to the specified model, confirms in chat

### Parsing rules
- **Strict lowercase** — only `/help` works, not `/Help` or `/HELP`
- **No space after slash** — `/ help` is NOT a command
- First word after `/` is checked against registered commands
- If not a registered command, the **entire input goes to AI** as a normal prompt (not treated as an error)

### Error handling
- Unknown commands that match a registered prefix show **fuzzy suggestions** using Levenshtein distance
- Error message format: "Unknown command: /halp. Did you mean: /help? Type /help for available commands."
- Displayed as system message in conversation

### /clear behavior
- Clears visible conversation (empties `conversation` vec)
- Resets token counters (in/out) to 0
- **Preserves** prompt history (arrow-up recall)
- Results in an empty chat view

### /new behavior
- Clears visible conversation + resets token counters (same as /clear)
- **Preserves** prompt history
- Displays the PolyCredo ASCII logo with version, model, and rank info (same as initial chat open)

### /settings behavior
- Opens the settings modal dialog (existing `show_settings_dialog` flag)

### Prompt history
- Slash commands **are recorded** in prompt history (arrow-up recalls them)
- Both `/git` and normal AI prompts share the same history

### Claude's Discretion
- Exact Levenshtein threshold and suggestion count for fuzzy matching
- Specific background colors for system messages (within dark/light theme constraints)
- Internal dispatch architecture (HashMap vs match-based vs enum)
- How /build polls for completion (channel pattern vs polling)

</decisions>

<specifics>
## Specific Ideas

- /build output should look like: "Building..." then update to "Build OK (0 errors, 2 warnings)" with listed warnings
- /git should resemble `git diff --stat` output in a code block
- /model list should mark the active model with `*` prefix
- Error messages for unknown commands should include the nearest match suggestion

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `send_query_to_agent()` in `src/app/ui/terminal/ai_chat/logic.rs` — entry point for prompt processing; slash dispatch hooks in here before AI call
- `AiChatAction::Send` in `ai_chat/mod.rs` — triggers `send_query_to_agent`; needs a pre-check for slash prefix
- `AiChatAction::NewQuery` — already implements /new-like behavior (clear + logo); can be reused for /new command
- `ChatState.conversation: Vec<(String, String)>` — holds chat history; slash command output goes here as (command, response) pairs
- `ChatState.history: Vec<String>` — prompt recall history; slash commands are added here
- `AiManager::get_logo()` — generates ASCII logo for /new command
- `build_runner::run_build_check()` — background build execution; reusable for /build
- `OllamaState.models: Vec<String>` — available models list; reusable for /model
- `egui_commonmark::CommonMarkCache` in `AiState` — markdown renderer already available
- Toast system — available but decided NOT to use for command output (conversation-based instead)

### Established Patterns
- `mpsc::Receiver<T>` for async background results (build, ollama check, model info)
- `ws.toasts.push(Toast::error(...))` for error feedback (but slash errors go to conversation instead)
- `i18n.get("key")` for localized strings — all command descriptions and error messages need i18n keys (Phase 23)
- `ws.show_settings_dialog` flag for opening settings modal

### Integration Points
- `send_query_to_agent()` in `logic.rs` — intercept at the top: if prompt starts with `/`, dispatch to slash handler instead of AI
- `handle_action()` in `ai_chat/mod.rs` — `AiChatAction::Send` branch is the trigger point
- `WorkspaceState` — holds all state needed by commands (git_branch, build_errors, ai state, root_path)
- New module: `src/app/cli/slash/` or `src/app/slash/` for command registry and handlers

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 19-slash-command-infrastructure*
*Context gathered: 2026-03-07*
