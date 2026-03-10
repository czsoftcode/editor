# Phase 11: File Operations, Watcher & Guard Removal - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit sandbox logiku z file operations (editor save/autosave guardy, readonly mode), terminálů (working dir switching, mode label funkce), plugin registry (sandbox_root, exec_in_sandbox), watcheru (sandbox event filtr) a settings migrace. Editor pracuje přímo s projektovými soubory bez sandbox přesměrování.

</domain>

<decisions>
## Implementation Decisions

### Editor save guardy
- Smazat readonly logiku úplně — bez sandbox módu není důvod mít readonly
- Smazat všechny 3 sandbox path checky v files.rs (`.polycredo/sandbox` checks pro save a autosave)
- Smazat parametr `sandbox_mode_enabled` z celého call chainu (render_editor_area() signatura a všichni calleři)
- local_history.rs: Ponechat .polycredo/ filtr, jen upravit komentář (odstranit zmínku "sandbox")

### Terminál & working dir
- Smazat všechny 3 funkce v terminal/mod.rs: `terminal_working_dir()`, `terminal_mode_label()`, `terminal_mode_label_for_workdir()`
- Smazat oba unit testy (`test_terminal_mode_label_for_workdir_*`) — testují smazanou sandbox logiku
- Vyčistit `sandbox_root` parametr z celého terminal call chainu (bottom/mod.rs, right/mod.rs a calleři)
- Terminál vždy používá project_root přímo

### Plugin registry & AI tools
- Přejmenovat `sandbox_root` → `project_root` v celém plugin systému: PluginManager, PluginHostState, security.rs, fs.rs, sys.rs
- Přejmenovat AI tool `exec_in_sandbox` → `exec` (tools.rs definice + host funkce)
- Aktualizovat všechny system prompty v types.rs — nahradit `exec_in_sandbox` za `exec`
- Registry::new přijímá project_root (už přejmenováno v Phase 9 na caller straně)

### Watcher & settings migrace
- Smazat sandbox filtr ve watcher.rs úplně — žádné speciální zacházení s .polycredo/
- Smazat `migrate_remove_sandbox_fields()` ze settings.rs — migrace není potřeba
- Smazat test `test_settings_migration_strips_sandbox_fields` — testuje smazanou funkci

### Claude's Discretion
- Pořadí odstraňování a řešení kaskádových kompilačních chyb
- Zda existují další skryté sandbox reference v callerech

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `project_root` v Registry::new — Phase 9 už přejmenovala na caller straně, teď dokončit uvnitř
- Settings loading/saving logika — po smazání migrace zůstane čistá

### Established Patterns
- Phase 9/10 vzor: agresivní mazání + fix kompilace — stejný přístup
- Rename pattern: sandbox_root → project_root konzistentně přes celý codebase

### Integration Points
- `src/app/ui/editor/files.rs:62,66,99,170` — sandbox path checks k odstranění
- `src/app/ui/editor/ui.rs:15,88,90` — sandbox_mode_enabled parametr + readonly logika
- `src/app/ui/terminal/mod.rs:30-58,66-80` — 3 sandbox funkce + 2 testy
- `src/watcher.rs:114-119` — sandbox event filtr
- `src/settings.rs:198-247,615-662` — migrate_remove_sandbox_fields + test
- `src/app/registry/plugins/mod.rs:18,26,33,48,220,315,319` — sandbox_root + exec_in_sandbox
- `src/app/registry/plugins/security.rs:39` — sandbox_root field
- `src/app/registry/plugins/host/fs.rs:37,134,185,463` — sandbox_root usage
- `src/app/registry/plugins/host/sys.rs:53,164,184` — host_exec_in_sandbox
- `src/app/ai/tools.rs:106-107` — exec_in_sandbox tool definice
- `src/app/ai/types.rs:30,32,41,50` — system prompty se sandbox zmínkami
- `src/app/local_history.rs:66` — komentář k úpravě
- `src/app/ui/ai_panel.rs:161` — komentář (už přepsán v Phase 9)

</code_context>

<specifics>
## Specific Ideas

- Plugin registry rename sandbox_root → project_root je konzistentní s Phase 9 (caller strana už přejmenována)
- AI tool rename exec_in_sandbox → exec — kratší, přesnější po sandbox removálu
- Watcher filtr smazat úplně (ne zjednodušit) — uživatel chce čistý řez
- Settings migrace smazat — serde default values zvládnou neznámá pole

</specifics>

<deferred>
## Deferred Ideas

- I18n sandbox klíče — Phase 12
- Runtime cleanup .polycredo/sandbox/ adresářů — Phase 12
- Unused imports a dead code warnings — Phase 12

</deferred>

---

*Phase: 11-file-operations-watcher-guard-removal*
*Context gathered: 2026-03-05*
