---
status: diagnosed
phase: 17-i18n-wasm-cleanup
source: 17-01-SUMMARY.md, 17-02-SUMMARY.md
started: 2026-03-06T20:00:00Z
updated: 2026-03-06T20:15:00Z
---

## Current Test

[testing complete]

## Tests

### 1. i18n - CLI chat strings localized
expected: Open the AI chat panel. All UI labels (chat buttons, approval dialogs, inspector, conversation controls like "Copy") should display in the currently selected language. Switch language in Settings and verify labels change accordingly.
result: issue
reported: "Version, Model, Rank jsou hardcoded"
severity: major

### 2. Ollama generation parameters in Settings
expected: Open Settings, scroll to the "PolyCredo CLI" section. You should see sliders for Top-P (0.0-1.0), Top-K (integer), Repeat Penalty, and a text input for Seed. Adjusting values and saving should persist them.
result: pass

### 3. AI settings section renamed
expected: In Settings, the AI-related section should be labeled "PolyCredo CLI" (localized) instead of any old plugin-related name.
result: pass

### 4. WASM plugin menu removed
expected: In the File menu, there should be NO "Plugins" submenu or any plugin-related menu entries.
result: pass

### 5. Command palette - no plugin option
expected: Open the command palette. There should be no "Plugins" or plugin-related command available.
result: skipped

### 6. Plugin bar simplified
expected: The left-side AI bar should show just "AI Chat" launch and a Settings button. No plugin combobox or plugin selection dropdown.
result: issue
reported: "misto AI tam dej CLI, at je to odlisne od AI terminalu"
severity: major

### 7. Clean compilation and tests
expected: Running `cargo build` and `cargo test` should complete without errors or warnings related to plugins/extism/WASM.
result: issue
reported: "src/app/ai/executor.rs:955:19, src/app/ai/mod.rs:11:9, src/app/ai/mod.rs:15:9 atd. vse warning v adresari src/ai"
severity: minor

## Summary

total: 7
passed: 3
issues: 3
pending: 0
skipped: 1

## Gaps

- truth: "All CLI chat UI labels should be localized via i18n, not hardcoded"
  status: failed
  reason: "User reported: Version, Model, Rank jsou hardcoded"
  severity: major
  test: 1
  root_cause: "settings.rs has hardcoded 'Rank:', 'Depth:', 'Junior/Senior/Master', 'Fast/Balanced/Deep'; render.rs has hardcoded 'Model...', 'Filter...'; conversation.rs has hardcoded 'Version:', 'Model:', 'Rank:' pattern matching"
  artifacts:
    - path: "src/app/ui/widgets/ai/chat/settings.rs"
      issue: "Hardcoded labels Rank:, Depth: and ComboBox values Junior/Senior/Master, Fast/Balanced/Deep"
    - path: "src/app/ui/terminal/ai_chat/render.rs"
      issue: "Hardcoded placeholder 'Model...' (line 33) and 'Filter...' (line 68)"
    - path: "src/app/ui/widgets/ai/chat/conversation.rs"
      issue: "Hardcoded pattern matching for 'Version:', 'Model:', 'Rank:' (lines 255-257)"
  missing:
    - "Add i18n keys cli-chat-label-rank, cli-chat-label-depth, cli-chat-rank-junior/senior/master, cli-chat-depth-fast/balanced/deep, cli-chat-placeholder-model, cli-chat-placeholder-filter to all 5 locales"
    - "Replace hardcoded strings with tr! macro calls in settings.rs and render.rs"

- truth: "AI bar should be labeled CLI to differentiate from AI terminal"
  status: failed
  reason: "User reported: misto AI tam dej CLI, at je to odlisne od AI terminalu"
  severity: major
  test: 6
  root_cause: "i18n key cli-bar-label is set to 'AI:' in all 5 locales"
  artifacts:
    - path: "locales/en/cli.ftl"
      issue: "cli-bar-label = AI: (line 18)"
    - path: "locales/cs/cli.ftl"
      issue: "cli-bar-label = AI: (line 18)"
    - path: "locales/de/cli.ftl"
      issue: "cli-bar-label = KI: (line 18)"
    - path: "locales/ru/cli.ftl"
      issue: "cli-bar-label = ИИ: (line 18)"
    - path: "locales/sk/cli.ftl"
      issue: "cli-bar-label = AI: (line 18)"
  missing:
    - "Change cli-bar-label value to 'CLI:' in all 5 locale files"

- truth: "Build should complete without warnings in src/app/ai/"
  status: failed
  reason: "User reported: src/app/ai/executor.rs:955:19, src/app/ai/mod.rs:11:9, src/app/ai/mod.rs:15:9 atd. vse warning v adresari src/ai"
  severity: minor
  test: 7
  root_cause: "Unused import ChangeTag in executor.rs:955, unnecessary mut on child in executor.rs:835, unused re-exports pub use provider::* and pub use tools::get_standard_tools in mod.rs:11,15"
  artifacts:
    - path: "src/app/ai/executor.rs"
      issue: "Unused import ChangeTag (line 955), unnecessary mut on child (line 835)"
    - path: "src/app/ai/mod.rs"
      issue: "Unused re-exports: pub use provider::* (line 11), pub use tools::get_standard_tools (line 15)"
  missing:
    - "Remove ChangeTag from import in executor.rs:955"
    - "Remove mut from let child in executor.rs:835"
    - "Remove pub use provider::* and pub use tools::get_standard_tools from mod.rs"
