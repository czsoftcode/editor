---
status: diagnosed
trigger: "Investigate why there is no autocomplete popup when typing `/` in the AI chat prompt"
created: 2026-03-07T00:00:00Z
updated: 2026-03-07T00:00:00Z
---

## Current Focus

hypothesis: Autocomplete popup for slash commands was never implemented
test: Read all input handling and rendering code for AI chat prompt
expecting: No code that detects `/` during typing or renders a suggestion popup
next_action: Report findings

## Symptoms

expected: When user types `/` in the AI chat prompt, a popup/dropdown appears listing available slash commands, filterable as user types more characters
actual: No popup appears; slash commands are only processed on Enter via dispatch
errors: N/A (missing feature, not a bug)
reproduction: Type `/` in AI chat prompt — nothing happens until Enter
started: Always — feature was never implemented

## Eliminated

(none needed — root cause identified on first pass)

## Evidence

- timestamp: 2026-03-07
  checked: src/app/ui/widgets/ai/chat/input.rs (the prompt input widget)
  found: Only handles Enter (send), Ctrl+J (newline), ArrowUp/Down (history). No per-keystroke detection of `/` prefix. No popup/autocomplete logic at all.
  implication: The input widget has no awareness of slash commands

- timestamp: 2026-03-07
  checked: src/app/ui/terminal/ai_chat/slash.rs (slash command system)
  found: COMMANDS array with 7 commands (help, clear, new, model, git, build, settings). dispatch() function only called from logic.rs on Send. Has fuzzy matching (levenshtein) but only for unknown commands after dispatch. No public API for "list matching commands given partial input".
  implication: Slash system is dispatch-only, no query/filter API exists

- timestamp: 2026-03-07
  checked: src/app/ui/terminal/ai_chat/logic.rs line 17-19
  found: Slash intercept happens only inside send_query_to_agent() — `if ws.ai.chat.prompt.starts_with('/') { super::slash::dispatch(ws, shared); return; }`
  implication: Slash commands are invisible until user presses Enter

- timestamp: 2026-03-07
  checked: src/app/ui/terminal/ai_chat/render.rs lines 358-378
  found: Prompt area calls AiChatWidget::ui_input() and only checks the returned `send_via_kb` bool. No post-input inspection of prompt text for `/` prefix. No popup rendering code anywhere in render_chat_content().
  implication: The render layer has no autocomplete UI

## Resolution

root_cause: Slash command autocomplete was never implemented. The entire slash system is dispatch-only (activated on Enter). Four things are missing: (1) per-frame detection of `/` prefix in the prompt text, (2) a public API on the slash module to filter/query matching commands, (3) popup UI rendering near the prompt input, (4) keyboard navigation within the popup (arrow keys, Enter to select, Escape to dismiss).
fix: N/A (investigation only)
verification: N/A
files_changed: []
