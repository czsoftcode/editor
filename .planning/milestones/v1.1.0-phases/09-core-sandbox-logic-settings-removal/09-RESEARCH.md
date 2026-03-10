# Phase 9: Core Sandbox Logic & Settings Removal - Research

**Researched:** 2026-03-05
**Domain:** Rust refactoring -- dead code removal, serde migration, cascading compile-fix
**Confidence:** HIGH

## Summary

Phase 9 is a pure code removal phase: delete `src/app/sandbox.rs`, remove `sandbox_mode` from Settings, strip related structs/fields from workspace state, and ensure the project compiles (warnings allowed). The codebase currently has **494 sandbox references across 41 files**, but this phase targets only the **core module, settings field, and state structs** -- UI elements, file operations, and i18n cleanup are deferred to later phases.

The main risk is cascading compilation errors. Removing `sandbox.rs` breaks `WorkspaceState.sandbox` field (type `crate::app::sandbox::Sandbox`), which cascades into `init.rs`, `mod.rs` (workspace), `background.rs`, and many UI files. The strategy is: delete core module first, then fix compilation top-down by removing/stubbing dependent code. Settings migration requires a one-time load-time cleanup to strip `sandbox_mode` and `project_read_only` from persisted TOML/JSON.

**Primary recommendation:** Delete `sandbox.rs` first, then iteratively fix compilation errors file-by-file. Treat the compiler as the guide -- each error points to the next removal target.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Explicitni migrace pri loadu -- detekovat sandbox_mode/project_read_only v raw TOML/JSON, odstranit a ihned prepsat soubor
- Migrace pro oba formaty: settings.toml i settings.json (legacy)
- Oba aliasy (sandbox_mode i project_read_only) odstranit najednou v jednom kroku
- Jednorazova migrace -- po prepsani souboru pole zmizi natrvalo
- Smazat `src/app/sandbox.rs` cely + `mod sandbox` deklaraci z `app/mod.rs`
- Smazat vsechny struktury zavisle na Sandbox/SyncPlan (PendingSettingsSave, SandboxApplyRequest, SandboxPersistFailure, TabRemapRequest)
- Smazat vsech ~18 sandbox-related poli z WorkspaceState
- Smazat `sandbox_off_toast_shown` z AppShared
- Smazat 6 sandbox ToastActionKind variant + vsechny match arms
- Smazat sandbox helper funkce v settings.rs
- Smazat metody `apply_sandbox_mode_change()` a `should_apply_sandbox_request()` ze state/mod.rs
- Smazat `spawn_get_staged_files()` a `process_pending_sandbox_apply()` z workspace/mod.rs
- Smazat sandbox inicializaci z `app/mod.rs` (sandbox_root) a `state/init.rs`
- **PONECHAT pro Phase 10:** `modal_dialogs/sandbox.rs` (cely soubor -- UI dialog)
- **PONECHAT pro Phase 12:** runtime cleanup .polycredo/sandbox/ adresaru

### Claude's Discretion
- Poradi odstranovani souboru (co prvni, co pak)
- Jak resit kompilacni chyby vznikle kaskadovym mazanim
- Zda nektere sandbox-related testy migrovat nebo jen smazat

### Deferred Ideas (OUT OF SCOPE)
- Runtime cleanup .polycredo/sandbox/ adresaru -- Phase 12
- Sandbox modal dialog odstraneni -- Phase 10
- UI prvky (sandbox toggle, tooltip, build bar indikator) -- Phase 10
- File operations sandbox remapping -- Phase 11
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-01 | Soubor `src/app/sandbox.rs` je kompletne odstranen | Delete file + remove `pub mod sandbox;` from `app/mod.rs` line 14 |
| CORE-02 | Struct `Sandbox`, `SyncPlan` a vsechny sandbox metody jsou odstraneny | All defined in `sandbox.rs` (lines 6-305), referenced in 41 files -- cascading removal needed |
| SET-01 | `Settings.sandbox_mode` field odstranen | Field at `settings.rs` line 128-129, default at line 155, tests at lines 583-627 |
| SET-02 | Legacy migrace `project_read_only` odstranena | Alias at line 128, test at lines 605-627 -- replace with load-time migration |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x | Serialization/deserialization | Already in use for Settings |
| toml | 0.8.x | TOML parsing for settings migration | Already used for settings load/save |
| serde_json | 1.x | JSON parsing for legacy settings migration | Already used for legacy JSON migration |

### Supporting
No new libraries needed. This is a pure removal/refactoring phase.

## Architecture Patterns

### Recommended Deletion Order

The compiler-driven approach works best. Recommended sequence:

