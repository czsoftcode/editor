---
id: S08
parent: M001
milestone: M001
provides:
  - Okamžité apply sandbox režimu po Save
  - Persist → runtime apply flow s revert/keep volbou
  - Multi-window propagace přes settings_version
  - Graceful restart terminálů při změně režimu
  - Přemapování tabů mezi rooty s varováním u chybějících souborů
  - Blokace OFF při staged souborech s dialogem
  - Sync dialog při ON s automatickým přenosem
  - Automatický cleanup pending_tab_remap
  - Kompletní i18n pokrytí (cs, en, sk, de, ru)
key_files:
  - src/app/ui/workspace/modal_dialogs/settings.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/editor/files.rs
  - src/app/ui/terminal/instance/mod.rs
  - src/app/ui/panels.rs
key_decisions:
  - "Sandbox apply přes pending_sandbox_apply — spouští se až po persistu"
  - "Persist failure řeší toast s volbou revert/keep"
  - "Label terminálu se mění okamžitě při nové instanci"
  - "Blokace OFF bezprostředně po záznamu sandbox_mode_change"
  - "Sync dialog přes sandbox_sync_confirmation v process_pending_sandbox_apply"
  - "Sync v background threadu přes spawn_task"
  - "pending_tab_remap cleanup ihned po retain()"
patterns_established:
  - "Pending apply fronta přes settings_version pro multi-window sync"
  - "spawn_task pro asynchronní sandbox operace"
  - "Toast s akcí pro uživatelské rozhodnutí (remap/skip)"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
  - tasks/T03-SUMMARY.md
  - tasks/T04-SUMMARY.md
duration: 66min
verification_result: passed
completed_at: 2026-03-05
---

# S08: Okamžité Aplikování Změny Režimu Sandboxu

**Kompletní runtime sandbox workflow: Save→persist→apply s multi-window propagací, graceful restart terminálů, tab remap, staged blokace OFF a sync při ON.**

## What Happened

Čtyři tasky vybudovaly plný runtime sandbox lifecycle: T01 zavedl Save/Cancel flow s potvrzením OFF, multi-window propagaci přes settings_version a persist error handling s revert/keep volbou. T02 přidal graceful restart terminálů s ponecháním běžících procesů, remap tabů s varováním u chybějících souborů a toast prompt. T03 implementoval blokaci OFF při staged souborech (draft revert + staged dialog) a sync dialog při ON s background přenosem. T04 uzavřel gap — automatický cleanup zombie pending_tab_remap po expiraci toastu a dokumentace label-timing záměru.

## Verification

- `cargo check` čistý
- `cargo test` — 71 testů zelených
- i18n test `all_lang_keys_match_english` — 27 chybějících klíčů doplněno a ověřeno

## Deviations

- Chybějící i18n klíče pro sk, de, ru — auto-fixed v T03 (249c2eb)
- `cargo fmt` formátování vyžadováno po T02

## Known Limitations

- Tab focus traversal po Tab completion v autocomplete (kosmetické, odloženo)

## Follow-ups

- CLI UAT (multi-window + persist failure) — doporučen manuální test

## Files Created/Modified

- `src/app/ui/workspace/modal_dialogs/settings.rs` — potvrzení OFF, persist flow, staged blokace
- `src/app/ui/workspace/state/mod.rs` — pending struktury, runtime apply helper, dokumentace
- `src/app/ui/workspace/mod.rs` — process_pending, sync confirmation, remap prompt
- `src/app/ui/terminal/instance/mod.rs` — graceful exit, background tick
- `src/app/ui/editor/files.rs` — remap tabů, testy
- `src/app/ui/panels.rs` — toast akce, cleanup
- `src/app/ui/workspace/modal_dialogs/sandbox.rs` — sync dialog modal
- `locales/*/ui.ftl` — 27 nových klíčů pro 5 jazyků

## Forward Intelligence

### What the next slice should know
- Sandbox runtime je kompletní — další slicy mohou na něj navazovat

### What's fragile
- pending_tab_remap cleanup závisí na toast retain() pořadí — pokud se změní toast lifecycle, může se rozbít

### Authoritative diagnostics
- i18n test all_lang_keys_match_english — spolehlivý indikátor chybějících klíčů

### What assumptions changed
- Label se mění okamžitě při nové instanci, ne po exitu starého PTY — záměrné
