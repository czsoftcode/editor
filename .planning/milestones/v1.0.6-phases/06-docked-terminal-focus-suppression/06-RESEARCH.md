# Phase 6: Docked Terminal Focus Suppression - Research

**Researched:** 2026-03-05
**Domain:** egui focus management, terminal widget keyboard capture
**Confidence:** HIGH

## Summary

Phase 6 addresses a focus-stealing bug where docked terminals (right Claude panel, bottom build panel) hijack keyboard input from modal dialogs and AI Chat. The root cause is twofold: (1) `TerminalAction::Hovered` triggers `focused_panel` changes, causing hover-to-focus behavior, and (2) float-mode terminal paths pass `focused` without `dialog_open` guards, so modals cannot suppress terminal keyboard capture.

The fix is surgical: remove hover-to-focus entirely (Hovered never changes `focused_panel`), add `dialog_open` guards to float-mode paths, and ensure AI Chat TextEdit gets `request_focus()` on open. No new libraries, abstractions, or architectural changes needed -- the existing `dialog_open_base` + `FocusedPanel` pattern is correct and just needs consistent application.

**Primary recommendation:** Remove all `Hovered => focused_panel = X` branches across all terminal callers; add `!dialog_open` guard to float-mode `focused` parameter; use `request_focus()` for AI Chat TextEdit autofocus.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Hover-to-focus na terminal se kompletne rusi (docked i float, vsechny terminaly)
- `TerminalAction::Hovered` NIKDY nemeni `focused_panel`
- Terminal prebira fokus pouze pres `TerminalAction::Clicked` nebo klavesovou zkratku (Ctrl+Alt+B/A)
- Toto plati univerzalne -- ne jen kdyz je otevreny modal/AI chat
- `show_ai_chat` se NESMI pridavat do `dialog_open_base`
- AI Chat neblokuje editor ani terminal -- uzivatel muze psat do AI Chatu, editoru i terminalu
- AI Chat je bezny panel, ne modalni okno
- Pri otevreni AI Chatu se TextEdit okamzite dostane fokus (autofocus)
- `FocusedPanel::AiChat` se nastavi pri interakci s AI Chat TextEdit
- Terminal nedostane `focused = true` kdyz `focused_panel == AiChat`
- `dialog_open` blokuje terminalovy fokus (klik i hover) -- terminal nedostane fokus pri otevrenem modalu
- Terminal vola `request_focus()` kazdy frame kdyz `focused == true` -- toto musi byt potlaceno pri `dialog_open`
- Informativni modaly (About) se zavrou klikem mimo modal
- Datove modaly (Settings, conflict, staged) se NEZAVROU klikem mimo -- vyzaduji explicitni zavreni (X / Cancel / Save)

### Claude's Discretion
- Presny mechanismus potlaceni `request_focus()` v `TerminalView::focus()` -- bud uprava `focused` parametru v callerech, nebo guard v samotnem widgetu
- Zda je potreba upravit `instance/mod.rs` keyboard forwarding (radky 346-373) nebo staci spravne nastaveni `focused` v callerech
- Implementace "klik mimo zavre modal" pro informativni modaly

### Deferred Ideas (OUT OF SCOPE)
- Focus indikator (vizualni highlight aktivniho panelu) -- UX vylepseni, ne bug fix
- Konfigurovatelne hover-to-focus (settings toggle) -- pokud by nekdo hover-to-focus chtel zpet
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FSUP-01 | Docked terminal (pravy i spodni) neprebira klavesovy fokus pri otevrenem modalnim okne | Existing `dialog_open` guard in docked path is correct; float path and Hovered handler need fix |
| FSUP-02 | Docked terminal neprebira klavesovy fokus pri otevrenem AI Chat panelu | AI Chat sets `FocusedPanel::AiChat`, terminal `focused` already checks `focused_panel == Build/Claude`; Hovered handler is the leak |
| AICF-01 | AI Chat TextEdit drzi klavesovy fokus po celou dobu, kdy je chat otevreny | `request_focus()` on TextEdit response already implemented via `ai_focus_requested`; needs to persist each frame or on re-entry |
| AICF-02 | Uzivatel muze psat do AI Chatu bez ztraty fokusu na terminal | Removing Hovered focus-steal + ensuring terminal `focused=false` when `focused_panel==AiChat` solves this |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| egui | 0.31.x | GUI framework, focus/input management | Already in use, all focus APIs are egui native |
| egui_term | current | Terminal widget with `set_focus()` / `request_focus()` | Already in use, `set_focus(bool)` is the control point |

