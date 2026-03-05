# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- 🚧 **v1.1.0 Sandbox Removal** — Phases 9-12 (in progress)

## Phases

<details>
<summary>✅ v1.0.2 Dark/Light Mode (Phases 1-5) — SHIPPED 2026-03-05</summary>

- [x] Phase 1: Základ — 4/4 plans — completed 2026-03-04
- [x] Phase 2: Terminal + Git barvy — 2/2 plans — completed 2026-03-04
- [x] Phase 3: Light varianty + Settings UI — 5/5 plans — completed 2026-03-05
- [x] Phase 4: Infrastructure — 2/2 plans — completed 2026-03-05
- [x] Phase 5: Okamžité aplikování sandbox režimu — 4/4 plans — completed 2026-03-05

Archive: `.planning/milestones/v1.0.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.0.6 Focus Management (Phase 6) — SHIPPED 2026-03-05</summary>

- [x] Phase 6: Docked Terminal Focus Suppression — 1/1 plans — completed 2026-03-05
- ~~Phase 7: Float Terminal Focus Suppression~~ — cancelled (covered by Phase 6)
- ~~Phase 8: Focus Restore & Regression Verification~~ — cancelled (covered by Phase 6)

Archive: `.planning/milestones/v1.0.6-ROADMAP.md`

</details>

### 🚧 v1.1.0 Sandbox Removal (In Progress)

**Milestone Goal:** Kompletne odstranit sandbox rezim z editoru — veskerý kód, UI prvky, logiku a settings.

- [ ] **Phase 9: Core Sandbox Logic & Settings Removal** - Odstraneni sandbox.rs, Sandbox/SyncPlan struktur a sandbox settings fieldu
- [ ] **Phase 10: UI & State Cleanup** - Odstraneni vsech sandbox UI prvku a sandbox-related state fieldu
- [ ] **Phase 11: File Operations, Watcher & Guard Removal** - Odstraneni sandbox logiky z file ops, watcheru a git/build guardu
- [ ] **Phase 12: I18n Cleanup & Integrity Verification** - Odstraneni i18n klicu, verifikace kompilace, testu a funkcnosti

## Phase Details

### Phase 9: Core Sandbox Logic & Settings Removal
**Goal**: Sandbox modul a jeho datove struktury jiz neexistuji v codebase
**Depends on**: Nothing (first phase of v1.1.0)
**Requirements**: CORE-01, CORE-02, SET-01, SET-02
**Success Criteria** (what must be TRUE):
  1. Soubor `src/app/sandbox.rs` neexistuje a `mod sandbox` deklarace je odstranena z `app/mod.rs`
  2. Struktury `Sandbox`, `SyncPlan` a vsechny sandbox metody neexistuji v zadnem souboru
  3. `Settings.sandbox_mode` field neexistuje a settings serializace/deserializace funguje bez nej
  4. Legacy migrace `project_read_only` je odstranena a settings loading funguje korektne
  5. Projekt se kompiluje (warnings povoleny v teto fazi)
**Plans:** 2 plans

Plans:
- [ ] 09-01-PLAN.md — Smazat sandbox.rs, odstranit sandbox_mode ze Settings, pridat migraci
- [ ] 09-02-PLAN.md — Odstranit vsechny sandbox struktury/fieldy/metody, opravit kompilaci

### Phase 10: UI & State Cleanup
**Goal**: Uzivatel nevidi zadne sandbox prvky v UI a interni state neobsahuje sandbox fieldy
**Depends on**: Phase 9
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, UI-06, STATE-01, STATE-02, STATE-03, STATE-04
**Success Criteria** (what must be TRUE):
  1. Settings dialog neobsahuje sandbox toggle, tooltip ani inline poznamku
  2. File tree nezobrazuje "Sandbox" toggle button ani "Soubory (Sandbox)" label
  3. Build bar nezobrazuje "Sandbox ON/OFF" indikator
  4. Zadne sandbox-related toast akce (Apply now/Defer, Remap/Skip, Revert/Keep) se nemohou zobrazit
  5. Projekt se kompiluje (warnings povoleny v teto fazi)
**Plans**: TBD

### Phase 11: File Operations, Watcher & Guard Removal
**Goal**: Editor pracuje primo s projektovymi soubory bez sandbox presmerovani a bez sandbox guardu
**Depends on**: Phase 10
**Requirements**: FILE-01, FILE-02, FILE-03, WATCH-01, WATCH-02, GIT-01, GIT-02
**Success Criteria** (what must be TRUE):
  1. Otevirani a ukladani souboru probiha primo bez sandbox tab remappingu
  2. File tree vzdy zobrazuje projektový koren bez sandbox/project root switchingu
  3. Terminaly vzdy pouzivaji projektovy adresar bez sandbox working directory switchingu
  4. Git operace a build/deb akce jsou vzdy povoleny bez sandbox guardu
  5. Projekt se kompiluje (warnings povoleny v teto fazi)
**Plans**: TBD

### Phase 12: I18n Cleanup & Integrity Verification
**Goal**: Editor je ciste zkompilovan bez warnigu, vsechny testy prochasi a editor je plne funkcni
**Depends on**: Phase 11
**Requirements**: I18N-01, I18N-02, INT-01, INT-02, INT-03
**Success Criteria** (what must be TRUE):
  1. Zadne sandbox i18n klice neexistuji v zadnem z 5 jazyku (cs, en, de, ru, sk)
  2. Test `all_lang_keys_match_english` prochazi
  3. `cargo build` projde bez warnigu (zadne unused imports, dead code)
  4. Vsechny existujici testy prochazi (`cargo test`)
  5. Editor se spusti a je plne funkcni — otevirani souboru, editace, terminaly, git, build
**Plans**: TBD

## Progress

**Execution Order:** 9 → 10 → 11 → 12

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Základ | v1.0.2 | 4/4 | Complete | 2026-03-04 |
| 2. Terminal + Git barvy | v1.0.2 | 2/2 | Complete | 2026-03-04 |
| 3. Light varianty + Settings UI | v1.0.2 | 5/5 | Complete | 2026-03-05 |
| 4. Infrastructure | v1.0.2 | 2/2 | Complete | 2026-03-05 |
| 5. Okamžité aplikování sandbox režimu | v1.0.2 | 4/4 | Complete | 2026-03-05 |
| 6. Docked Terminal Focus Suppression | v1.0.6 | 1/1 | Complete | 2026-03-05 |
| ~~7. Float Terminal Focus Suppression~~ | v1.0.6 | — | Cancelled | — |
| ~~8. Focus Restore & Regression~~ | v1.0.6 | — | Cancelled | — |
| 9. Core Sandbox Logic & Settings Removal | v1.1.0 | 0/2 | Not started | - |
| 10. UI & State Cleanup | v1.1.0 | 0/? | Not started | - |
| 11. File Operations, Watcher & Guard Removal | v1.1.0 | 0/? | Not started | - |
| 12. I18n Cleanup & Integrity Verification | v1.1.0 | 0/? | Not started | - |
