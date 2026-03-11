---
phase: 31
slug: ai-terminal-runtime-migration
plan: 05
status: passed
updated: 2026-03-11
verification_type: goal-backward
---

# Phase 31 - Goal-Backward Verification

## Final Status

`passed`

## Goal-Backward Verdict

**Goal fáze:** migrace provider/executor/tooling na AI terminal bez legacy CLI vrstvy.

Verdikt: **PASS**.

Důkazy proti cíli:
- Runtime entrypoint pro prompt běží přes AI terminal flow (`send_query_to_agent`) a stream startuje přes `provider.stream_chat(...)` bez `app::cli` závislostí.
- Slash/GSD dispatch je centralizovaný v AI terminal modulu (`dispatch`) a navazuje na externí assistant workflow.
- Security + approval + audit běží přes `ai_core` (`ToolExecutor`, `PathSandbox`, `RateLimiter`, `AuditLogger`).
- `src/app/cli` v codebase neexistuje a grep nevrací importy `crate::app::cli`.

## Plan Frontmatter + REQUIREMENTS Coverage

| Requirement | Plan frontmatter coverage | REQUIREMENTS coverage | Code/Test evidence | Result |
|---|---|---|---|---|
| TERM-01 | 31-01, 31-02, 31-04 | mapped to Phase 31 | `logic.rs` (`send_query_to_agent`), `cargo test ai_chat -- --nocapture` PASS | PASS |
| TERM-02 | 31-02, 31-04 | mapped to Phase 31 | `background.rs` non-blocking polling (`try_recv`, `drain_stream_events`), `cargo check` PASS | PASS |
| TERM-03 | 31-01, 31-02, 31-04, 31-05 | mapped to Phase 31 | `slash.rs` (`dispatch`, `/gsd`) + assistant-only AI bar bez provider-picker UI, `cargo test gsd -- --nocapture` PASS | PASS |
| SAFE-01 | 31-03, 31-04 | mapped to Phase 31 | `executor.rs` (`ToolResult::NeedsApproval`, `process_approval_response`), `cargo test approval -- --nocapture` PASS (11/11) | PASS |
| SAFE-02 | 31-03, 31-04 | mapped to Phase 31 | `security.rs` (`validate_path`, `PathSandbox`, `RateLimiter`, blacklisty), enforcement v `executor.rs`, `cargo test security -- --nocapture` PASS (28/28) | PASS |
| SAFE-03 | 31-01, 31-03, 31-04 | mapped to Phase 31 | `audit.rs` (`ai-audit.log`), visible error surfacing v `background.rs` (`Toast::error` + stream error text), `./check.sh` PASS | PASS |

## Must_Haves Verification (Current Codebase)

### 31-01 must_haves
- Truth: AI terminal runtime je jediná cesta bez návratu na legacy CLI.
  - Evidence: `src/app/cli` chybí; grep bez `crate::app::cli`; AI chat flow v `src/app/ui/terminal/ai_chat/logic.rs`.
  - Result: PASS
- Truth: Prompt entrypoint + panel flow zůstává funkční.
  - Evidence: `send_query_to_agent`, `stream_chat`, `cargo test ai_chat` PASS.
  - Result: PASS
- Truth: Slash/GSD je external/terminal workflow bez legacy PolyCredo slash vrstvy.
  - Evidence: `src/app/ui/terminal/ai_chat/slash.rs` (`dispatch`, `/gsd`, async guardy).
  - Result: PASS
- Truth: Chyby jsou viditelné.
  - Evidence: `src/app/ui/background.rs` (`StreamEvent::Error` -> text do konverzace + `Toast::error`).
  - Result: PASS

### 31-02 must_haves
- Truth: Prompt -> stream -> done tok je neblokující.
  - Evidence: `try_recv` polling + `drain_stream_events` v `background.rs`.
  - Result: PASS
- Truth: Assistant-only flow zustava funkcni bez provider-picker UI.
  - Evidence: `src/app/ui/terminal/right/ai_bar.rs` obsahuje pouze assistant picker + Start.
  - Result: PASS
- Truth: Slash/GSD async placeholder update respektuje generation guard.
  - Evidence: `slash.rs` (`should_apply_async_result` + testy generation guardu).
  - Result: PASS

### 31-03 must_haves
- Truth: Approval flow approve/deny/resume je zachovaný.
  - Evidence: `executor.rs` (`NeedsApproval`, `process_approval_response`) + `background.rs` approval event handling.
  - Result: PASS
- Truth: Security guardy nejdou obejít.
  - Evidence: centralizované checky v `security.rs` + enforcement v `executor.rs`.
  - Result: PASS
- Truth: Audit + error handling jsou průkazné a viditelné.
  - Evidence: `audit.rs` zápisy + `background.rs` user-visible error feedback.
  - Result: PASS

### 31-04 must_haves
- Truth: Explicitní acceptance důkazy pro všechny TERM/SAFE.
  - Evidence: tato matice + reprodukovatelné příkazy níže.
  - Result: PASS
- Truth: Locked boundary (AI terminal-only, bez PolyCredo CLI).
  - Evidence: žádný `app::cli` namespace, `src/app/cli` neexistuje, runtime flow v `ai_core` + `ui/terminal/ai_chat`.
  - Result: PASS
- Truth: Quality gate zelený.
  - Evidence: `cargo check` PASS, `./check.sh` PASS.
  - Result: PASS

### 31-05 gap-closure must_haves
- Truth: Source-of-truth boundary je assistant-only bez provider-picker couplingu.
  - Evidence: `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`, `31-CONTEXT.md` aligned.
  - Result: PASS
- Truth: AI terminal runtime path nema aktivni coupling na `ws.ai.ollama.*` v cilovych call-sitech.
  - Evidence: grep gate PASS pro `ai_bar.rs`, `logic.rs`, `background.rs`, `workspace/state/mod.rs`.
  - Result: PASS
- Truth: SAFE approval/security parity zustava beze zmeny.
  - Evidence: `cargo test approval -- --nocapture` (11/11 PASS), `cargo test security -- --nocapture` (28/28 PASS), audit/path/rate-limit symboly pritomne.
  - Result: PASS

## Executed Verification Commands (2026-03-11)

```bash
RUSTC_WRAPPER= cargo check
cargo test ai_chat -- --nocapture
cargo test gsd -- --nocapture
RUSTC_WRAPPER= cargo test approval -- --nocapture
RUSTC_WRAPPER= cargo test security -- --nocapture
RUSTC_WRAPPER= ./check.sh
```

Výsledek: všechny příkazy **PASS**.

## Residual Risk Check

- Nalezeny pouze názvové relikty konstant (`config::CLI_VERSION`, `config::CLI_TIER`), ale bez návratu legacy CLI vrstvy do runtime architektury.
- To je kosmetické pojmenování, ne funkční porušení cíle fáze 31.

## Final Decision

Status: `passed`
