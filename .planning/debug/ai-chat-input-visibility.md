---
status: diagnosed
trigger: "AI Chat panel doesn't clearly show where to type a message"
created: 2026-03-06T00:00:00Z
updated: 2026-03-06T00:00:00Z
---

## Current Focus

hypothesis: UX confusion — two separate AI panels exist, user sees terminal panel (right) but expects chat input there
test: code reading confirmed
expecting: n/a
next_action: return diagnosis

## Symptoms

expected: Visible text input field to type AI chat messages
actual: User sees Ollama models and agent picker but no text input
errors: none
reproduction: Open right panel, see Ollama models, no text input visible
started: pre-existing UX issue, not a phase 14 regression

## Eliminated

- hypothesis: Phase 14 refactor broke input visibility
  evidence: render.rs lines 204-253 unconditionally render prompt input; refactor only changed field paths (ws.ai.chat.* etc), no conditional logic was altered
  timestamp: 2026-03-06

## Evidence

- checked: Right panel (right/mod.rs) vs AI Chat window (ai_chat/mod.rs)
  found: Two completely separate AI panels exist
  implication: User confusion stems from panel architecture

- checked: ai_chat/render.rs lines 204-253
  found: Text input (TextEdit::multiline) is ALWAYS rendered in the AI Chat window — no conditional hiding
  implication: Input is not missing, it's in a different window

- checked: ai_chat/mod.rs line 31
  found: AI Chat window only shows when ws.show_ai_chat == true (default: false)
  implication: User must explicitly open the AI Chat window

- checked: right/mod.rs + ai_bar.rs
  found: Right panel only contains: Ollama status dot, model picker ComboBox, agent picker ComboBox, Start button, and terminal tabs
  implication: No text input exists in the right panel — it's a terminal-only panel

- checked: panels.rs lines 160-172
  found: Left panel "Start" button opens AI Chat window (sets show_ai_chat=true)
  implication: There's a separate flow to open the chat window with text input

## Resolution

root_cause: UX architecture confusion — there are two separate AI-related panels and the user is looking at the wrong one
fix: n/a (research only)
verification: n/a
files_changed: []
