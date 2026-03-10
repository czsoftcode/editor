---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-03-10T19:42:22.637Z"
last_activity: "2026-03-10 - Completed plan 26-03: Save feedback regression pack"
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 18
  completed_plans: 17
  percent: 100
---

---

## gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
last_updated: "2026-03-10T15:47:14.744Z"
last_activity: 2026-03-10 - Completed quick task 8: Root close button closes active project
progress:
  [██████████] 100%
  completed_phases: 1
  total_plans: 11
  completed_plans: 5
  percent: 100

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-07)

**Core value:** Editor nesmi zahrivat notebook v klidovem stavu — idle CPU zatez musi byt minimalni.
**Current focus:** Phase 26 - Save UX Polish + Regression Hardening

## Current Position

Phase: 26 of 26 (Save UX Polish + Regression Hardening)
Plan: 03 of 4 completed (next: 04)
Status: In Progress
Last activity: 2026-03-10 - Completed plan 26-03: Save feedback regression pack

Progress: [██████████] 98%

## Performance Metrics

**Velocity:**

- v1.0.2: 17 plans completed (5 phases)
- v1.0.6: 1 plan completed (1 phase, covered 3 planned phases)
- v1.1.0: 8 plans completed (4 phases), 15 feat/fix commits, -2,878 net lines
- v1.2.0: 19 plans completed (6 phases), 42 feat/fix commits, +3,212 net lines
- Total: 45 plans across 16 phases (4 milestones)

## Accumulated Context

### Decisions

Key decisions logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [v1.2.0]: ureq + std::thread misto reqwest/tokio — odpovida threading modelu codebase
- [v1.2.0]: Security-first tool execution — PathSandbox + approval workflow
- [v1.2.1-dev]: Zero new dependencies — cely GSD port pouziva existujici crates
- [v1.2.1-dev]: Slash commands — static slice registry, SYSTEM_MSG_MARKER prefix, Levenshtein fuzzy match
- [Phase 19]: Async slash commands via mpsc + generation counter for stale result detection
- [Phase 19]: Conservative code-fence check: skip ALL path regex for blocks containing any code fence
- [Phase 20]: Custom YAML-like frontmatter parser with raw_lines round-trip preservation pattern
- [Phase 20]: GSD dispatch via match-based routing, two-level autocomplete for /gsd subcommands
- [Phase 20]: ISO timestamp generation without chrono crate using Howard Hinnant date algorithm
- [Phase 24]: SaveMode persistence uses snake_case serde values automatic/manual for stable TOML representation.
- [Phase 24]: Missing save_mode in legacy settings defaults to Manual via serde default for backward compatibility.
- [Phase 24-save-mode-foundation]: Ctrl+S prefers settings draft save when Settings modal is open; editor save remains default outside modal.
- [Phase 24-save-mode-foundation]: Save mode change toast is emitted only after successful settings save and only when mode actually changed.
- [Phase 24-save-mode-foundation]: Save mode labels/toasts/status are fully i18n-driven for language parity.
- [Phase 24-save-mode-foundation]: Menu Save and Ctrl+S now call one workspace-level manual save handler to keep behavior identical.
- [Phase 24-save-mode-foundation]: Save error dedupe key is the final localized error message with a 1.5s suppression window.
- [Phase 24-save-mode-foundation]: Validation map is locked to final task IDs from plans 24-01 through 24-03.
- [Phase 24-save-mode-foundation]: Nyquist compliance flip now requires green automated gates plus PASS for all M-* manual scenarios.
- [Phase 25-unsaved-close-guard]: Ctrl+W handling moved to egui consume_shortcut to prevent TextEdit fallback.
- [Phase 25-unsaved-close-guard]: Editor lock derives from dialog_open_base OR pending_close_flow active state.
- [Phase 25-unsaved-close-guard]: DirtyCloseQueueMode::SingleTab(target) vrací max. jednu položku pouze pro dirty target tab.
- [Phase 25-unsaved-close-guard]: TabBarAction::Close(idx) řeší target přes snapshot path a při race (idx mimo rozsah) je bezpečný no-op.
- [Phase 25-unsaved-close-guard]: Esc v unsaved guard dialogu se explicitne consume a mapuje na Cancel vetev.
- [Phase 25-unsaved-close-guard]: Guard queue handoff otevira tab bez focus requestu, aby modal drzel fokus do ukonceni flow.
- [Phase 26-save-ux-polish-regression-hardening]: Status bar save mode key cte pouze runtime mode; settings draft se nepropisuje mimo apply.
- [Phase 26-save-ux-polish-regression-hardening]: Tab save mode marker je pouze doplnkovy na aktivnim tabu (·M/·A) a dirty symbol zustava primarni.
- [Phase 26-save-ux-polish-regression-hardening]: MODE-04 regression coverage je oddelena v src/app/ui/workspace/tests/save_mode.rs.
- [Phase 26]: Dirty stav ve status baru je explicitne primarni signal (dirty-first) pri soubehu s mode informaci.
- [Phase 26]: Save UX kontrast/priorita jsou kryte targeted regression testy save_ux_contrast_regression.
- [Phase 26-save-ux-polish-regression-hardening]: Ctrl+S routing is mediated through manual_save_request_for_shortcut to keep branch mapping deterministic.
- [Phase 26-save-ux-polish-regression-hardening]: Guard save-failure handling is centralized to keep inline error, toast feedback, and close eligibility testable.
- [Phase 26-save-ux-polish-regression-hardening]: Save error dedupe uses an explicit within-window classifier while preserving existing 1.5s semantics.