### Supporting
No additional libraries needed. This is a pure logic fix within existing code.

## Architecture Patterns

### Current Focus Architecture
```
WorkspaceState
  focused_panel: FocusedPanel     // {Build, Claude, Editor, AiChat, Files}
  dialog_open_base: bool          // computed each frame from modal flags

Terminal callers compute:
  let focused = ws.focused_panel == FocusedPanel::X && !dialog_open;
  terminal.ui(ui, focused, ...)

Terminal::ui() calls:
  TerminalView::set_focus(focused)  // line 215: controls request_focus()/surrender_focus()

Terminal::ui() returns:
  TerminalAction::{Clicked, Hovered, Navigate}

Callers handle action:
  Clicked | Hovered => ws.focused_panel = FocusedPanel::X  // BUG: Hovered steals focus
```

### Recommended Fix Pattern

**Pattern 1: Remove Hover-to-Focus**
**What:** All `TerminalAction::Hovered` handlers that change `focused_panel` must be removed or made no-op.
**Where:** 4 locations across 2 files.

```rust
// BEFORE (right/mod.rs docked path, line 275-278):
Some(TerminalAction::Hovered) => {
    if !config.dialog_open {
        ws.focused_panel = FocusedPanel::Claude;
    }
}

// AFTER:
Some(TerminalAction::Hovered) => {
    // No-op: hover does not steal focus (FSUP-01/02)
}
```

**Pattern 2: Add dialog_open Guard to Float Paths**
**What:** Float-mode terminal paths currently pass `focused` without `!dialog_open` guard.
**Where:** `right/mod.rs` line 112, `bottom/mod.rs` line 67.

```rust
// BEFORE (right/mod.rs float path, line 112):
ws_arg.focused_panel == FocusedPanel::Claude,

// AFTER:
ws_arg.focused_panel == FocusedPanel::Claude && !config.dialog_open,
```

```rust
// BEFORE (bottom/mod.rs float path, line 67):
let is_focused = ws_arg.focused_panel == FocusedPanel::Build;

// AFTER:
let is_focused = ws_arg.focused_panel == FocusedPanel::Build && !dialog_open;
```
Note: `dialog_open` is passed as `_dialog_open` parameter to `render_bottom_panel` (line 19) but NOT used in float path -- needs to be un-prefixed and used.

**Pattern 3: AI Chat Autofocus**
**What:** AI Chat TextEdit gets `request_focus()` when opened.
**Where:** `ai_chat/render.rs` line 245-247 already implements this via `ai_focus_requested`.

```rust
// Already implemented:
if ws.ai_focus_requested {
    resp.request_focus();
    ws.ai_focus_requested = false;
}
```
This fires once on open. The AI Chat TextEdit will hold focus naturally because egui preserves focus on the last `request_focus()` widget until another widget claims it. With hover-to-focus removed, no terminal will steal it.

**Pattern 4: Block Terminal Click-Focus During Modal**
**What:** `TerminalAction::Clicked` must also be suppressed when `dialog_open` is true.
**Where:** All `Clicked` handlers in float paths.

```rust
// Float path Clicked handler:
TerminalAction::Clicked => {
    if !dialog_open {
        ws_arg.focused_panel = FocusedPanel::Build;
    }
}
```

### Anti-Patterns to Avoid
- **Adding `show_ai_chat` to `dialog_open_base`:** AI Chat is NOT a modal. It must coexist with editor and terminal. User explicitly decided this.
- **Building a FocusStack/FocusManager abstraction:** Over-engineering. The `focused_panel` enum + `dialog_open` pattern is sufficient.
- **Modifying `Terminal::ui()` internals for focus guards:** The `focused` parameter is already the control point. Fix the callers, not the widget.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Focus management system | Central FocusStack | Existing `FocusedPanel` enum + `dialog_open` guard | Already works for docked paths; just needs consistency |
| Modal dialog framework | Custom modal overlay system | Existing `egui::Window` + `dialog_open_base` boolean | Pattern is established, just needs float-path guards |

