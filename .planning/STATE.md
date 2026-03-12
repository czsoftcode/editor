---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: milestone
status: planning
last_updated: "2026-03-12T10:02:17.517Z"
last_activity: 2026-03-12 - Completed 36-03 plan execution
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 6
  completed_plans: 6
  percent: 100
---

## gsd_state_version: 1.0

## Current Milestone: v1.3.1 Safe Trash Delete

**Goal:** Zmenit mazani souboru na bezpecne presouvani do `.polycredo/trash` s moznosti obnovy.

**Target features:**
- Zalozeni a sprava `.polycredo/trash` adresare
- Presmerovani delete operaci na move-to-trash tok
- Definice obnovy a cleanup pravidel bez blokace UI

**Status:** Ready to plan

---

## Project Reference

See: .planning/PROJECT.md

**Core value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.
**Current focus:** Phase 36 execution (Safe Move-to-Trash Engine)

## Current Position

Phase: 36 (completed)
Plan: 03 (completed)
Status: Phase 36 completed, ready for phase 37
Last activity: 2026-03-12 - Completed 36-03 plan execution

Progress: [██████████] 100%

---

## Accumulated Context

### Decisions

- [v1.3.0]: `src/app/cli/*` je mimo dalsi smer produktu, zustane pouze AI terminal workflow.
- [v1.3.0]: Cleanup bude rozdelen do fazi 30-32 kvuli kontrolovane migraci.
- [Phase 30]: Public root export switched from legacy CLI namespace to app::odstraneny runtime modul with internal cli visibility retained for staged migration.
- [Phase 30]: Foundation migration scope locked to settings/types/workspace-state; runtime AI terminal migration deferred to next plans.
- [Phase 30-cli-namespace-removal-foundation]: AI terminal head/right bar byl uzamcen do assistant-only UX bez provider model/status prvku.
- [Phase 30-cli-namespace-removal-foundation]: CLI-02 audit scope byl docisten i o odstraneny chat modul/slash.rs, protoze je soucasti overovaneho subsetu.
- [Phase 30]: AiManager a runtime AI vrstva byly presunuty do app::odstraneny runtime modul a legacy CLI namespace namespace byl odstraneny bez fallback aliasu.
- [Phase 30]: CLI-01 je dokumentovan explicitnim 30-02-AUDIT.md artefaktem s build a grep PASS dukazy.
- [Phase 30-cli-namespace-removal-foundation]: Public API app root bylo zúženo na odstraneny runtime modul; ostatní moduly jsou interní (pub(crate)).
- [Phase 30-cli-namespace-removal-foundation]: Task 2 byl potvrzen samostatným audit commitem bez změny kódu kvůli atomickému task trace.
- [Phase 31-ai-terminal-runtime-migration]: Retry flow je explicitni UI akce vazana na posledni validni prompt, aktivovana jen po runtime chybe.
- [Phase 31-ai-terminal-runtime-migration]: Slash async stale-guard je sjednoceny jednim helperem pro /build i /git.
- [Phase 31-ai-terminal-runtime-migration]: Prompt se normalizuje na vstupu (trim/slash) a prázdný model je guardovaný před startem streamu.
- [Phase 31-ai-terminal-runtime-migration]: AI bar je assistant-only bez provider-picker UI a bez vazby na provider model list.
- [Phase 31-ai-terminal-runtime-migration]: execute_approved musi znovu validovat sandbox/blacklist/rate-limit guardy i po schvaleni.
- [Phase 31-ai-terminal-runtime-migration]: Audit detail payloady jsou sanitovany na jeden radek kvuli citelne forenzni stope.
- [Phase 31-ai-terminal-runtime-migration]: TERM/SAFE evidence is unified in 31-VERIFICATION.md for single-artifact audit traceability.
- [Phase 31-ai-terminal-runtime-migration]: Gap closure boundary je assistant-only AI terminal bez provider-picker couplingu; SAFE approval/security kontrakt zustava beze zmeny.
- [Phase 31-ai-terminal-runtime-migration]: Final gate requires both cargo check and check.sh PASS before enabling nyquist_compliant true.
- [Phase 31-ai-terminal-runtime-migration]: Model/provider picker controls were removed from AI bar to keep assistant-only boundary explicit.
- [Phase 31-ai-terminal-runtime-migration]: Provider sync/poll and connection access were centralized behind AiState helpers to avoid direct UI/runtime coupling.
- [Phase 31-ai-terminal-runtime-migration]: SAFE approval/security contract remained unchanged and was re-verified by approval/security test suites.
- [Phase 31-ai-terminal-runtime-migration]: ARCH-01 reference byla odstranena remove variantou z phase 31 artefaktu bez rozsirovani REQUIREMENTS.
- [Phase 31-ai-terminal-runtime-migration]: Konfliktni Task 3 verifikace byla uzavrena konzistentnim remove-only checkem bez ARCH-01 v 31-VERIFICATION.
- [Phase 32-cleanup-tests-and-stabilization]: Phase32 regression tests use explicit active runtime file lists to guard against CLI namespace relapse.
- [Phase 32-cleanup-tests-and-stabilization]: Denied approval errors now emit toast feedback to keep failure visibility and retry context explicit.
- [Phase 32-cleanup-tests-and-stabilization]: STAB-01/STAB-02 evidence was centralized in 32-VERIFICATION.md with command-level PASS records.
- [Phase 32-cleanup-tests-and-stabilization]: STAB-01 and STAB-02 sign-off was centralized into one evidence-first artifact with command-level PASS mapping.
- [Phase 32-cleanup-tests-and-stabilization]: Planning traceability updates stayed limited to active v1.3 artifacts and avoided historical file rewrites.
- [Phase 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu]: AiExpertiseRole/AiReasoningDepth moved to app::ai_prefs to keep settings compatibility after odstraneny runtime modul hard-removal.
- [Phase 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu]: Historical tests were switched to assert phase33 removal state (deleted odstraneny runtime modul/odstraneny chat modul files) so quality gate matches launcher-only architecture.
- [Phase 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu]: Legacy i18n rodiny cli-chat/cli-tool byly odstraneny bez fallback textu; ponechany jen aktivne pouzivane launcher/settings klice.
- [Phase 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu]: No-fallback grep guard zustava v plan scope, false-positive toast.*ai byl resen neutralni lokalni vazbou bez zmeny chovani.
- [Phase 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu]: Ai_bar launcher dispatch byl zprehlednen explicitnim helperem send_selected_agent_command.
- [Phase 33]: Wave 3 quality gate audit byl omezen na aktivni scope; globalni historical cleanup zustava explicitne ve wave 4 planu.
- [Phase 33]: Build gate byl proveden s RUSTC_WRAPPER= kvuli lokalnimu sccache permission blockeru bez zasahu do kodu.
- [Phase 33]: Batch A/B byly potvrzeny jako ciste; traceability je zachovana audit-only task commity bez obsahovych diffu.
- [Phase 33]: Historicke command evidence v 33-VERIFICATION.md bylo neutralizovano placeholder patterny bez navratu zakazanych literalu.
- [Phase 33]: Finalni gate byl overen s RUSTC_WRAPPER workaroundem kvuli lokalnimu sccache permission blockeru.
- [Phase 34]: Phase 33 verification byla rebaselinovana na PASS chain bez dalsiho rozsirovani scope mimo verification artefakty.
- [Phase 34]: Cross-phase drift v phase 31 byl resen prepisem evidence na command-level a aktivni launcher-only cesty.
- [Phase 34-milestone-gap-closure-and-traceability-rebaseline]: Task 3 byl realizovan jako TDD gate: failing traceability test nasledovany explicitnim final_gate markerem.
- [Phase 34-milestone-gap-closure-and-traceability-rebaseline]: Milestone audit byl prepnut na passed az po fast/full gate chain s 15/15 coverage.
- [Phase 35]: Phase 35 zustava strictne delete-path only; restore-zaklad symboly jsou explicitne zakazane.
- [Phase 35]: Delete-path chyby se mapuji na kontextovy format trash move failed: {err} pred toast surfacingem.
- [Phase 36-safe-move-to-trash-engine]: Engine-level guard blocks both .polycredo/trash root and nested paths, independent of UI hiding.
- [Phase 36-safe-move-to-trash-engine]: Move failures use a centralized fail-closed formatter with actionable next-step guidance.
- [Phase 36]: Delete engine errors are normalized into localized toast reason categories before surfacing.
- [Phase 36]: TryRecvError::Disconnected is fail-visible and closes delete_rx immediately.
- [Phase 36]: Scope guard evidence uses grep-safe symbol construction to keep deterministic plan checks.
- [Phase 36-safe-move-to-trash-engine]: Task 1 a Task 3 byly uzavreny audit-only commity, protoze focused verify probehl bez nutnosti patchu.
- [Phase 36-safe-move-to-trash-engine]: Verification evidence je centralizovana v 36-VERIFICATION.md s explicitnim requirement-to-hook mapovanim.

