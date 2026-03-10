# Phase 17: i18n & WASM Cleanup - Research

**Researched:** 2026-03-06
**Domain:** i18n (Fluent), WASM plugin removal, settings cleanup
**Confidence:** HIGH

## Summary

Phase 17 has two orthogonal workstreams: (1) i18n completion -- replacing hardcoded strings with Fluent keys, renaming existing key prefixes, creating a new `cli.ftl` file, and removing obsolete keys; (2) WASM plugin system removal -- deleting ~3000 LOC of plugin infrastructure, removing the `extism` dependency, cleaning up registry, menu, settings, and types.

The codebase is well-structured for both tasks. The i18n system uses `include_str!` to embed FTL files at compile time (in `src/i18n.rs`), and a test `all_lang_keys_match_english` enforces parity across all 5 languages. Adding a new `cli.ftl` file requires updating the `RESOURCES_*` arrays in `i18n.rs`. The WASM removal is primarily a deletion task, but requires careful cleanup of references in `app/mod.rs`, `registry/mod.rs`, types, menubar, and settings.

**Primary recommendation:** Execute i18n changes first (new keys, renames, new cli.ftl), then WASM removal second. This order avoids broken references during the transition, since WASM-related i18n keys will be deleted as part of WASM removal.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Plugin Manager UI -- kompletni odstraneni (menu, command palette, permission bar, error dialog)
- AI bar klice se PREJMENUJOU: ai-plugin-bar-* -> cli-bar-*
- Settings prejmenovani: "Plugin Blacklist" -> "CLI Blacklist", AI sekce -> "PolyCredo CLI"
- PRIDAT Ollama parametry do Settings: temperature, num_ctx, top-p, top-k + dalsi
- Lokalizace: prefix cli-tool-* pro tool klice, cli-chat-* pro chat klice
- NOVY soubor cli.ftl ve vsech 5 jazycich
- Stary ai.ftl se SMAZE
- Hardcoded ceske retezce v approval.rs, render.rs, inspector.rs se nahradi i18n klici
- WASM odstraneni: src/app/registry/plugins/ (~1931 LOC), src/plugins/ (~822 LOC), docs/samples/hello-plugin/, extism dependency

### Claude's Discretion
- Ktere dalsi Ollama API parametry pridat do Settings (repeat_penalty, seed, mirostat, ...)
- Presne pojmenovani novych i18n klicu (pri dodrzeni prefixu cli-tool-* a cli-chat-*)
- Poradi odstranovani WASM kodu (dependency graph)
- Jak vycistit registry/mod.rs po odstraneni WASM pluginu

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLEN-02 | Odstraneni WASM plugin systemu -- extism, PluginManager, ~2000 LOC | Full inventory of files, dependencies, and references documented below |
| CLEN-03 | i18n aktualizace -- nove klice pro novy chat, odstraneni starych WASM klicu | Complete audit of hardcoded strings, key renames, and new keys needed |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| fluent-bundle | (existing) | i18n runtime | Already in use, Fluent is the standard for Rust i18n |
| unic-langid | (existing) | Language ID parsing | Dependency of fluent-bundle |

### No New Dependencies
This phase removes `extism = "1.5"` and adds zero new dependencies. All changes use existing Rust std library and egui APIs.

## Architecture Patterns

### i18n File Structure (Current -> Target)
```
locales/{cs,en,de,ru,sk}/
  menu.ftl       -- menu strings (remove plugin entries)
  ui.ftl         -- UI strings (remove ~30 plugin keys, rename ai-* -> cli-*)
  dialogs.ftl    -- unchanged
  errors.ftl     -- unchanged
  ai.ftl         -- DELETE (5 gemini-specific keys)
  cli.ftl        -- NEW (all CLI/AI chat keys consolidated here)
```

### i18n Registration Pattern
Adding `cli.ftl` requires updating `src/i18n.rs`:
```rust
// For EACH language (cs, en, sk, de, ru), add to RESOURCES_XX array:
const RESOURCES_EN: &[&str] = &[
    include_str!("../locales/en/menu.ftl"),
    include_str!("../locales/en/ui.ftl"),
    include_str!("../locales/en/dialogs.ftl"),
    include_str!("../locales/en/errors.ftl"),
    // REMOVE: include_str!("../locales/en/ai.ftl"),
    include_str!("../locales/en/cli.ftl"),  // NEW
];
```

