# Requirements: PolyCredo Editor v1.1.0 — Sandbox Removal

**Defined:** 2026-03-05
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1.1.0 Requirements

Requirements for sandbox removal milestone. Each maps to roadmap phases.

### Core Code Removal

- [x] **CORE-01**: Soubor `src/app/sandbox.rs` je kompletně odstraněn
- [x] **CORE-02**: Struct `Sandbox`, `SyncPlan` a všechny sandbox metody jsou odstraněny

### State Cleanup

- [x] **STATE-01**: Všech ~15 sandbox-related fieldů odstraněno z `WorkspaceState`
- [x] **STATE-02**: `SandboxApplyRequest`, `SandboxPersistFailure`, `PendingSettingsSave.sandbox_off_confirmed` odstraněny
- [x] **STATE-03**: `ToastActionKind` varianty pro sandbox (6 variant) odstraněny z `types.rs`
- [x] **STATE-04**: `AppShared.sandbox_off_toast_shown` odstraněno

### UI Removal

- [x] **UI-01**: Settings toggle pro sandbox mode odstraněn ze Settings dialogu
- [x] **UI-02**: Sandbox sync confirmation modal (`modal_dialogs/sandbox.rs`) odstraněn
- [x] **UI-03**: Sandbox OFF confirmation dialog odstraněn ze settings
- [x] **UI-04**: File tree "Sandbox" toggle button a "Soubory (Sandbox)" label odstraněny
- [x] **UI-05**: Build bar "Sandbox ON/OFF" indikátor odstraněn
- [x] **UI-06**: Toast akce (Apply now/Defer, Remap/Skip, Revert/Keep) odstraněny

### Settings Cleanup

- [x] **SET-01**: `Settings.sandbox_mode` field odstraněn
- [x] **SET-02**: Legacy migrace `project_read_only` odstraněna

### I18n Cleanup

- [ ] **I18N-01**: Všech ~40+ sandbox i18n klíčů odstraněno ze všech 5 jazyků (cs, en, de, ru, sk)
- [ ] **I18N-02**: Test `all_lang_keys_match_english` stále prochází po odstranění

### File Operations

- [x] **FILE-01**: Tab remapping logika pro sandbox odstraněna z `editor/files.rs`
- [x] **FILE-02**: File tree sandbox/project root switching logika odstraněna
- [x] **FILE-03**: Terminal working directory sandbox switching odstraněno

### Watcher & Background

- [x] **WATCH-01**: Sandbox-specific logika ve `watcher.rs` odstraněna (staged detection, auto-sync)
- [x] **WATCH-02**: Background tasks pro sandbox sync a staging detection odstraněny

### Git & Build Restrictions

- [x] **GIT-01**: Git disabled-in-sandbox guards odstraněny
- [x] **GIT-02**: Build/deb disabled-in-sandbox guards odstraněny

### Integrity

- [ ] **INT-01**: Projekt se kompiluje bez warningů (unused imports, dead code)
- [ ] **INT-02**: Existující testy procházejí
- [ ] **INT-03**: Editor je plně funkční bez sandbox režimu

## Future Requirements

### Performance Optimization

- **PERF-01**: Identifikovat hlavní příčiny vysokého CPU v idle (profilování / měření)
- **PERF-02**: Omezit zbytečné překreslování egui renderovací smyčky (conditional repaint)
- **PERF-03**: Snížit frekvenci nebo optimalizovat git polling (aktuálně každých 5s)
- **PERF-04**: Optimalizovat autosave timer (aktuálně 500ms interval)
- **PERF-05**: Prověřit FileWatcher/ProjectWatcher overhead

### Tech Debt

- **DEBT-01**: Opravit kontrast warning textu v light mode (Settings modal)
- **DEBT-02**: Nyquist VALIDATION.md: 6 fází ve stavu draft

## Out of Scope

| Feature | Reason |
|---------|--------|
| Odstranění .polycredo/ adresáře | Uživatel chce ponechat pro budoucí použití |
| Přidání nových features | Čistý refactoring milestone |
| Performance optimalizace | Plánováno na budoucí milestone |
| Sandbox větev v gitu | Ponechána jako archiv (`sandbox` branch) |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 9 | Complete |
| CORE-02 | Phase 9 | Complete |
| STATE-01 | Phase 10 | Complete |
| STATE-02 | Phase 10 | Complete |
| STATE-03 | Phase 10 | Complete |
| STATE-04 | Phase 10 | Complete |
| UI-01 | Phase 10 | Complete |
| UI-02 | Phase 10 | Complete |
| UI-03 | Phase 10 | Complete |
| UI-04 | Phase 10 | Complete |
| UI-05 | Phase 10 | Complete |
| UI-06 | Phase 10 | Complete |
| SET-01 | Phase 9 | Complete |
| SET-02 | Phase 9 | Complete |
| I18N-01 | Phase 12 | Pending |
| I18N-02 | Phase 12 | Pending |
| FILE-01 | Phase 11 | Complete |
| FILE-02 | Phase 11 | Complete |
| FILE-03 | Phase 11 | Complete |
| WATCH-01 | Phase 11 | Complete |
| WATCH-02 | Phase 11 | Complete |
| GIT-01 | Phase 11 | Complete |
| GIT-02 | Phase 11 | Complete |
| INT-01 | Phase 12 | Pending |
| INT-02 | Phase 12 | Pending |
| INT-03 | Phase 12 | Pending |

**Coverage:**
- v1.1.0 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

---
*Requirements defined: 2026-03-05*
*Last updated: 2026-03-05 after roadmap creation*
