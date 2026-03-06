# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- ✅ **v1.1.0 Sandbox Removal** — Phases 9-12 (shipped 2026-03-06)
- ✅ **v1.2.0 AI Chat Rewrite** — Phases 13-18 (shipped 2026-03-06)

## Phases

<details>
<summary>✅ v1.0.2 Dark/Light Mode (Phases 1-5) — SHIPPED 2026-03-05</summary>

- [x] Phase 1: Základ (4/4 plans) — completed 2026-03-04
- [x] Phase 2: Terminal + Git barvy (2/2 plans) — completed 2026-03-04
- [x] Phase 3: Light varianty + Settings UI (5/5 plans) — completed 2026-03-05
- [x] Phase 4: Infrastructure (2/2 plans) — completed 2026-03-05
- [x] Phase 5: Okamžité aplikování sandbox režimu (4/4 plans) — completed 2026-03-05

Archive: `.planning/milestones/v1.0.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.0.6 Focus Management (Phase 6) — SHIPPED 2026-03-05</summary>

- [x] Phase 6: Docked Terminal Focus Suppression (1/1 plans) — completed 2026-03-05
- ~~Phase 7: Float Terminal Focus Suppression~~ — cancelled (covered by Phase 6)
- ~~Phase 8: Focus Restore & Regression Verification~~ — cancelled (covered by Phase 6)

Archive: `.planning/milestones/v1.0.6-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1.0 Sandbox Removal (Phases 9-12) — SHIPPED 2026-03-06</summary>

- [x] Phase 9: Core Sandbox Logic & Settings Removal (3/3 plans) — completed 2026-03-05
- [x] Phase 10: UI & State Cleanup (1/1 plans) — completed 2026-03-05
- [x] Phase 11: File Operations, Watcher & Guard Removal (2/2 plans) — completed 2026-03-05
- [x] Phase 12: I18n Cleanup & Integrity Verification (2/2 plans) — completed 2026-03-05

Archive: `.planning/milestones/v1.1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2.0 AI Chat Rewrite (Phases 13-18) — SHIPPED 2026-03-06</summary>

- [x] Phase 13: Provider Foundation (3/3 plans) — completed 2026-03-06
- [x] Phase 14: State Refactor (2/2 plans) — completed 2026-03-06
- [x] Phase 15: Streaming Chat UI (5/5 plans) — completed 2026-03-06
- [x] Phase 16: Tool Execution (4/4 plans) — completed 2026-03-06
- [x] Phase 17: i18n & WASM Cleanup (3/3 plans) — completed 2026-03-06
- [x] Phase 18: Phase 16 Verification & i18n Fixes (2/2 plans) — completed 2026-03-06

Archive: `.planning/milestones/v1.2.0-ROADMAP.md`

</details>

## Progress

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
| 9. Core Sandbox Logic & Settings Removal | v1.1.0 | 3/3 | Complete | 2026-03-05 |
| 10. UI & State Cleanup | v1.1.0 | 1/1 | Complete | 2026-03-05 |
| 11. File Operations, Watcher & Guard Removal | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 12. I18n Cleanup & Integrity Verification | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 13. Provider Foundation | v1.2.0 | 3/3 | Complete | 2026-03-06 |
| 14. State Refactor | v1.2.0 | 2/2 | Complete | 2026-03-06 |
| 15. Streaming Chat UI | v1.2.0 | 5/5 | Complete | 2026-03-06 |
| 16. Tool Execution | v1.2.0 | 4/4 | Complete | 2026-03-06 |
| 17. i18n & WASM Cleanup | v1.2.0 | 3/3 | Complete | 2026-03-06 |
| 18. Phase 16 Verification & i18n Fixes | v1.2.0 | 2/2 | Complete | 2026-03-06 |

## Known Issues / TODO

- **Syntax highlighting v AI chatu**: `egui_commonmark` s `better_syntax_highlighting` feature a `syntax_theme_dark("base16-ocean.dark")` nefunguje — code blocky (```rust) se zobrazuji cernobile bez barev. Nutno vyresit — mozna vlastni rendering code bloku pres syntect (uz pouzivame v editoru).
