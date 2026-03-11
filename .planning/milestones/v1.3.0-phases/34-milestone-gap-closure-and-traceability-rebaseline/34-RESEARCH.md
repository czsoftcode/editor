# Phase 34 Research - Milestone Gap Closure and Traceability Rebaseline

## Kontext a vstupy

Tento research navazuje na audit `.planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md` (stav `gaps_found`, score 11/15), kde jsou R33-A/B/C/D oznacene jako `unsatisfied` primarne kvuli tomu, ze `33-VERIFICATION.md` je stale `status: gaps_found`.

Aktualni stav artefaktu:
- `.planning/REQUIREMENTS.md` mapuje `R33-A..R33-D` na Phase 34 (Pending).
- `.planning/ROADMAP.md` obsahuje novou Phase 34 se scope na re-baseline verifikace a traceability.
- `.planning/phases/33-.../33-VERIFICATION.md` potvrzuje PASS pro R33-A/B/C, ale FAIL pro R33-D kvuli zakazanym patternum v aktivnim planning scope.
- `.planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md` stale referencuje cesty/soubory odstranene ve phase 33 (`ai_chat`, `ai_core`) a vytvari auditni drift.
- `.planning/STATE.md` je stale ve stavu "milestone complete" pro v1.3.0, ktery je v konfliktu s audit verdictem `gaps_found`.

## Problem Statement

Milestone v1.3.0 neni blokovany funkcionalitou runtime kodu, ale nekonzistenci planning/verifikacni evidence po launcher-only removalu ve phase 33.

Kriticke body k uzavreni:
- R33-D nema konzistentni dukazni retezec v aktivnim planning scope.
- Cross-phase drift: phase 31/32 texty a odkazy nejsou plne rebaselined na post-phase33 realitu.
- Status drift mezi REQUIREMENTS/ROADMAP/STATE/VERIFICATION artefakty.

## Scope pro fazi 34 (R33-A, R33-B, R33-C, R33-D)

### R33-A
Potvrdit, ze jediny aktivni AI tok je `ai_bar -> terminal.send_command`, bez alternativnich entrypointu.

### R33-B
Potvrdit trvale odstraneni runtime/chat modulu a aktivnich referenci (`ai_core`, `ui/terminal/ai_chat`, stare enum branch/dispatch cesty).

### R33-C
Potvrdit absenci legacy fallback/deprecated vetvi v UI/i18n/planning callsitech, ktere by znovu zavadely CLI chat behavior.

### R33-D
Dokoncit planning cleanup tak, aby zakazane terminy byly odstranene nebo neutralizovane dle Phase 33 planu v aktivnim scope (minimalne REQUIREMENTS, ROADMAP, STATE, 33-VERIFICATION a phase34 artefakty).

## Navrhovany technicky pristup

1. Evidence-first revalidation phase 33
- Znovu spustit grep/build gate s jednoznacnym command-evidence logem.
- Opravit/aktualizovat `33-VERIFICATION.md` tak, aby reflektovalo finalni stav (bez stale FAIL artefaktu, pokud budou nalezy odstraneny).

2. Traceability rebaseline napric artefakty
- Synchronizovat status v:
  - `.planning/REQUIREMENTS.md` (R33-A..D -> Complete po dukazech),
  - `.planning/ROADMAP.md` (Phase 34 coverage + closure),
  - `.planning/STATE.md` (milestone status a posledni aktivita),
  - `31-VERIFICATION.md` (odstranit reference na neexistujici phase33-smazane cesty; zachovat TERM/SAFE dokazatelnost bez historicke nepresnosti),
  - pripadne `32-VERIFICATION.md`, pokud obsahuje phase33-neplatne reference.

3. Milestone audit closure
- Aktualizovat nebo regenerovat audit tak, aby final verdict prepnul z `gaps_found` na `passed` pouze pokud bude 15/15 requirement coverage prokazatelne a konzistentni.

## Artefakty, ktere maji byt upraveny ve phase 34

- `.planning/phases/33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu/33-VERIFICATION.md`
- `.planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md` (nebo navazujici closure addendum, podle zvoleneho workflow)

