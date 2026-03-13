# Phase 10: UI & State Cleanup - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit všechny sandbox UI prvky (settings dialog, file tree, build bar, modal dialogy) a vyčistit zbývající sandbox-related state fieldy. Uživatel nesmí vidět žádné sandbox prvky v UI. I18n klíče se odstraní až v Phase 12. File operations a watcher cleanup patří do Phase 11.

</domain>

<decisions>
## Implementation Decisions

### Settings dialog
- Smazat celý sandbox blok v kategorii "general" (řádky 272–294): separator, disabled checkbox stub, tooltip, hint text a terminal note
- Kategorii "general" ponechat — po odstranění zůstane výběr jazyka + default project path
- I18n klíče (settings-safe-mode, settings-safe-mode-tooltip, settings-safe-mode-hint, settings-safe-mode-terminal-note) ponechat pro Phase 12

### File tree
- Odebrat parametr `is_sandbox` z `render()` a `render_node()` v file_tree
- Lazy line count a zvýraznění velkých souborů (500+ řádků) ponechat jako globální feature — aktivovat vždy, ne jen v sandbox módu
- Odebrat podmínku `if is_sandbox` a nechat logiku běžet pro všechny soubory
- Žádný dynamický label "Soubory (Sandbox)" neexistuje — nic k odstranění

### Modal dialogy
- Smazat celý soubor `modal_dialogs/sandbox.rs` (80 řádků, 100% sandbox-only)
- Odebrat TODO komentář `// Phase 10 — remove modal_dialogs/sandbox.rs entirely` z `modal_dialogs.rs`
- Odebrat `mod sandbox` deklaraci a `pub use` z `modal_dialogs.rs`

### Build bar
- Smazat celý label "Terminal" i s hover textem `hover-build-sandbox` — label je po odstranění sandbox nepotřebný

### State fieldy
- WorkspaceState a types.rs jsou už čisté (Phase 9 odstranila sandbox fieldy, ToastActionKind varianty i SandboxApplyRequest)
- V gitignore filtru (state/init.rs řádek 218) odebrat jen `"sandbox"` podmínku, ponechat `.polycredo` filtr

### Claude's Discretion
- Pořadí odstraňování souborů a řešení kompilačních chyb
- Jak přepojit volání `is_sandbox` parametru v callerech file_tree

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `StandardModal` widget — používán v sandbox.rs, po smazání souboru žádná změna
- File tree lazy line count logika — přechází z sandbox-only na globální feature

### Established Patterns
- Phase 9 vzor: agresivní mazání + fix kompilace — stejný přístup pro Phase 10
- `modal_dialogs.rs` jako hub modul — po smazání sandbox.rs odebrat deklaraci

### Integration Points
- `src/app/ui/workspace/modal_dialogs/settings.rs:272-294` — sandbox blok k odstranění
- `src/app/ui/workspace/modal_dialogs/sandbox.rs` — celý soubor k odstranění
- `src/app/ui/workspace/modal_dialogs.rs:12` — TODO komentář + mod deklarace
- `src/app/ui/file_tree/render.rs:31,115,132` — is_sandbox parametr a podmínky
- `src/app/ui/file_tree/mod.rs:96,125` — is_sandbox v render() signatuře a volání
- `src/app/ui/terminal/bottom/build_bar.rs:14-16` — Terminal label + sandbox hover
- `src/app/ui/workspace/state/init.rs:218` — gitignore sandbox filtr

</code_context>

<specifics>
## Specific Ideas

- File tree line count feature se stává globální — žádná ztráta funkcionality, naopak vylepšení
- Build bar label "Terminal" je po sandbox nepotřebný — čistý řez bez náhrady
- I18n klíče zůstanou osiřelé do Phase 12 — vědomé rozhodnutí dle roadmapy

</specifics>

<deferred>
## Deferred Ideas

- I18n klíče sandbox — Phase 12
- Runtime cleanup .polycredo/sandbox/ adresářů — Phase 12
- File operations sandbox remapping — Phase 11
- Watcher sandbox logika — Phase 11

</deferred>

---

*Phase: 10-ui-state-cleanup*
*Context gathered: 2026-03-05*
