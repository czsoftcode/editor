# Phase 5: Okamžité aplikování změny režimu sandboxu po přepnutí checkboxu — Research

**Date:** 2026-03-05
**Goal:** Co je potřeba vědět pro dobré plánování okamžité aplikace sandbox módu po Save v Settings.

## Summary
- Změna sandbox módu má být aplikována po `Save` v Settings, bez reopen projektu, a musí se projevit napříč všemi okny stejného projektu.
- Hlavní integrační body jsou: `WorkspaceState` (settings snapshot + dirty), `AppShared.settings` + `settings_version`, `sandbox_mode_enabled`/`build_in_sandbox`/`file_tree_in_sandbox`, terminály a file tree.
- Rizika: více oken současně mění režim, probíhající staged změny při OFF, otevřené dialogy, a I/O persist selže.

## Key Code Areas
- `src/app/ui/workspace/modal_dialogs/settings.rs`
  - Settings modal, Save/Cancel flow, snapshot/revert logika.
- `src/app/ui/workspace/state/mod.rs`
  - `sandbox_mode_enabled`, `build_in_sandbox`, `file_tree_in_sandbox`, dirty tracking, `settings_version`.
- `src/app/ui/workspace/state/init.rs`
  - Inicializace runtime stavu ze settings při startu.
- `src/app/ui/terminal/mod.rs` + `src/app/ui/terminal/bottom/build_bar.rs`
  - Derivace working dir, labely režimu, restart terminálu při změně working dir.
- `src/app/ui/panels.rs`
  - Přepínání `file_tree_in_sandbox`, reload tree.
- `src/app/ui/workspace/mod.rs`
  - Sandbox staged bar, refresh staged.
- `src/app/sandbox.rs`
  - Sync plan, staged files, promote/reconcile.

## Behavioral Requirements (from CONTEXT)
- Apply až po Save; Cancel vrací původní režim a runtime změny.
- Persist na disk před runtime přepnutím; pokud persist selže → toast s volbou revert/keep temporary.
- Potvrzení jen při OFF.
- Restart terminálů, ale běžící procesy doběhnout; label až po restartu.
- File tree přepnout na nový root; otevřené taby přemapovat, neexistující soubory zůstanou s „nenalezen“ stavem.
- Pokud existují staged soubory při OFF → blokace přepnutí a dialog.
- Při ON se zeptat na sync do sandboxu.
- Konflikty mezi okny: upozornění (toast/dialog) pokud souběžná změna.

## Gaps / Open Questions
- Kde přesně se dnes propaguje změna `settings_version` do `WorkspaceState` a UI labelů? (projít signalizaci mezi AppShared a okny)
- Existuje centrální místo, které mění `sandbox_mode_enabled` po Save? (ideálně nové helper API pro aplikaci runtime sandbox módu)
- Jak je řešeno „staged dialog“ a blokace přepnutí OFF? (zda už existuje entrypoint v `sandbox.rs` nebo `workspace/mod.rs`)

## Suggested Implementation Shape
- Přidat explicitní „apply_sandbox_mode_change(...)“ helper, který:
  - Validuje staged state při OFF a případně blokuje.
  - Případně spustí sync při ON (po potvrzení).
  - Upraví runtime state (`sandbox_mode_enabled`, `file_tree_in_sandbox`, `build_in_sandbox`).
  - Restartuje terminály a obnoví file tree.
  - Zajistí přemapování otevřených tabů.
- V Settings Save flow:
  - Persist settings (I/O) → pokud fail, toast s možností revert.
  - Pokud OK, zavolat runtime apply helper.
  - Pokud běží jiný dialog, nabídnout „apply now / defer“.
- Multi-window: změna by měla být distribuována přes existující shared settings + `settings_version`, ale runtime sandbox změny musí běžet v každém okně (lokální UI/terminaly/file tree). To vyžaduje hook při detekci změny settings v každém okně.

## Validation Architecture
- Unit: ověřit, že změna settings (sandbox flag) spouští runtime apply pouze po Save (ne při toggle).
- Integration: simulovat Save v Settings a zkontrolovat:
  - `sandbox_mode_enabled` změnu
  - restart terminálů (new session/working dir)
  - file tree reload + root
- Error handling: simulovat persist fail → toast + možnost revert/keep; runtime apply se nespustí, pokud uživatel zvolí revert.
- Concurrency: když dvě okna přepnou současně, druhé dostane upozornění.

## Risks
- UI blokování při async dialogu/sync – potřeba udržet neblokující flow.
- Příliš agresivní reload file tree může zahodit rozbalení; je potřeba zachovat state podle relativních cest.

## Output
Plán musí zahrnout:
- změny v Settings modal Save/Cancel,
- runtime apply helper + cross-window hook,
- terminály/file tree/tabs/staged handling,
- UX: toasty, confirm OFF, tooltip/inline text.