## Common Pitfalls

### Pitfall 1: Forgetting Float Path Guards
**What goes wrong:** Docked paths have `!dialog_open` guard but float paths don't. Terminal steals focus from modals when in float mode.
**Why it happens:** Float and docked are separate code branches in `right/mod.rs` and `bottom/mod.rs`.
**How to avoid:** Audit ALL 4 code paths (docked-right, float-right, docked-bottom, float-bottom) for consistent `!dialog_open` guard.
**Warning signs:** Terminal captures keyboard input when Settings modal is open and terminal is in float mode.

### Pitfall 2: Hovered Action Still Emitted
**What goes wrong:** `Terminal::ui()` still emits `TerminalAction::Hovered` (line 258-259 of instance/mod.rs). If a caller accidentally handles it, focus stealing returns.
**Why it happens:** The action enum value is still generated; only callers are changed.
**How to avoid:** Either (a) make all callers explicitly no-op on Hovered, or (b) remove Hovered from the match arms entirely (compiler will warn if unhandled).
**Warning signs:** Any `Hovered =>` branch that modifies `focused_panel`.

### Pitfall 3: AI Chat Focus Lost on Re-render
**What goes wrong:** egui might not preserve focus across frames if TextEdit ID changes or widget is recreated.
**Why it happens:** `request_focus()` is called once (via `ai_focus_requested`) but if the response ID changes, focus is lost.
**How to avoid:** Ensure TextEdit uses a stable `id_salt` / `Id`. The current implementation uses egui's auto-ID which is stable for the same widget in the same layout position.
**Warning signs:** Typing in AI Chat loses focus after first character.

### Pitfall 4: Bottom Panel `_dialog_open` Parameter Unused in Float
**What goes wrong:** `render_bottom_panel` receives `_dialog_open` (prefixed with underscore), but the float path (line 66-86) never uses it. Terminal in float mode ignores modals.
**Why it happens:** Parameter was added for docked path but float path was not updated.
**How to avoid:** Remove underscore prefix, use `dialog_open` in float path `is_focused` calculation.

### Pitfall 5: Keyboard Forwarding Still Active
**What goes wrong:** `instance/mod.rs:346-373` forwards keyboard events to PTY when `focused && !self.exited`. If `focused` is incorrectly true, keys go to terminal.
**Why it happens:** This is downstream of the `focused` parameter. If callers set it correctly, this code is fine.
**How to avoid:** Fix `focused` calculation in callers. No need to modify `instance/mod.rs` keyboard forwarding logic.

## Code Examples

### All Locations Requiring Changes

#### 1. right/mod.rs -- Float path (line 110-119)
```rust
// Current: no dialog_open guard, Hovered steals focus
let terminal_action = terminal.ui(
    ui,
    ws_arg.focused_panel == FocusedPanel::Claude, // MISSING: && !config.dialog_open
    config.font_size,
    i18n,
);
if let Some(act) = terminal_action {
    match act {
        TerminalAction::Clicked | TerminalAction::Hovered => { // BUG: Hovered steals
            ws_arg.focused_panel = FocusedPanel::Claude;
        }
```

#### 2. right/mod.rs -- Docked path (line 264-278)
```rust
// Current: dialog_open guard EXISTS on focused, but Hovered still steals
// focused param is correct: config.focused == FocusedPanel::Claude && !config.dialog_open
// But Hovered handler overrides focused_panel:
Some(TerminalAction::Hovered) => {
    if !config.dialog_open {
        ws.focused_panel = FocusedPanel::Claude; // BUG: hover steals focus
    }
}
```

#### 3. bottom/mod.rs -- Float path (line 66-86)
```rust
// Current: no dialog_open guard at all
let is_focused = ws_arg.focused_panel == FocusedPanel::Build; // MISSING: && !dialog_open
// Hovered steals:
TerminalAction::Clicked | TerminalAction::Hovered => {
    ws_arg.focused_panel = FocusedPanel::Build;
}
```