```
Step 1: Delete src/app/sandbox.rs
Step 2: Remove `pub mod sandbox;` from src/app/mod.rs
Step 3: Remove sandbox_root initialization from app/mod.rs (lines 122-127)
Step 4: Remove Settings.sandbox_mode field + alias + default
Step 5: Add settings migration logic (raw TOML/JSON field stripping)
Step 6: Remove sandbox structs from state/mod.rs (PendingSettingsSave, SandboxApplyRequest, SandboxPersistFailure, TabRemapRequest)
Step 7: Remove ~18 sandbox fields from WorkspaceState
Step 8: Remove sandbox_off_toast_shown from AppShared (types.rs line 182)
Step 9: Remove 6 ToastActionKind variants (types.rs lines 203-208)
Step 10: Remove apply_sandbox_mode_change() and should_apply_sandbox_request() from state/mod.rs
Step 11: Remove sandbox functions from workspace/mod.rs
Step 12: Fix remaining compile errors in init.rs, background.rs, and other files
Step 13: Remove/update sandbox-related tests
Step 14: cargo check -- iterate until clean
```

### Settings Migration Pattern

Current settings loading (`Settings::load_from_config_dir`) reads TOML/JSON and deserializes via serde. After removing the `sandbox_mode` field, existing settings files with `sandbox_mode = true` will be silently ignored by serde (`#[serde(default)]`). However, the CONTEXT.md requires **active migration** -- detecting and removing the field from files.

**Recommended approach:**

```rust
// In Settings::load_from_config_dir, after deserializing:
fn migrate_settings_file(path: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(path) else { return };

    // For TOML: parse as toml::Value, remove sandbox keys, rewrite
    if let Ok(mut table) = content.parse::<toml::Value>() {
        if let Some(t) = table.as_table_mut() {
            let had_sandbox = t.remove("sandbox_mode").is_some();
            let had_readonly = t.remove("project_read_only").is_some();
            if had_sandbox || had_readonly {
                if let Ok(new_content) = toml::to_string_pretty(&table) {
                    let _ = std::fs::write(path, new_content);
                }
            }
        }
    }
}
```

For JSON legacy files: same principle -- parse as `serde_json::Value`, remove keys, rewrite.

### Pattern: Handling Cascading Compile Errors

When field `ws.sandbox` (type `Sandbox`) is removed from `WorkspaceState`, every access site breaks. The key files that reference it:

| File | Occurrences | What to Remove |
|------|-------------|----------------|
| `workspace/mod.rs` | 70 | `trigger_sandbox_staged_refresh`, `process_pending_sandbox_apply`, `render_sandbox_staged_bar`, `render_sandbox_deletion_sync_dialog` |
| `workspace/state/init.rs` | 28 | Sandbox instance creation, staged files init, `sandbox_mode_enabled` init |
| `workspace/modal_dialogs/settings.rs` | 83 | `SandboxModeChange` enum, confirmation flow, all sandbox-related tests |
| `background.rs` | 32 | Staged files refresh, sandbox sync background tasks |
| `terminal/bottom/build_bar.rs` | 10 | Sandbox ON/OFF indicator -- **defer to Phase 10** |
| `terminal/right/mod.rs` | 6 | Claude panel sandbox references |
| `panels.rs` | 16 | Panel layout sandbox references |
| `file_tree/render.rs` | 5 | File tree sandbox toggle -- **defer to Phase 10** |

**Critical distinction:** In Phase 9, remove only the **core logic and data structures**. Where UI code references removed fields, add temporary stubs (e.g., `let sandbox_mode_enabled = false;`) to keep compilation working. Phase 10 will clean up UI.

### Anti-Patterns to Avoid
- **Removing UI elements in this phase:** The boundary is explicitly set -- `modal_dialogs/sandbox.rs` stays, build bar indicators stay. Only core logic and settings.
- **Breaking compilation without fixing:** Each removal must be followed by a compile check. Don't delete 500 lines and then try to fix.
- **Forgetting the Registry sandbox_root:** `app/mod.rs` line 127 passes `sandbox_root` to `Registry::new()`. This needs a replacement value (project root or a dummy path).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML field removal | Manual string manipulation | `toml::Value` parse + table mutation | Preserves formatting, handles edge cases |
| JSON field removal | Regex on JSON string | `serde_json::Value` parse + map removal | Correct JSON handling |

## Common Pitfalls

### Pitfall 1: Registry sandbox_root Dependency
**What goes wrong:** `Registry::new(sandbox_root)` at `app/mod.rs` line 127 requires a PathBuf. After removing sandbox, what path to use?
**Why it happens:** Registry uses this path for plugin sandbox isolation.
**How to avoid:** Check what `Registry::new()` does with this path. If it's only for sandbox file access, replace with project root. If it creates directories, use a neutral path.
**Warning signs:** Plugins failing to load after removal.