### WASM Deletion Dependency Graph
```
1. Remove menu entry (menubar/file.rs) -- eliminates UI trigger
2. Remove plugins modal (modal_dialogs/plugins.rs, modal_dialogs.rs)
3. Remove AppAction::Plugin* variants (types.rs) + handlers (app/mod.rs)
4. Remove CommandAction::Plugin variant + plugin commands (registry/mod.rs)
5. Remove plugins field from Registry struct (registry/mod.rs)
6. Delete src/app/registry/plugins/ directory (1931 LOC)
7. Delete src/plugins/ directory (822 LOC)
8. Delete docs/samples/hello-plugin/
9. Remove extism from Cargo.toml
10. Remove plugin-related fields from settings.rs + WorkspaceState
```

### Registry Cleanup Pattern
After WASM removal, `registry/mod.rs` needs:
- Remove `pub mod plugins;` and `use ... PluginManager`
- Remove `plugins: Arc<PluginManager>` from `Registry` struct
- Remove `CommandAction::Plugin` variant (mark dead or remove entirely)
- Remove `PanelRegistry` and `PanelArea` if only used by plugins (check usage)
- Simplify `Registry::new()` -- no `project_root` parameter needed if only plugins used it
- Remove `ui.show_plugins` command from `init_defaults()`

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| i18n key parity | Manual verification | `all_lang_keys_match_english` test | Existing test catches missing/extra keys automatically |
| FTL syntax | Custom parser | Fluent FTL format | Standard format, validated by fluent-bundle at compile time |

## Common Pitfalls

### Pitfall 1: Forgetting a Language
**What goes wrong:** Adding i18n keys to `en` but missing one of cs/de/ru/sk
**Why it happens:** 5 languages is easy to miss one
**How to avoid:** Always edit all 5 FTL files in the same commit. Run `cargo test` -- `all_lang_keys_match_english` will catch mismatches.
**Warning signs:** Test failure listing missing keys

### Pitfall 2: Orphaned References After WASM Deletion
**What goes wrong:** Compilation fails because deleted types/modules are still referenced
**Why it happens:** Plugin system has tentacles in 14+ files
**How to avoid:** Follow the dependency graph order above. Compile after each major deletion step.
**Warning signs:** `cargo check` errors

### Pitfall 3: Breaking the i18n Key Rename
**What goes wrong:** Renaming i18n key in FTL but not updating the Rust `i18n.get("old-key")` call
**Why it happens:** String-based key system has no compile-time check
**How to avoid:** Use grep to find all usages of each renamed key before changing FTL files
**Warning signs:** UI shows raw key names like "ai-chat-title" instead of translated text

### Pitfall 4: Settings Serialization Break
**What goes wrong:** Removing `plugins: HashMap<String, PluginSettings>` from Settings breaks existing user config files
**Why it happens:** TOML deserialization fails if struct doesn't match saved fields
**How to avoid:** Keep `#[serde(default)]` on all fields, or use `#[serde(flatten)]` for backward compat. Alternatively, since `plugins` has `#[serde(default)]`, removing it should be fine -- serde ignores unknown fields by default in TOML.
**Warning signs:** App crashes on startup for existing users

### Pitfall 5: Hardcoded Strings in Tool Approval UI
**What goes wrong:** approval.rs has ~20 hardcoded Czech strings that are invisible in English locale
**Why it happens:** Written during rapid Phase 16 development
**How to avoid:** Audit identified all instances (see Code Examples below)

## Code Examples

### Hardcoded Strings Inventory (approval.rs)

All strings that need i18n keys (file: `src/app/ui/terminal/ai_chat/approval.rs`):

