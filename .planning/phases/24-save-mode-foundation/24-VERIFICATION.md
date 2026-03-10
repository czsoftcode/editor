---
phase: 24
slug: save-mode-foundation
status: passed
verified_at: 2026-03-09
human_approval: 2026-03-09
verifier: codex
---

# Phase 24 Verification

## Goal Check
Goal: "Uživatel má předvídatelné ukládání s Ctrl+S default a přepínatelným auto/manual režimem."

Verdikt: **částečně potvrzeno automaticky, finální uzavření vyžaduje human UAT**.

- `Ctrl+S` je napojené na jednotný handler a mimo Settings modal ukládá aktivní tab.
- Save mode (`Automatic`/`Manual`) je v Settings přepínatelný, persistovaný a runtime používaný.
- Chování odpovídá plánu 24-01..24-03 v implementačních must-have bodech.
- Chybí provedené manuální scénáře z `24-VALIDATION.md` (`M-*`), proto nelze dát `passed`.

## Score
- **Celkem: 88/100**
- Implementace proti requirements: 6/6 pokryto v kódu
- Automatizované ověření v tomto běhu: `cargo check` + cílené testy = PASS
- Blokery pro `passed`: neprovedené manual UAT scénáře + `./check.sh` fail na pre-existing fmt drift mimo scope fáze

## Requirement Coverage (SAVE-01..03, MODE-01..03)

### SAVE-01 — Ctrl+S uloží aktivní tab
**Stav:** PASS (code evidence), HUMAN UAT pending

Důkazy:
- `Ctrl+S` volá `handle_manual_save_action`: `src/app/ui/workspace/mod.rs:193-195`
- Save z menu volá stejný handler: `src/app/ui/workspace/menubar/mod.rs:85-87`
- Samotný save aktivního tabu: `src/app/ui/workspace/mod.rs:71-77` + `src/app/ui/editor/files.rs:72-129`

### SAVE-02 — Modified -> Saved bez změny fokusu
**Stav:** PASS (code evidence), HUMAN UAT pending

Důkazy:
- Po úspěšném save se nastaví `modified = false` a `save_status = Saved`: `src/app/ui/editor/files.rs:111-114`
- Shortcut flow nevyžaduje focus změnu, reaguje přímo na `Ctrl+S`: `src/app/ui/workspace/mod.rs:193-195`

### SAVE-03 — Save chyby jsou viditelné v UI a nejsou tiché
**Stav:** PASS (code evidence + test), HUMAN UAT pending

Důkazy:
- Chyba save vrací lokalizovaný error string a stav tabu zůstává `Modified`: `src/app/ui/editor/files.rs:116-128`
- Ruční save zobrazuje error toast (s dedupe politikou): `src/app/ui/workspace/mod.rs:73-76`
- Autosave path zobrazuje error toast (stejná dedupe politika): `src/app/ui/background.rs:603-606`
- Dedupe helper test: `src/app/types.rs:255-272` (`save_error_dedupe_suppresses_repeated_error_within_window`)

### MODE-01 — Přepínání Automatic/Manual v Settings
**Stav:** PASS

Důkazy:
- UI radio toggle pro oba režimy: `src/app/ui/workspace/modal_dialogs/settings.rs:402-415`
- Persist model (`SaveMode`, `Settings.save_mode`): `src/settings.rs:70-99`

### MODE-02 — Persist režimu přes restart
**Stav:** PASS (automated evidence)

Důkazy:
- `save_mode` je serializovaný/deserializovaný přes TOML: `src/settings.rs:268-283`, `src/settings.rs:651-668`
- Backward kompatibilita bez pole `save_mode` -> `Manual`: `src/settings.rs:671-683`
- Default pro nové settings je `Manual`: `src/settings.rs:179-186`, `src/settings.rs:645-648`

### MODE-03 — Runtime apply po Settings Save bez restartu
**Stav:** PASS (code evidence), HUMAN UAT pending