## Rizika a mitigace

- Riziko: "remove-only" cleanup muze rozbit historickou citelnost phase 31.
  - Mitigace: zachovat explicitni poznamku, ze reference byly rebaselined po phase33 removalu, bez prepisu funkcni podstaty TERM/SAFE dukazu.

- Riziko: false-positive grep patterny v planning textech.
  - Mitigace: oddelit zakazane production patterny od auditnich/metodickych patternu; pouzivat presne scope souboru a fixni regex seznam.

- Riziko: stavova nekonzistence po lokalni editaci vice planning souboru.
  - Mitigace: zavest jednotny final gate, kde se kontroluje status alignment mezi REQUIREMENTS/ROADMAP/STATE/VERIFICATION.

## Validation Architecture

### Fast validation (iteracni smycka, do 1-2 minut)

Cil: rychle potvrdit, ze planning a kodovy scope neobsahuje relapsy pro R33-A/B/C/D.

Prikazy:

```bash
# 1) Launcher-only dispatch stale aktivni
rg -n "send_selected_agent_command|terminal\.send_command" src/app/ui/terminal/right/ai_bar.rs -S

# 2) Odstranene moduly realne neexistuji
test ! -d src/app/ai_core && test ! -d src/app/ui/terminal/ai_chat

# 3) Bez aktivnich referenci na stare runtime/chat entrypointy
! rg -n "\\bai_core\\b|ui/terminal/ai_chat|show_ai_chat|tool_executor|FocusedPanel::AiChat|run_agent\\s*==\\s*\"ai_chat\"" src -S

# 4) Bez fallback/deprecated AI chat patternu v relevantnich UI callsitech
! rg -n "fallback|deprecated ai|removed ai chat|legacy ai chat|toast.*ai" \
  src/app/ui/terminal/right/ai_bar.rs \
  src/app/ui/terminal/right/mod.rs \
  src/app/ui/workspace/mod.rs \
  src/app/ui/workspace/menubar/mod.rs \
  src/app/ui/panels.rs -S

# 5) Planning cleanup gate pro aktivni scope (R33-D)
! rg -n "puvodni CLI integrace|odstraneny runtime modul|odstraneny chat modul|legacy CLI namespace" \
  .planning/STATE.md \
  .planning/ROADMAP.md \
  .planning/REQUIREMENTS.md \
  .planning/phases/33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu/33-VERIFICATION.md \
  .planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md -S
```

### Full validation (final gate pred uzavrenim)

Cil: potvrdit, ze traceability je konzistentni napric artefakty a build/test gate zustava zeleny.

Prikazy:

```bash
# Build/test gate (s lokalnim workaroundem na sccache permission blocker)
RUSTC_WRAPPER= cargo check
RUSTC_WRAPPER= ./check.sh

# Traceability alignment kontrola
rg -n "R33-A|R33-B|R33-C|R33-D" \
  .planning/REQUIREMENTS.md \
  .planning/ROADMAP.md \
  .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md \
  .planning/phases/33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu/33-VERIFICATION.md -S

# Status alignment (gaps_found nesmi zustat v closure artefaktech pro v1.3.0 final)
! rg -n "status:\s*gaps_found" \
  .planning/phases/33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu/33-VERIFICATION.md \
  .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md -S
```

Strategie:
- Fast validation spoustet po kazdem patchi planning souboru.
- Full validation spustit az po finalnim sjednoceni REQUIREMENTS/ROADMAP/STATE/VERIFICATION.
- Teprve po full PASS prepnout milestone verdict na passed.

## Exit Criteria pro phase 34

- R33-A/B/C/D maji jednoznacny PASS dukaz bez konfliktu mezi artefakty.
- `33-VERIFICATION.md` uz neni `gaps_found`.
- Milestone audit v1.3.0 je konzistentni s REQUIREMENTS a ROADMAP (bez score driftu).
- `RUSTC_WRAPPER= cargo check` a `RUSTC_WRAPPER= ./check.sh` jsou PASS.