```
Line 22:  "Agent '{}' vyzaduje schvaleni akce"   -> cli-tool-approval-heading
Line 46:  "1 - Provest"                           -> cli-tool-approve
Line 57:  "2 - Schvalovat vzdy"                   -> cli-tool-approve-always
Line 68:  "3/Esc - Zamitnout"                     -> cli-tool-deny
Line 113: "Agent '{}' se pta:"                    -> cli-tool-ask-heading
Line 125: "Rychle moznosti:"                      -> cli-tool-quick-options
Line 137: "**Odpoved:** {}"                       -> (markdown, keep as-is or localize)
Line 143: "Nebo napiste vlastni:"                 -> cli-tool-custom-input
Line 151: "Vase odpoved..."                       -> cli-tool-input-placeholder
Line 161: "Odeslat"                               -> cli-tool-send
Line 173: "Zrusit"                                -> cli-tool-cancel
Line 220: "AI nastroj '{}' vyzaduje schvaleni"    -> cli-tool-tool-approval-heading
Line 233: "Sitovy prikaz -- data mohou..."        -> cli-tool-network-warning
Line 241: "Novy soubor (nizsi riziko)"            -> cli-tool-new-file-hint
Line 265: "1 - Schvalit"                          -> cli-tool-approve (reuse)
Line 276: "2 - Vzdy schvalit"                     -> cli-tool-approve-always (reuse)
Line 286: "3/Esc - Zamitnout"                     -> cli-tool-deny (reuse)
Line 323: "AI se pta:"                            -> cli-tool-ask-heading (reuse)
Line 335: "Rychle moznosti:"                      -> cli-tool-quick-options (reuse)
Line 351: "Nebo napiste vlastni:"                 -> cli-tool-custom-input (reuse)
Line 359: "Vase odpoved..."                       -> cli-tool-input-placeholder (reuse)
Line 366: "Odeslat"                               -> cli-tool-send (reuse)
Line 380: "Zrusit"                                -> cli-tool-cancel (reuse)
```

### Hardcoded Strings (render.rs + inspector.rs + conversation.rs)

```
terminal/ai_chat/render.rs:380:  "Stop"     -> cli-chat-stop
inspector.rs:9:                  "Clear"    -> cli-chat-clear
inspector.rs:13:                 "Copy"     -> cli-chat-copy
inspector.rs:7:                  "AI Inspector" -> cli-chat-inspector-title
widgets/ai/chat/conversation.rs:85:  "Copy"  -> btn-copy (reuse existing)
widgets/ai/chat/conversation.rs:159: "Copy"  -> btn-copy (reuse existing)
```

### Key Renames: ai-chat-* -> cli-chat-*

Keys to rename in all 5 FTL files (move from ui.ftl to new cli.ftl):
```
ai-chat-title            -> cli-chat-title
ai-chat-label-response   -> cli-chat-label-response
ai-chat-loading          -> cli-chat-loading
ai-chat-label-prompt     -> cli-chat-label-prompt
ai-chat-placeholder-prompt -> cli-chat-placeholder-prompt
ai-chat-btn-send         -> cli-chat-btn-send
ai-chat-btn-new          -> cli-chat-btn-new
ai-chat-settings-title   -> cli-chat-settings-title
ai-chat-label-language   -> cli-chat-label-language
ai-chat-btn-reset        -> cli-chat-btn-reset
ai-chat-label-system-prompt -> cli-chat-label-system-prompt
ai-chat-default-prompt   -> cli-chat-default-prompt
```

### Key Renames: ai-plugin-bar-* -> cli-bar-*

Keys to rename (move from ui.ftl to cli.ftl):
```
ai-plugin-bar-label        -> cli-bar-label
ai-plugin-bar-settings     -> cli-bar-settings
ai-plugin-bar-start-hover  -> cli-bar-start-hover
ai-plugin-bar-settings-hover -> cli-bar-settings-hover
```

### Keys to DELETE from ui.ftl (~30+ keys)