Důkazy:
- `Ctrl+S` při otevřeném Settings ukládá settings draft: `src/app/ui/workspace/mod.rs:52-70`
- Uložení settings aktualizuje `shared.settings` + `settings_version`: `src/app/ui/workspace/modal_dialogs/settings.rs:190-193`
- Autosave gate respektuje aktuální `save_mode` při každém background ticku: `src/app/ui/background.rs:600-604`
- Status bar zobrazuje aktivní režim runtime: `src/app/ui/workspace/mod.rs:307-314`

## Must-Haves vs Reality (PLAN)

### 24-01 (MODE-01, MODE-02)
- ✅ SaveMode enum + `Settings.save_mode` + `#[serde(default)]` existuje.
- ✅ Default `Manual` potvrzen implementací i testem.
- ✅ Backward compatibility test bez `save_mode` existuje.

### 24-02 (MODE-01, MODE-03)
- ✅ Settings modal má toggle `Automatic/Manual` v draft flow.
- ✅ Změna režimu se aplikuje při Save (nikoliv pouze klikáním v draftu).
- ✅ `Ctrl+S` v otevřeném Settings modalu ukládá settings draft.
- ✅ Aktivní režim je vidět ve status baru + i18n klíče existují ve všech jazycích.

### 24-03 (SAVE-01, SAVE-02, SAVE-03, MODE-03)
- ✅ Ctrl+S i menu Save jdou přes stejný handler.
- ✅ Save success přepíná `Modified -> Saved` okamžitě.
- ✅ Save error je surfacovaný toastem, tab zůstává `Modified`.
- ✅ Dedupe opakovaných chyb je implementovaná a testovaná.
- ✅ Autosave běží jen v `Automatic` režimu.

### 24-04 (validate closeout)
- ✅ `24-VALIDATION.md` obsahuje mapu tasků i coverage pro všech 6 requirement ID.
- ⚠️ Nyquist sign-off není uzavřen: manual scénáře jsou stále pending.

## Co je splněno
- Datový model save mode a persistence vrstva.
- UI toggle + runtime indikace režimu.
- Sjednocený save flow (Ctrl+S + menu Save).
- Autosave gate podle režimu.
- Error feedback + dedupe.
- Lokalizační pokrytí pro nové klíče ve všech 5 jazycích.

## Co chybí / rizika
- Nejsou doložené výsledky manual scénářů (`M-CTRL-S-EDITOR`, `M-SAVE-FAILURE`, `M-CTRL-S-MODAL`, `M-RESTART-PERSISTENCE`, `M-RUNTIME-APPLY`).
- `./check.sh` v aktuálním repu selhává na rozsáhlém pre-existing `cargo fmt --check` driftu mimo scope fáze 24; formální gate tedy není plně green.

## Human Verification Steps
Pro přepnutí na `status: passed` doporučuji provést a zapsat PASS/FAIL:

1. `M-CTRL-S-EDITOR`
2. `M-SAVE-FAILURE`
3. `M-CTRL-S-MODAL`
4. `M-RESTART-PERSISTENCE`
5. `M-RUNTIME-APPLY`

Dále:
1. Spustit `./check.sh` v čistém/stabilizovaném formátovacím stavu repa.
2. Doložit výstup příkazů do validační evidence (nebo explicitně potvrdit waiver na fmt drift mimo scope).

## Evidence From This Verification Run
- `cargo check` -> PASS (warning-only)
- `cargo test save_mode_ -- --nocapture` -> PASS
- `cargo test should_run_autosave_only_in_automatic_mode -- --nocapture` -> PASS
- `cargo test save_error_dedupe_suppresses_repeated_error_within_window -- --nocapture` -> PASS
- `./check.sh` -> FAIL (pre-existing formatting drift in unrelated files)

## Final Decision
**status: `human_needed`**

Odůvodnění: implementace a cílené testy potvrzují požadavky SAVE-01..03 a MODE-01..03 na úrovni kódu, ale bez provedených manual UAT scénářů a bez zeleného `./check.sh` nelze fázi 24 objektivně uzavřít jako `passed`.
