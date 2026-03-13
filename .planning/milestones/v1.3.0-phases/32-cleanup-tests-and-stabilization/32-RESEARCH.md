# Phase 32 Research: Cleanup, Tests, and Stabilization

**Phase:** 32-cleanup-tests-and-stabilization  
**Date:** 2026-03-11  
**Scope guard:** pouze cleanup/testy/dokumentace pro uzavření STAB-01 a STAB-02. Bez nových produktových capability.

## Research Summary

Phase 32 je čistě stabilizační fáze po odstranění CLI vrstvy. Kritické je:
- zavřít quality gate (`cargo check` + `./check.sh`) deterministicky,
- doplnit/aktualizovat regresní testy tak, aby explicitně hlídaly namespace migraci a klíčové assistant-only toky,
- uzavřít planning artefakty evidence-first stylem.

Nejvyšší riziko není nová funkcionalita, ale tiché regrese v prompt/stream/slash/approval toku a návrat od `ai_core` zpět na legacy `app::cli` reference.

## Standard Stack

Použít stávající stack bez změn:
- Rust test harness (`cargo test`)
- Quality gate skript `./check.sh` (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all-targets --all-features`)
- Textové audit/regression testy v `tests/phase30_plan*.rs` jako základ pro STAB namespace guard
- Stávající interní unit testy v:
  - `src/app/ui/terminal/ai_chat/logic.rs`
  - `src/app/ui/terminal/ai_chat/slash.rs`
  - `src/app/ui/background.rs`
  - `src/app/ai_core/executor.rs`

## Architecture Patterns

Použít tento postup (bez refaktoru architektury):
1. Nejprve stabilizační baseline: zjistit přesný stav failů (`cargo check`, `./check.sh`, cílené testy).
2. Poté pouze low-risk cleanup zásahy v aktivním scope (`src/`, `tests/`, aktivní planning docs v1.3).
3. Následně doplnit STAB regresní testy tak, aby validovaly:
   - absence `app::cli` namespace v kritických callsitech,
   - funkční guardy stream/slash/approval/retry,
   - konzistenci planning evidence.
4. Nakonec evidence-first verifikace (`32-VERIFICATION.md`) + docs sync (`ROADMAP.md`, `STATE.md`, `REQUIREMENTS.md`, `CHANGELOG.md`).

## Don't Hand-Roll

Nezavádět nové frameworky, vlastní test runner ani nové validační pipeline:
- Nepřepisovat `check.sh`; používat existující gate.
- Nevytvářet nové integrační harnessy, pokud stejný efekt pokryje cílený test v existujících modulech.
- Neotevírat nový feature scope (UI redesign, provider změny, nové runtime capability).

## Common Pitfalls

- „Green `cargo check`, red `./check.sh`“ kvůli `fmt`/`clippy` driftu mimo právě měněný soubor.
- STAB-02 splněn pouze implicitně (testy existují), ale bez explicitní vazby na nový namespace guard.
- Oprava mimo scope, která nechtěně mění assistant-only chování.
- Nedostatečná evidence v docs (PASS/FAIL bez příkazů, bez mapování na STAB-01/02).

## Actionable Research: STAB-01

### Cíl
Prokazatelně uzavřít `STAB-01`: `cargo check` a `./check.sh` prochází po odstranění CLI vrstvy.

### Doporučená implementační sekvence
1. Spustit baseline:
   - `cargo check`
   - `./check.sh`
2. Pokud `./check.sh` selže, triage podle kroku:
   - `cargo fmt --all -- --check` fail: provést pouze nutné formátovací opravy v aktivním scope.
   - `cargo clippy --all-targets --all-features -- -D warnings` fail: opravit warningy jen v dotčených stabilizačních souborech.
   - `cargo test --all-targets --all-features` fail: mapovat na STAB-02 test aktualizace.
3. Po každé opravě opakovat celý gate (`cargo check` + `./check.sh`), ne jen dílčí krok.
4. Do verifikace zapisovat přesné command evidence (command + PASS/FAIL).

### Evidence minimum pro 32-VERIFICATION
- `cargo check: PASS`
- `./check.sh: PASS`
- Datum a commit SHA/working-tree reference, na kterých byl gate spuštěn.

## Actionable Research: STAB-02

### Cíl
Prokazatelně uzavřít `STAB-02`: relevantní testy aktualizované na nový namespace a stabilizační toky.

### Co už je využitelné
- `tests/phase30_plan01_foundation_imports.rs`
- `tests/phase30_plan02_readiness_gate.rs`
- `tests/phase30_plan04_ai_terminal_imports.rs`

Tyto testy už hlídají zákaz `crate::app::cli` v kritických souborech. Pro STAB-02 je vhodné je rozšířit nebo doplnit phase32 test soubor(y), aby pokrývaly aktuální aktivní callsite subset po phase 31.

### Doporučená test strategie (minimální, cílená)
1. Přidat phase32 regression test(y) do `tests/` pro namespace guard:
   - explicitní seznam kritických souborů z assistant-only toku,
   - assert na zákaz `crate::app::cli`/`app::cli`.
2. Zachovat a případně zpřesnit interní unit testy pro runtime guardy:
   - `logic.rs`: normalizace promptu + validace modelu,
   - `slash.rs`: command matching + async generation guard,
   - `background.rs`: stream disconnect/error handling,
   - `executor.rs`: approval decision flow.
3. Spouštět cílené testy před full gate:
   - `cargo test phase30_plan`
   - `cargo test slash::tests`
   - `cargo test approval`
   - `cargo test background::tests`
4. Poté potvrdit full gate přes `./check.sh`.

### Done kritéria pro STAB-02
- Existuje explicitní phase32 namespace regression coverage v `tests/`.
- Cílené stabilizační testy jsou green.
- Full gate (`./check.sh`) je green bez výjimek.

## Validation Architecture

Nyquist-ready validační architektura pro phase 32:

### 1) Validation Layers
- Layer A: Compile/quality gate (`cargo check`, `./check.sh`) => STAB-01
- Layer B: Namespace regression guard testy (`tests/phase30_*` + phase32 doplnění) => STAB-02
- Layer C: Runtime guard testy (slash/stream/approval/retry jednotky) => stabilita klíčových toků
- Layer D: Docs traceability (`REQUIREMENTS.md`, `ROADMAP.md`, `STATE.md`, `32-VERIFICATION.md`, `CHANGELOG.md`) => audit uzávěra

### 2) Requirement Mapping
- STAB-01 <- Layer A (hard gate, povinné PASS)
- STAB-02 <- Layer B + Layer C (namespace + runtime regresní jistota)

### 3) Execution Order
1. Layer B/C (rychlá regresní detekce)
2. Layer A (`cargo check`, `./check.sh`)
3. Layer D (evidence-first zápis)

### 4) Evidence Contract
Každý validační krok musí mít v `32-VERIFICATION.md`:
- přesný příkaz,
- výsledek PASS/FAIL,
- krátké mapování na requirement (`STAB-01`/`STAB-02`),
- poznámku „mimo-scope“ u případných nerealizovaných nápadů.

## Suggested Task Breakdown for Planning

1. Baseline gate run + triage failů (bez feature změn).
2. Minimal cleanup fixy v aktivním scope (`src/`, `tests/`, aktivní docs).
3. STAB-02 test update (phase32 namespace guard + cílené runtime test potvrzení).
4. Full gate rerun (`cargo check` + `./check.sh`) do green stavu.
5. Dokumentační stabilizace (`ROADMAP`, `STATE`, `REQUIREMENTS`, `CHANGELOG`, `32-VERIFICATION`).

## Out-of-Scope Enforcement

V phase 32 neimplementovat:
- nové AI capability,
- změny UX nad rámec oprav regresí,
- velké cross-module refaktory,
- historický cleanup archivních milestone artefaktů.