### Known Tech Debt

- Nyquist VALIDATION.md: faze ve stavu draft
- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu nefunguje (egui_commonmark code blocky cernobile)

### Pending Todos

- Opravit kontrast warning textu v light mode (`modal_dialogs/settings.rs`)

### Blockers/Concerns

- ~~Research flag: Phase 20 (frontmatter parser)~~ — RESOLVED: two-pass parser implemented with FmNode raw_lines pattern
- Research flag: Phase 20 (state operations) — writeStateMd round-trip complexity potrebuje caching strategii
- Research flag: Phase 22 (AI init) — init command context aggregation potrebuje vyhodnoceni relevance pro Ollama

### Quick Tasks Completed


| #                                 | Description                                                                                                                                                                                                                      | Date       | Commit   | Directory                                                                                         |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | -------- | ------------------------------------------------------------------------------------------------- |
| 3                                 | Remove systemd-run wrapper — launch program directly                                                                                                                                                                             | 2026-03-06 | 6f50509  | [3-remove-systemd-run-wrapper-launch-progra](./quick/3-remove-systemd-run-wrapper-launch-progra/) |
| 4                                 | Move compile bar next to build bar, remove compile_bar.rs                                                                                                                                                                        | 2026-03-06 | 9c4b211  | [4-move-compile-bar-next-to-build-bar-remov](./quick/4-move-compile-bar-next-to-build-bar-remov/) |
| 5                                 | Revert validate_ollama_url port restriction + Bearer auth                                                                                                                                                                        | 2026-03-06 | 3cc63a0  | [5-revert-validate-ollama-url-port-restrict](./quick/5-revert-validate-ollama-url-port-restrict/) |
| 6                                 | Rename src/app/ai to src/app/cli + update 48 path references                                                                                                                                                                     | 2026-03-06 | eecb769  | [6-rename-ai-to-cli](./quick/6-rename-ai-to-cli/)                                                 |
| 7                                 | md nahled je nyni pod sebou. udelej mi vedle tlacitka Otevřít v externím prohlížeči jeste toggle tlacitko na zmeny nahledu - Pod sebou | Vedle sebe | Jenom kód | Jenom náhled - kter bude prepinat cyklicky mezi mody zobrazeni | 2026-03-09 | 84c7067  | [7-md-nahled-je-nyni-pod-sebou-udelej-mi-ve](./quick/7-md-nahled-je-nyni-pod-sebou-udelej-mi-ve/) |
| 8                                 | Root close button in main window now closes active project via RootProjectClose flow                                                                                                                                             | 2026-03-10 | N/A      | [8-kdyz-mam-otevreno-hlavni-okno-s-jednim-p](./quick/8-kdyz-mam-otevreno-hlavni-okno-s-jednim-p/) |
| Phase 19 P03                      | 1min                                                                                                                                                                                                                             | 1 tasks    | 1 files  |                                                                                                   |
| Phase 20 P01                      | 7min                                                                                                                                                                                                                             | 3 tasks    | 1 files  |                                                                                                   |
| Phase 20 P02                      | 8min                                                                                                                                                                                                                             | 2 tasks    | 7 files  |                                                                                                   |
| Phase 20 P03                      | 3min                                                                                                                                                                                                                             | 2 tasks    | 2 files  |                                                                                                   |
| Phase 24-save-mode-foundation P01 | 1min                                                                                                                                                                                                                             | 2 tasks    | 1 files  |                                                                                                   |
| Phase 24-save-mode-foundation P02 | 6min                                                                                                                                                                                                                             | 3 tasks    | 8 files  |                                                                                                   |
| Phase 24-save-mode-foundation P03 | 4min                                                                                                                                                                                                                             | 3 tasks    | 10 files |                                                                                                   |
| Phase 24-save-mode-foundation P04 | 2min                                                                                                                                                                                                                             | 2 tasks    | 2 files  |                                                                                                   |
| Phase 25-unsaved-close-guard P08 | 3min | 2 tasks | 3 files |
| Phase 25-unsaved-close-guard P10 | 3min | 3 tasks | 3 files |
| Phase 25-unsaved-close-guard P09 | 3min | 2 tasks | 4 files |
| Phase 26-save-ux-polish-regression-hardening P01 | 3min | 3 tasks | 4 files |
| Phase 26-save-ux-polish-regression-hardening P02 | 6min | 2 tasks | 2 files |
| Phase 26-save-ux-polish-regression-hardening P03 | 6 min | 3 tasks | 5 files |
