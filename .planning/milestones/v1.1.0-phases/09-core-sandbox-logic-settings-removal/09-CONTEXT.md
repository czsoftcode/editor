# Phase 9: Core Sandbox Logic & Settings Removal - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit sandbox modul (`src/app/sandbox.rs`), struktury `Sandbox`/`SyncPlan`, `sandbox_mode` field ze Settings a legacy `project_read_only` migraci. Projekt se musí zkompilovat (warnings povoleny). Modal dialogy sandbox.rs a runtime cleanup adresářů patří do pozdějších fází.

</domain>

<decisions>
## Implementation Decisions

### Zpětná kompatibilita settings
- Explicitní migrace při loadu — detekovat sandbox_mode/project_read_only v raw TOML/JSON, odstranit a ihned přepsat soubor
- Migrace pro oba formáty: settings.toml i settings.json (legacy)
- Oba aliasy (sandbox_mode i project_read_only) odstranit najednou v jednom kroku
- Jednorázová migrace — po přepsání souboru pole zmizí natrvalo

### Hranice odstranění (agresivní mazání)
- Smazat `src/app/sandbox.rs` celý + `mod sandbox` deklaraci z `app/mod.rs`
- Smazat všechny struktury závislé na Sandbox/SyncPlan:
  - `PendingSettingsSave`, `SandboxApplyRequest`, `SandboxPersistFailure`, `TabRemapRequest` (state/mod.rs)
  - Všech ~18 sandbox-related polí z `WorkspaceState`
  - `sandbox_off_toast_shown` z `AppShared`
- Smazat 6 sandbox `ToastActionKind` variant + všechny match arms
- Smazat sandbox helper funkce v settings.rs (`sandbox_mode_change()`, `requires_sandbox_off_confirm()`, `should_block_sandbox_off_due_to_staged()`, `should_block_sandbox_apply()`, `show_sandbox_off_confirm()`, `SandboxModeChange` enum)
- Smazat metody `apply_sandbox_mode_change()` a `should_apply_sandbox_request()` ze state/mod.rs
- Smazat `spawn_get_staged_files()` a `process_pending_sandbox_apply()` z workspace/mod.rs
- Smazat sandbox inicializaci z `app/mod.rs` (sandbox_root) a `state/init.rs`
- **PONECHAT pro Phase 10:** `modal_dialogs/sandbox.rs` (celý soubor — UI dialog)
- **PONECHAT pro Phase 12:** runtime cleanup .polycredo/sandbox/ adresářů

### Existující .polycredo/sandbox/ adresáře
- Aktivně smazat .polycredo/sandbox/ (ne celý .polycredo/) při startu editoru
- Implementovat až v Phase 12 (verifikace), ne v Phase 9
- Phase 9 se zabývá pouze kódem, ne runtime chováním na disku

### Claude's Discretion
- Pořadí odstraňování souborů (co první, co pak)
- Jak řešit kompilační chyby vzniklé kaskádovým mazáním
- Zda některé sandbox-related testy migrovat nebo jen smazat

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `settings_version` AtomicU64 zůstává — slouží i pro theme propagaci, není sandbox-only
- Settings loading/saving logika v `src/settings.rs` — migrace se napojí na existující load path

### Established Patterns
- Serde `#[serde(alias = "...")]` pro zpětnou kompatibilitu — existující vzor pro migraci
- Settings snapshot + Save/Cancel flow v modal_dialogs/settings.rs — po odstranění sandbox checkboxu se zjednoduší
- `WorkspaceState` inicializace v `state/init.rs` — sandbox init kód se odstraní, zbytek zůstane

### Integration Points
- `src/app/mod.rs` řádky 122-125: sandbox_root inicializace → smazat
- `src/app/mod.rs` řádky 566-590, 757-780: sandbox mode change detection → smazat
- `src/app/ui/workspace/state/init.rs` řádky 22-34: sandbox instance + staged files init → smazat
- `src/settings.rs` řádek 128-129: sandbox_mode field + alias → smazat, přidat migrační logiku
- `src/app/types.rs` řádky 182, 203-208: sandbox-related pole a enum varianty → smazat
- `src/app/ui/workspace/state/mod.rs`: 4 sandbox struktury + ~18 polí + 2 metody → smazat

</code_context>

<specifics>
## Specific Ideas

- Migrace settings má být při loadu (ne lazy) — přepsat soubor ihned po detekci sandbox polí
- Modal dialog `modal_dialogs/sandbox.rs` explicitně ponechat pro Phase 10 — nepřekračovat hranici UI cleanup
- Runtime cleanup sandbox adresářů patří do Phase 12, ne sem

</specifics>

<deferred>
## Deferred Ideas

- Runtime cleanup .polycredo/sandbox/ adresářů — Phase 12
- Sandbox modal dialog odstranění — Phase 10
- UI prvky (sandbox toggle, tooltip, build bar indikátor) — Phase 10
- File operations sandbox remapping — Phase 11

</deferred>

---

*Phase: 09-core-sandbox-logic-settings-removal*
*Context gathered: 2026-03-05*