#### 4. bottom/mod.rs -- Docked path (line 149-162)
```rust
// Current: dialog_open guard EXISTS on focused, but Hovered still steals
let is_focused = ws.focused_panel == FocusedPanel::Build && !dialog_open; // OK
// But:
Some(TerminalAction::Hovered) => {
    if !dialog_open {
        ws.focused_panel = FocusedPanel::Build; // BUG: hover steals focus
    }
}
```

### Summary of Changes

| File | Path | Change |
|------|------|--------|
| `terminal/right/mod.rs` | Float (line 112) | Add `&& !config.dialog_open` to focused |
| `terminal/right/mod.rs` | Float (line 118-119) | Remove Hovered focus steal |
| `terminal/right/mod.rs` | Docked (line 275-278) | Remove Hovered focus steal |
| `terminal/bottom/mod.rs` | Float (line 67) | Add `&& !dialog_open` to is_focused |
| `terminal/bottom/mod.rs` | Float (line 72-73) | Remove Hovered focus steal, add dialog guard to Clicked |
| `terminal/bottom/mod.rs` | Docked (line 157-160) | Remove Hovered focus steal |
| `terminal/bottom/mod.rs` | Param (line 19) | Rename `_dialog_open` to `dialog_open` |

Total: ~7 localized edits across 2 files. No structural changes.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hover-to-focus for terminals | Click-only focus | This phase | Prevents accidental focus theft |
| Inconsistent dialog_open guards | Uniform guards on all paths | This phase | Modals reliably block terminal input |

## Open Questions

1. **Informative modal "click outside to close"**
   - What we know: About dialog should close on click outside. Settings/conflict/staged should NOT.
   - What's unclear: egui Window doesn't have built-in "click outside to close". Needs manual implementation.
   - Recommendation: Check if `!inner.response.rect.contains(pointer_pos) && pointer.any_click()` works. This is a minor addition, Claude's discretion per CONTEXT.md.

2. **Should `TerminalAction::Hovered` variant be removed from enum?**
   - What we know: No caller should act on it after this phase.
   - What's unclear: Whether other code (viewport mode in Phase 8) might need it.
   - Recommendation: Keep the enum variant but make all callers no-op. Phase 8 can decide for viewport mode.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | `Cargo.toml` (test profile) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FSUP-01 | Docked terminal no focus steal with modal open | manual-only | N/A -- requires GUI interaction | N/A |
| FSUP-02 | Docked terminal no focus steal with AI Chat open | manual-only | N/A -- requires GUI interaction | N/A |
| AICF-01 | AI Chat TextEdit holds focus | manual-only | N/A -- requires GUI interaction | N/A |
| AICF-02 | User can type in AI Chat without terminal capture | manual-only | N/A -- requires GUI interaction | N/A |

**Justification for manual-only:** All 4 requirements involve egui widget focus behavior which requires a running GUI event loop. egui does not provide a headless test harness for focus/input simulation. The fix is purely in focus logic at call sites (boolean conditions), not in complex business logic.

### Sampling Rate
- **Per task commit:** `cargo test --lib` (verify no compilation errors or regressions)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** Manual UAT: open Settings modal, verify terminal doesn't capture keys; open AI Chat, verify typing works.

### Wave 0 Gaps
None -- no automated test infrastructure needed. All requirements are GUI-interaction tests verified via manual UAT. Existing `cargo test` covers regression (i18n key parity, etc.).

## Sources

### Primary (HIGH confidence)
- Direct source code analysis of `terminal/right/mod.rs`, `terminal/bottom/mod.rs`, `terminal/instance/mod.rs`, `terminal/ai_chat/mod.rs`, `terminal/ai_chat/render.rs`, `workspace/mod.rs`
- `FocusedPanel` enum definition in `app/types.rs`
- `StandardTerminalWindow` focus management in `terminal/window.rs`

### Secondary (MEDIUM confidence)
- egui `request_focus()` / `surrender_focus()` semantics from egui documentation -- last widget to call `request_focus()` wins for that frame

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new libraries, pure code-level fix
- Architecture: HIGH - existing patterns analyzed directly from source, changes are surgical
- Pitfalls: HIGH - each pitfall identified from actual code inspection with line numbers

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable -- no external dependency changes expected)