### Pitfall 2: WorkspaceState Field Count
**What goes wrong:** Removing ~18 fields from `WorkspaceState` but missing some in `init.rs` constructor.
**Why it happens:** The struct has 80+ fields -- easy to miss initializers.
**How to avoid:** The compiler catches missing fields in struct literal. Remove field from struct definition first, then fix the constructor.

### Pitfall 3: Test Cleanup
**What goes wrong:** Tests in `settings.rs` (lines 583-627) and `state/mod.rs` (lines 287-305) and `settings.rs modal` tests reference `sandbox_mode`.
**Why it happens:** Tests verify sandbox behavior that no longer exists.
**How to avoid:** Delete sandbox-specific tests entirely. Verify remaining tests still pass.

### Pitfall 4: settings_conflict and Settings Propagation
**What goes wrong:** The `sandbox_mode_enabled != settings_snapshot.sandbox_mode` comparison at `app/mod.rs` lines 757-780 and 566-590 drives the entire sandbox apply flow.
**Why it happens:** This was the core settings change detection for sandbox mode.
**How to avoid:** Remove these entire if-blocks. The `settings_conflict` struct itself (`SettingsConflict`) may still be needed for other settings changes -- verify before removing.

### Pitfall 5: modal_dialogs/sandbox.rs Must Stay
**What goes wrong:** Accidentally deleting or breaking `modal_dialogs/sandbox.rs` which is deferred to Phase 10.
**Why it happens:** It imports `SyncPlan` from `sandbox.rs` which is being deleted.
**How to avoid:** This file WILL fail to compile. Either: (a) temporarily comment it out with a `#[cfg(never)]` annotation, or (b) stub the `SyncPlan` type locally. The CONTEXT says "PONECHAT" -- which means keep the file but it may need temporary stubs.

### Pitfall 6: Semantic Indexer Sandbox Root
**What goes wrong:** `spawn_semantic_indexer` in `state/init.rs` lines 283-323 indexes files from `thread_sandbox_root`. After sandbox removal, it should index from project root.
**Why it happens:** The semantic indexer was designed to work with sandbox copies.
**How to avoid:** Change `thread_sandbox_root` to `thread_root` (project root) in the indexer. This is a behavioral change but necessary.

## Code Examples

### Example 1: Settings Migration (Raw TOML Field Stripping)

```rust
// Source: Project analysis of settings.rs load path
fn migrate_remove_sandbox_fields(path: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(path) else { return };

    if let Ok(mut value) = content.parse::<toml::Value>() {
        if let Some(table) = value.as_table_mut() {
            let removed_sandbox = table.remove("sandbox_mode").is_some();
            let removed_readonly = table.remove("project_read_only").is_some();
            if removed_sandbox || removed_readonly {
                if let Ok(new_content) = toml::to_string_pretty(&value) {
                    let _ = std::fs::write(path, new_content);
                }
            }
        }
    }
}
```

### Example 2: Removing Sandbox Fields from WorkspaceState

Fields to remove from `WorkspaceState` (state/mod.rs):
```rust
// DELETE these fields:
pub pending_settings_save: Option<PendingSettingsSave>,      // line 131
pub pending_sandbox_apply: Option<SandboxApplyRequest>,      // line 132
pub sandbox_persist_failure: Option<SandboxPersistFailure>,  // line 133
pub sandbox_persist_decision: Option<bool>,                  // line 134
pub pending_tab_remap: Option<TabRemapRequest>,              // line 135
pub sandbox_deletion_sync: Option<PathBuf>,                  // line 144
pub promotion_success: Option<PathBuf>,                      // line 147
pub show_sandbox_staged: bool,                               // line 148
pub sync_confirmation: Option<crate::app::sandbox::SyncPlan>,       // line 168
pub sandbox_sync_confirmation: Option<crate::app::sandbox::SyncPlan>, // line 169
pub sandbox_sync_rx: Option<mpsc::Receiver<Result<usize, String>>>, // line 170
pub sandbox_mode_enabled: bool,                              // line 173
pub build_in_sandbox: bool,                                  // line 174
pub file_tree_in_sandbox: bool,                              // line 175
pub sandbox: crate::app::sandbox::Sandbox,                   // line 178
pub sandbox_staged_files: Vec<PathBuf>,                      // line 179
pub sandbox_staged_rx: Option<mpsc::Receiver<Vec<PathBuf>>>, // line 180
pub sandbox_staged_dirty: bool,                              // line 181
pub sandbox_staged_last_dirty: std::time::Instant,           // line 182
pub sandbox_staged_last_refresh: std::time::Instant,         // line 183
```