```
plugins-title, plugins-list-label, plugins-no-selection, plugins-enabled-label,
plugins-config-label, plugins-unknown-agent, plugins-category-ai, plugins-category-general,
plugins-item-settings, plugins-item-welcome, plugins-welcome-title, plugins-welcome-text,
plugins-welcome-hint, plugins-security-info, plugins-settings-saved,
plugins-placeholder-api-key, plugins-placeholder-model,
command-name-show-plugins, command-name-plugin-hello,
command-name-plugin-gemini, command-name-plugin-ollama, command-name-plugin-ai-chat,
plugin-auth-bar-msg, plugin-auth-bar-allow, plugin-auth-bar-deny,
plugin-error-title, plugin-error-heading
```

### Keys to DELETE from menu.ftl

```
menu-file-plugins, menu-file-plugins-manager
```

### Settings: Ollama Parameters to Add

Current `ProviderConfig` has only `temperature` and `num_ctx`. Recommended additions for Settings UI:

| Parameter | Type | Default | Range | Ollama API field |
|-----------|------|---------|-------|-----------------|
| temperature | f64 | 0.7 | 0.0-2.0 | options.temperature |
| num_ctx | u64 | 4096 | 512-131072 | options.num_ctx |
| top_p | f64 | 0.9 | 0.0-1.0 | options.top_p |
| top_k | u64 | 40 | 1-100 | options.top_k |
| repeat_penalty | f64 | 1.1 | 0.0-2.0 | options.repeat_penalty |
| seed | i64 | 0 (random) | any | options.seed |

These need to be added to: `ProviderConfig` struct, `Settings` struct, Settings UI (modal_dialogs/settings.rs), and all `ProviderConfig` construction sites (background.rs, logic.rs).

### Settings Fields to Remove

```rust
// settings.rs -- remove these:
pub plugins: HashMap<String, PluginSettings>,  // plugin configs
pub blacklist: Vec<String>,                    // plugin file blacklist
pub ai_settings_migrated: bool,               // migration flag (one-time, done)

// Also remove:
struct PluginSettings                          // entire struct
fn migrate_plugin_ai_settings()                // migration function
```

Rename:
```rust
// settings.rs:
"Plugin Blacklist (blocked files)" -> "CLI Blacklist"  (settings-blacklist key)
```

### Files to DELETE

```
src/app/registry/plugins/mod.rs      (496 LOC)
src/app/registry/plugins/security.rs (73 LOC)
src/app/registry/plugins/types.rs    (111 LOC)
src/app/registry/plugins/host/mod.rs (81 LOC)
src/app/registry/plugins/host/fs.rs  (638 LOC)
src/app/registry/plugins/host/sys.rs (449 LOC)
src/app/registry/plugins/host/search.rs (83 LOC)
src/plugins/ollama/src/lib.rs        (489 LOC)
src/plugins/gemini/src/lib.rs        (327 LOC)
src/plugins/hello/src/lib.rs         (6 LOC)
docs/samples/hello-plugin/           (directory)
src/app/ui/workspace/modal_dialogs/plugins.rs (457 LOC)
locales/{cs,en,de,ru,sk}/ai.ftl      (5 keys each)
```

Total: ~3210 LOC deleted + ~25 keys * 5 langs deleted

### Files to MODIFY