### Roadmap Evolution

- Phase 33 added: odstranit veskerou zminku a funkce polycredo cli ze systemu

### Known Tech Debt

- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu nefunguje (egui_commonmark code blocky cernobile)

### Pending Todos

- Opravit kontrast warning textu v light mode (`modal_dialogs/settings.rs`)
- Upravit dolni listu: branch oznameni vice doprava, UTF vice doleva (`src/app/ui/terminal/bottom/git_bar.rs`)
- V `Sestavit > Upravit` pridat online nahled zmen zapisovanych do `.polycredo/profiles.toml`
- Zprovoznit `.polycredo/trash` a presouvat tam smazane soubory
- Nabidnout otevirani projektu v aktualnim okne nebo v novem

### Blockers/Concerns

- Migrace approval/security casti bez regresi muze odhalit skryte couplingy mimo `src/app/cli/*`.

---

## Quick Tasks Completed

| #  | Description | Date | Commit | Directory |
|----|-------------|------|--------|-----------|
| ... | (pokracovani z historie) | | | |
| 9 | GitHub Actions Windows build: localtime + warningy | 2026-03-11 | f3ba79e | .planning/quick/9-github-action-windows-build-fix-localtim |
| 10 | cargo test vykazuje chybu, podivej se na to a oprav ji | 2026-03-11 | 6cbb8dc | .planning/quick/10-cargo-test-vykazuje-chybu-podivej-se-na- |

---
*Last updated: 2026-03-12*