### Example 3: Removing ToastActionKind Variants

```rust
// DELETE from types.rs -- all 6 variants:
pub(crate) enum ToastActionKind {
    // SandboxApplyNow,      -- DELETE
    // SandboxApplyLater,    -- DELETE
    // SandboxPersistRevert, -- DELETE
    // SandboxPersistKeep,   -- DELETE
    // SandboxRemapTabs,     -- DELETE
    // SandboxSkipRemap,     -- DELETE
}
// After removal, enum is EMPTY. Either remove the enum entirely
// or keep it if Phase 10 toast actions still need it.
// Check if any non-sandbox toast actions exist.
```

**Note:** After removing all 6 variants, `ToastActionKind` becomes empty. The `ToastAction` struct and related code (`Toast::info_with_actions`, `is_expired` lifetime logic) may need adjustment. If no other action kinds exist, the entire toast-action mechanism can be simplified, but check if Phase 10/11 might add new action kinds.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Sandbox mode as core feature | Direct project editing | Phase 9 (v1.1.0) | Simplifies entire codebase |
| `project_read_only` field | Was renamed to `sandbox_mode` | v1.0.x | Both aliases being removed |
| JSON settings | TOML settings | v1.0.x | Migration handles both formats |

## Open Questions

1. **Registry::new(sandbox_root) replacement**
   - What we know: Registry takes a PathBuf for sandbox root
   - What's unclear: Whether Registry actually needs this path after sandbox removal, or if it can be removed from Registry constructor
   - Recommendation: Inspect `src/app/registry/mod.rs` to understand usage. If sandbox-only, replace with project root or remove parameter.

2. **ToastActionKind becomes empty**
   - What we know: All 6 variants are sandbox-related
   - What's unclear: Whether to keep the empty enum for future use or remove it
   - Recommendation: Remove the enum and simplify Toast to not have actions (just message + error flag). Future phases can re-add if needed.

3. **modal_dialogs/sandbox.rs compilation**
   - What we know: This file must stay (Phase 10) but imports `SyncPlan` from deleted module
   - What's unclear: Best approach -- `#[cfg(never)]` vs local stub
   - Recommendation: Use `#[allow(dead_code)]` with a minimal local `SyncPlan` stub, or gate the entire module with `#[cfg(feature = "sandbox")]` (never enabled). Simplest: just comment out the module registration in `modal_dialogs.rs` with a TODO.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `#[test]` |
| Config file | Cargo.toml (standard) |
| Quick run command | `cargo test --lib 2>&1` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | sandbox.rs deleted | smoke | `test ! -f src/app/sandbox.rs && echo OK` | N/A (file check) |
| CORE-02 | No Sandbox/SyncPlan structs | smoke | `! grep -r 'struct Sandbox' src/ && echo OK` | N/A (grep check) |
| SET-01 | sandbox_mode field removed | unit | `cargo test settings::tests -- 2>&1` | Existing tests need update |
| SET-02 | Legacy migration works without sandbox fields | unit | `cargo test settings::tests -- 2>&1` | Existing tests need update |

### Sampling Rate
- **Per task commit:** `cargo check 2>&1`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Update `settings::tests::test_sett02_canonical_toml_persists_sandbox_mode` -- delete (tests removed feature)
- [ ] Update `settings::tests::test_sett05_legacy_project_read_only_maps_to_sandbox_mode` -- convert to migration test
- [ ] Delete `sandbox::tests` module entirely (in sandbox.rs which is being deleted)
- [ ] Delete `state::tests::should_apply_sandbox_request` tests (3 tests)
- [ ] Delete `modal_dialogs/settings.rs` sandbox tests (6 tests, lines 732-826)
- [ ] Add new test: settings migration strips sandbox_mode from TOML
- [ ] Add new test: settings migration strips project_read_only from JSON

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of all 41 files containing "sandbox" references
- `src/app/sandbox.rs` -- full 305-line module examined
- `src/settings.rs` -- Settings struct with sandbox_mode field at line 128
- `src/app/ui/workspace/state/mod.rs` -- WorkspaceState with ~20 sandbox fields
- `src/app/types.rs` -- ToastActionKind with 6 sandbox variants
- `src/app/mod.rs` -- EditorApp with sandbox_root init and mode change detection

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions on boundary (what to keep vs remove)
- REQUIREMENTS.md traceability matrix

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries, pure removal
- Architecture: HIGH -- compiler-driven deletion, well-understood codebase
- Pitfalls: HIGH -- all identified from direct code analysis

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable -- refactoring of known codebase)