```
src/i18n.rs                          -- swap ai.ftl -> cli.ftl in RESOURCES arrays
src/app/registry/mod.rs              -- remove plugins module, Plugin variant, cleanup
src/app/mod.rs                       -- remove plugin init, Plugin* action handlers
src/app/types.rs                     -- remove Plugin* AppAction variants + PluginApprovalResponse
src/app/ui/workspace/menubar/file.rs -- remove Plugins menu
src/app/ui/workspace/menubar/mod.rs  -- remove plugin action handling
src/app/ui/panels.rs                 -- rename ai-plugin-bar-* to cli-bar-*
src/app/ui/workspace/modal_dialogs.rs -- remove plugins modal trigger
src/app/ui/workspace/mod.rs          -- remove show_plugins/plugins_draft
src/app/ui/workspace/state/mod.rs    -- remove plugin-related fields
src/app/ui/workspace/state/init.rs   -- update key references
src/app/ui/terminal/ai_chat/approval.rs -- replace ALL hardcoded strings with i18n
src/app/ui/terminal/ai_chat/render.rs   -- replace "Stop" with i18n
src/app/ui/terminal/ai_chat/inspector.rs -- replace "Clear", "Copy" with i18n
src/app/ui/widgets/ai/chat/conversation.rs -- replace "Copy" with btn-copy
src/app/ui/widgets/ai/chat/settings.rs    -- update key references
src/settings.rs                      -- remove PluginSettings, add Ollama params
src/app/ai/provider.rs               -- add top_p, top_k, repeat_penalty, seed to ProviderConfig
src/app/ai/ollama.rs                 -- pass new params in options JSON
src/app/ui/background.rs             -- update ProviderConfig construction
src/app/ui/terminal/ai_chat/logic.rs -- update ProviderConfig construction
Cargo.toml                           -- remove extism = "1.5"
locales/{cs,en,de,ru,sk}/ui.ftl      -- remove plugin keys, move ai-* keys out
locales/{cs,en,de,ru,sk}/menu.ftl    -- remove plugin menu keys
locales/{cs,en,de,ru,sk}/cli.ftl     -- NEW file with all cli-* keys
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WASM plugins via extism | Native Rust AI (Ollama direct) | Phase 13-16 | ~3000 LOC to remove |
| Hardcoded Czech strings | Fluent i18n system | Phase 1 (partial) | Approval UI still has Czech hardcoded |
| ai.ftl for AI keys | cli.ftl consolidation | Phase 17 (planned) | Cleaner i18n organization |

## Open Questions

1. **PanelRegistry and PanelArea -- keep or remove?**
   - What we know: Currently defined in registry/mod.rs, all methods are `#[allow(dead_code)]`
   - What's unclear: Whether any future phase plans to use them
   - Recommendation: Keep them -- they're small, generic infrastructure. Remove only `#[allow(dead_code)]` on Plugin variant of CommandAction.

2. **pending_plugin_approval vs pending_tool_approval in WorkspaceState**
   - What we know: Both exist in WorkspaceState. `pending_plugin_approval` is the old WASM path, `pending_tool_approval` is the new native path (Phase 16).
   - Recommendation: Remove `pending_plugin_approval`, `pending_ask_user`, and the old `render_approval_ui`/`render_ask_user_ui` functions. Keep only the native `render_tool_approval_ui`/`render_tool_ask_ui`.

3. **Settings backward compatibility for removed `plugins` field**
   - What we know: `plugins` field uses `#[serde(default)]`. Serde's TOML deserializer ignores unknown fields by default.
   - Recommendation: Safe to remove -- old config files with `[plugins.xxx]` sections will be silently ignored by serde.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust) |
| Config file | Cargo.toml |
| Quick run command | `cargo test all_lang_keys_match_english` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLEN-02 | WASM removed, editor compiles | smoke | `cargo check` | N/A (compile check) |
| CLEN-03 | All i18n keys present in 5 languages | unit | `cargo test all_lang_keys_match_english` | Exists in src/i18n.rs |
| CLEN-03 | No hardcoded Czech strings in approval.rs | manual | grep for Czech characters in approval.rs | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check && cargo test all_lang_keys_match_english`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full `cargo test` green + `cargo check` clean

### Wave 0 Gaps
- [ ] Grep check for remaining hardcoded strings after i18n migration (no test file needed -- manual verification)
- None critical -- existing `all_lang_keys_match_english` test covers i18n parity automatically

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection of all relevant files
- `src/i18n.rs` -- i18n system architecture and test structure
- `locales/en/ui.ftl` -- complete key inventory
- `src/app/registry/mod.rs` -- registry architecture
- `src/settings.rs` -- settings structure
- `src/app/ui/terminal/ai_chat/approval.rs` -- hardcoded string inventory

### Secondary (MEDIUM confidence)
- Ollama API documentation for parameter names and defaults (verified against existing code in `src/app/ai/ollama.rs`)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new deps, all existing patterns
- Architecture: HIGH -- complete file inventory from direct codebase inspection
- Pitfalls: HIGH -- based on actual code structure analysis
- i18n key inventory: HIGH -- exhaustive grep of all hardcoded strings
- WASM deletion scope: HIGH -- counted lines, mapped all 14 referencing files

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (stable -- internal codebase, no external API changes)
