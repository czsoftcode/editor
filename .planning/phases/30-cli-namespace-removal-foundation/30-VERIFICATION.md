# Phase 30 Goal-Backward Verification

Datum verifikace: 2026-03-11
Fáze: `30-cli-namespace-removal-foundation`
Status: `passed`

## Ověřený cíl fáze

Cíl z roadmapy/contextu: odstranit vazby na `app::cli` a připravit čistý základ pro AI terminal-only architekturu.

Goal-backward verdict:
- `app::cli` vazby v `src/` nejsou přítomné (`rg -n "crate::app::cli|app::cli" src` -> žádný nález).
- legacy strom `src/app/cli/*` je fyzicky odstraněn (`test ! -d src/app/cli` -> PASS).
- modulový strom je přepnutý na `app::ai_core` (`src/app/mod.rs` obsahuje `pub mod ai_core;`, bez `pub mod cli;`).
- quality gate je zelená (`RUSTC_WRAPPER= cargo check`, `./check.sh`).

Tím je foundation cíl fáze 30 splněn a základ pro AI terminal-only architekturu je připraven.

## Coverage CLI-01 / CLI-02 / CLI-03

### Plan frontmatter vs REQUIREMENTS

- CLI-01:
  - frontmatter: `30-02-PLAN.md`
  - REQUIREMENTS: `CLI-01` = complete, mapped to phase 30
  - stav v kódu: `src/app/cli` neexistuje
  - verdict: PASS

- CLI-02:
  - frontmatter: `30-01-PLAN.md`, `30-04-PLAN.md`, `30-03-PLAN.md`
  - REQUIREMENTS: `CLI-02` = complete, mapped to phase 30
  - stav v kódu: žádné `app::cli` reference v `src/`
  - verdict: PASS

- CLI-03:
  - frontmatter: `30-03-PLAN.md`
  - REQUIREMENTS: `CLI-03` = complete, mapped to phase 30
  - stav v kódu: žádné `mod cli`/`pub mod cli` exporty v `src/app`
  - verdict: PASS

Coverage závěr: CLI-01/CLI-02/CLI-03 jsou konzistentně pokryté mezi plan frontmatter, REQUIREMENTS a aktuálním codebase stavem.

## Must_haves verifikace proti codebase

### 30-01 must_haves
- Root namespace přepnutý na `pub mod ai_core;` bez public compat aliasu `cli`: PASS (`src/app/mod.rs`).
- Core settings/types/workspace-state bez `crate::app::cli::*`: PASS (globální grep audit + phase30 testy).
- `app::ai_core` existuje jako compile-first surface: PASS (`src/app/ai_core/mod.rs`, `src/app/ai_core/types.rs`).

### 30-04 must_haves
- AI terminal UI bez Ollama model list/connection status prvků: PASS (`rg` na `OllamaConnectionStatus|selected_model|model_info|ollama` v subsetu -> žádný nález).
- Assistant-only flow zachován: PASS (výskyty `ai-label-assistant` v `ai_bar.rs` a `ai_chat/render.rs`).
- AI terminal UI/background bez `crate::app::cli::*`: PASS (subset + globální grep audit).

### 30-02 must_haves
- `src/app/cli/*` fyzicky odstraněno: PASS.
- Hard removal po migračních krocích v green stavu: PASS (phase artifact testy + aktuální `cargo check`/`./check.sh`).
- Bez orphan namespace/include vazeb: PASS (build green, grep clean).

### 30-03 must_haves
- Bez mrtvých `mod`/`pub use` exportů vázaných na CLI vrstvu: PASS (`rg -n "pub mod cli|mod cli" src/app` -> žádný nález).
- Finální grep bez `app::cli` reference ve `src/`: PASS.
- Povinná quality gate `./check.sh` green: PASS.

## Důkazní příkazy a výsledky

```bash
test ! -d src/app/cli
# PASS

rg -n "crate::app::cli|app::cli" src
# no matches (exit 1 = expected)

rg -n "pub mod cli|mod cli" src/app
# no matches (exit 1 = expected)

rg -n "OllamaConnectionStatus|selected_model|model_info|ollama" src/app/ui/terminal/ai_chat src/app/ui/terminal/right/ai_bar.rs
# no matches (exit 1 = expected)

RUSTC_WRAPPER= cargo check
# PASS

./check.sh
# PASS (fmt + clippy + tests)
```

## Finální verdict

Phase 30 je **explicitně `passed`**.
