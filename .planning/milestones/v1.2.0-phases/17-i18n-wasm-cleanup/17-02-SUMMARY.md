---
phase: 17-i18n-wasm-cleanup
plan: 02
subsystem: cleanup
tags: [wasm, extism, plugin-removal, i18n, dead-code]

requires:
  - phase: 17-01
    provides: "Renamed AI i18n keys from plugin-prefixed to cli-prefixed"
provides:
  - "Clean codebase without WASM plugin system (~6500 LOC deleted)"
  - "No extism dependency in Cargo.toml"
  - "No plugin-related AppAction variants or PluginManager"
  - "No plugin i18n keys in any locale file"
affects: []

tech-stack:
  added: []
  patterns:
    - "AI settings read directly from top-level Settings fields (not plugin config maps)"
    - "Registry::new() takes no arguments (no project_root needed for plugins)"

key-files:
  created: []
  modified:
    - "Cargo.toml"
    - "src/app/registry/mod.rs"
    - "src/app/types.rs"
    - "src/app/mod.rs"
    - "src/settings.rs"
    - "src/app/ui/workspace/state/mod.rs"
    - "src/app/ui/workspace/state/init.rs"
    - "src/app/ui/workspace/menubar/file.rs"
    - "src/app/ui/workspace/menubar/mod.rs"
    - "src/app/ui/widgets/command_palette.rs"
    - "src/app/ui/terminal/ai_chat/approval.rs"
    - "src/app/ui/terminal/ai_chat/render.rs"
    - "src/app/ui/terminal/ai_chat/mod.rs"

key-decisions:
  - "AI init uses top-level Settings fields (ollama_base_url, ai_expertise) instead of plugin config maps"
  - "Old WASM approval/ask_user UI removed; native Phase 16 tool approval UI retained"
  - "Plugin bar simplified to AI chat launch + settings button (no plugin combobox)"

patterns-established:
  - "Registry::new() is parameterless - no project_root dependency"

requirements-completed: [CLEN-02]

duration: 14min
completed: 2026-03-06
---

# Phase 17 Plan 02: WASM Plugin System Removal Summary

**Complete removal of legacy WASM plugin system (extism, PluginManager, 39 files, ~6500 LOC deleted) with clean compilation and all 182 tests passing**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-06T19:07:41Z
- **Completed:** 2026-03-06T19:22:24Z
- **Tasks:** 2
- **Files modified:** 52 (39 deleted, 13 modified)

## Accomplishments
- Deleted entire WASM plugin infrastructure: src/app/registry/plugins/ (~1931 LOC), src/plugins/ (~822 LOC), docs/samples/hello-plugin/
- Removed extism dependency from Cargo.toml, PluginManager from Registry, Plugin* AppAction variants
- Removed PluginSettings, plugins HashMap, ai_settings_migrated flag, and migrate_plugin_ai_settings() from Settings
- Removed plugins modal dialog, menu entry, command palette Plugin variant
- Removed old WASM approval/ask_user UI (kept native Phase 16 tool approval UI)
- Removed ~25 plugin i18n keys from ui.ftl and menu.ftl across all 5 languages
- Updated AI state initialization to read from top-level Settings fields instead of plugin config maps

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove WASM plugin code, types, and UI references** - `fb18b9d` (feat)
2. **Task 2: Remove plugin i18n keys and run full test suite** - `733cf3c` (chore)

## Files Created/Modified
- `Cargo.toml` - Removed extism, added smallvec serde feature
- `src/app/registry/mod.rs` - Removed PluginManager, CommandAction::Plugin, plugins field
- `src/app/types.rs` - Removed Plugin* AppAction variants and PluginApprovalResponse
- `src/app/mod.rs` - Removed plugin initialization, authorization, action handlers
- `src/settings.rs` - Removed PluginSettings, plugins, ai_settings_migrated, migrate function
- `src/app/ui/workspace/state/mod.rs` - Removed show_plugins, plugins_draft, plugin_error, pending_plugin_approval, pending_ask_user
- `src/app/ui/workspace/state/init.rs` - Updated AI init to use top-level Settings
- `src/app/ui/workspace/menubar/file.rs` - Removed Plugins submenu
- `src/app/ui/workspace/menubar/mod.rs` - Removed plugins/run_plugin action fields
- `src/app/ui/widgets/command_palette.rs` - Removed Plugins CommandId, Plugin action handler
- `src/app/ui/terminal/ai_chat/approval.rs` - Removed old WASM approval/ask_user UIs
- `src/app/ui/terminal/ai_chat/render.rs` - Removed old approval UI calls
- `src/app/ui/panels.rs` - Simplified AI bar (no plugin combobox)
- `locales/*/ui.ftl` - Removed ~25 plugin keys per language
- `locales/*/menu.ftl` - Removed menu-file-plugins* keys

## Decisions Made
- AI initialization now reads directly from top-level Settings fields (ollama_base_url, ai_expertise, ai_reasoning_depth) rather than going through plugin config maps -- simpler and more maintainable
- Old WASM approval/ask_user UI functions removed entirely; native Phase 16 tool approval UI is the only approval mechanism now
- Plugin bar in left panel simplified to just "AI Chat" launch + Settings button

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed smallvec missing serde feature**
- **Found during:** Task 1
- **Issue:** SmallVec<[f32; 384]> in SemanticSnippet failed Serialize/Deserialize -- pre-existing bug exposed when extism removal changed compile graph
- **Fix:** Added `features = ["serde"]` to smallvec dependency in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** cargo check passes cleanly
- **Committed in:** fb18b9d (Task 1 commit)

**2. [Rule 1 - Bug] Fixed orphaned plugin-related references in non-plan files**
- **Found during:** Task 1-2
- **Issue:** panels.rs, ai_panel.rs, ai_bar.rs, ai_dialogs.rs, and a dead ai.rs file referenced deleted plugin types/keys
- **Fix:** Simplified plugin bar, replaced plugins-unknown-agent references, cleaned ai_dialogs.rs, deleted dead ai.rs
- **Files modified:** src/app/ui/panels.rs, src/app/ui/ai_panel.rs, src/app/ui/terminal/right/ai_bar.rs, src/app/ui/workspace/modal_dialogs/ai_dialogs.rs
- **Verification:** cargo check and cargo test pass
- **Committed in:** fb18b9d, 733cf3c

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both auto-fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Codebase is clean of WASM plugin references
- All 182 tests pass
- Editor compiles and runs without WASM runtime
- Phase 17 is complete (both plans executed)

---
*Phase: 17-i18n-wasm-cleanup*
*Completed: 2026-03-06*
