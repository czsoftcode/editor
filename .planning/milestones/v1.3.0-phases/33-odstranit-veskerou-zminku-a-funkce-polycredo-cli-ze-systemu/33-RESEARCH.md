# Phase 33 Research: odstranit veskerou zminku a funkce polycredo cli ze systemu

**Phase:** 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu  
**Date:** 2026-03-11  
**Scope guard:** ponechat pouze `ai_bar -> terminal.send_command`, odstranit `src/app/ai_core/*` + `src/app/ui/terminal/ai_chat/*` + navazane reference bez fallback UX/toastu.

## Research Summary

Pro dobry plan Phase 33 je nutne nejdriv uzamknout hranici: aplikace ma po fazi fungovat jako terminal launcher pro custom agenty (pres `ai_bar`) a nesmi obsahovat integrovany AI chat/runtime/executor.

Nejvetsi riziko neni samotne mazani slozek, ale sirka couplingu mimo ne:
- `WorkspaceState` drzi AI runtime stavy (`ai`, `show_ai_chat`, `tool_executor`, slash/approval state).
- `workspace/mod.rs`, `panels.rs`, `background.rs`, `menubar/mod.rs` volaji `terminal::ai_chat::*` a `ai_core::*`.
- `settings.rs` a `app/types.rs` importuji AI enums z `ai_core`.
- locale soubory (`locales/*/cli.ftl`) obsahují velke mnozstvi textu na mazane UI flows.
- planning/historicke artefakty obsahuji mnoho referenci na `ai_chat`/`ai_core`/`PolyCredo CLI`.

Plan-ready zaver: fazi je potreba rozdelit na kodovy removal, i18n cleanup, a explicitne definovany policy-driven cleanup planning artefaktu (jinak scope exploduje).

## Standard Stack

Pouzit stavajici stack bez novych nastroju:
- `rg` pro inventory a forbidden-pattern guardy.
- `cargo check` pro rychle compile gate po kazdem removal kroku.
- `./check.sh` jako final quality gate.
- Cileny grep audity v `src`, `locales`, `.planning`.

## Architecture Patterns

Doporuceny implementacni vzor pro plan:
1. **Inventory freeze:** vyrobit explicitni seznam aktivnich souboru, ktere smi odkazovat AI launcher (`ai_bar`, `terminal/right`, `registry agents`, `terminal instance`).
2. **Hard removal runtime/chat:** odstranit moduly `ai_core` a `terminal/ai_chat` a vsechny compile reference.
3. **State flattening:** z `WorkspaceState` odstranit AI-chat/runtime pole a souvisejici helpery, ktere uz nejsou potrebne pro launcher-only model.
4. **UI entrypoint cleanup:** odstranit vsechny trigger body `show_ai_chat` (`Ctrl+Alt+G`, plugin bar start, menu run_agent ai_chat, command-palette navaznosti).
5. **i18n cleanup:** odstranit `cli.*` texty navazane na odstranene flow a zachovat jen texty potrebne pro `ai_bar`/terminal launcher.
6. **Planning artifact cleanup:** provest dle predem schvalene politiky (viz sekce Open Questions), protoze user chce globalni odstraneni zminen vcetne historie.
7. **Validation pass:** compile + quality gate + grep evidence + docs traceability update.

## Scope Map (What Stays vs Removes)

### Must stay
- `src/app/ui/terminal/right/ai_bar.rs`
- `src/app/ui/terminal/right/mod.rs` (AI panel shell + tabs + terminal body)
- `terminal.send_command(...)` flow do aktivniho claude terminal tabu
- registry custom agentu (`settings.custom_agents`)

### Must remove (code)
- `src/app/ai_core/*`
- `src/app/ui/terminal/ai_chat/*`
- `pub mod ai_core;` v `src/app/mod.rs`
- `pub mod ai_chat;` v `src/app/ui/terminal/mod.rs`
- vsechny `use crate::app::ai_core::*` a `terminal::ai_chat::*` reference
- stale runtime state v `WorkspaceState` (`show_ai_chat`, `ai`, `tool_executor`, pending approvals, slash async/autocomplete fields)

### Must remove (UX behavior)
- vsechny vstupy, ktere oteviraji/spousti AI chat (menu, hotkeys, plugin bar start->ai_chat)
- fallback toasty/disabled hlasky pro odstranene akce (explicitne zakazano)

### Must remove (textual mentions)
- `PolyCredo CLI` zminky v aktivnich UI textech
- `cli-chat*`, `cli-tool*`, `cli-settings-section` klice v `locales/*/cli.ftl`, pokud nejsou uz pouzivane
- planning/historicke zminky dle schvalene strategie rozsahu

## Dependency Map

Kriticke compile zavislosti pro plan:
- `src/settings.rs` + `src/app/types.rs` zavisi na `AiExpertiseRole/AiReasoningDepth` z `ai_core`.
- `src/app/ui/workspace/state/init.rs` instanciuje `AiState` a navazny chat/ollama state.
- `src/app/ui/background.rs` je silne navazan na `ai_core::executor`, `AiMessage`, slash stale guards.
- `src/app/ui/widgets/ai/chat/*` importuje `ai_core` a `terminal::ai_chat::slash`.
- `src/app/ui/panels.rs` stale spousti `terminal::ai_chat::handle_action`.

Duvsledek pro plan: nestaci jen smazat adresare; je nutna navazna simplifikace `workspace state + background + widgets`.

## Common Pitfalls

- Smazani modulu bez odstraneni `show_ai_chat` flow -> desitky compile erroru v workspace orchestrace.
- Ponechani orphan i18n klicu v casti kodu -> runtime missing-key warnings nebo mrtve texty.
- Prilis agresivni globalni prepis historickych planning souboru -> vysoky diff noise a riziko ztraty auditni stopy.
- Zasah do `ai_bar` tak, ze prestane fungovat `selected_agent_id -> send_command` cesta.

## Don't Hand-Roll

- Nezavadet novy AI subsystem ani mezivrstvu nahrazujici `ai_core`.
- Nepridavat fallback toasty/soft deprecation obrazovky pro odstranene funkce.
- Nepsat custom skripty na masivni rewrite bez pattern guardu a revize diffu po segmentech.

## Plan-Readiness Inputs (co musi byt rozhodnuto pred PLAN)

1. **Historical/planning cleanup policy**
- Potvrdit, zda "globalne" znamena opravdu prepis i archivnich milestone/fazi souboru v `.planning/**`.
- Doporuceni: rozdelit na `active artifacts mandatory` + `historical optional but requested`, aby slo mit kontrolovatelne commity.

2. **Settings contract po odstraneni ai_core**
- Rozhodnout, zda `ai_expertise`, `ai_reasoning_depth`, `ollama_*`, `ai_default_model`, `ai_file_blacklist_patterns` zustanou (neuzite) nebo budou odstraneny z `Settings`/migrace.
- Bez tohoto rozhodnuti nelze stabilne upravit `settings.rs`, `types.rs`, `workspace/state/init.rs`.

3. **UI panel naming semantics**
- Potvrdit, zda texty "AI panel" v menu zustavaji (jako terminal launcher) nebo se maji prejmenovat.
- To ovlivni lokalizace a command labels.

4. **Acceptance for no-fallback behavior**
- Potvrdit, ze odstranenim triggeru (`Ctrl+Alt+G`, plugin bar start chat, `run_agent == ai_chat`) nevznika nahradni UX.

## Suggested Task Breakdown for Planning

1. Vytvorit `forbidden-pattern` seznam pro phase 33 (`ai_core`, `terminal/ai_chat`, `show_ai_chat`, `tool_executor`, `cli-chat`, `cli-tool`, `PolyCredo CLI`).
2. Odstranit moduly `ai_core` + `terminal/ai_chat` + mod declarations.
3. Refaktorovat `WorkspaceState` a `init_workspace` na launcher-only model.
4. Vycistit `workspace/mod.rs`, `panels.rs`, `background.rs`, `menubar/mod.rs`, `widgets/ai/chat/*` od chat/runtime flow.
5. Provest i18n cleanup v `locales/*/cli.ftl` + pripadne presunout zbyvajici potrebne klice.
6. Provest planning artifact cleanup podle schvalene policy.
7. Spustit validation gate (`cargo check`, `./check.sh`, grep audity) a zapsat evidence.

## Validation Architecture

### 1) Validation Layers
- **Layer A: Compile gate**
  - `cargo check`
  - Cíl: zero compile references na odstranene moduly.
- **Layer B: Quality gate**
  - `./check.sh`
  - Cíl: format/clippy/test bez regresi.
- **Layer C: Forbidden-pattern audit (source + locales)**
  - `rg -n "ai_core|ui/terminal/ai_chat|show_ai_chat|tool_executor" src`
  - `rg -n "cli-chat|cli-tool|PolyCredo CLI" locales src`
  - Cíl: zbyva pouze launcher flow a jeho legitimni texty.
- **Layer D: Planning/global mention audit**
  - `rg -n "PolyCredo CLI|ai_core|ai_chat|app::cli" .planning`
  - Cíl: splnit user-requested global cleanup policy v dohodnutem rozsahu.

### 2) Requirement Mapping for this phase
- R33-A: pouze `ai_bar -> send_command` launcher behavior aktivni.
- R33-B: integrovany AI runtime/chat kompletne odstranen.
- R33-C: zadne fallback UX/toasty pro legacy akce.
- R33-D: textual mentions cleanup proveden v kodu/locales/.planning dle schvalene policy.

Mapovani vrstev:
- R33-A <- Layer A + manual smoke (start agenta z ai_bar)
- R33-B <- Layer A + Layer C
- R33-C <- code review checklist + Layer C
- R33-D <- Layer D

### 3) Execution Order
1. Layer C pre-check (baseline inventory)
2. Kodove removal kroky + prubezny Layer A
3. Layer B full gate
4. Layer C post-check
5. Layer D (planning/global mentions) + evidence zapis

### 4) Evidence Contract
Do verifikace phase 33 zapisovat:
- presne prikazy,
- PASS/FAIL,
- mapovani na R33-A..R33-D,
- seznam vedomych vyjimek (pokud nejaka historical zminka zustane z auditnich duvodu).

## Open Questions / Unknowns

- Je pozadavek na globalni cleanup historickych artefaktu absolutni, nebo lze ponechat auditni fakta ve verifikacnich dokumentech starsich fazi?
- Ma po odstraneni runtime zustat `src/app/ui/widgets/ai/*` jako reusable UI komponenty, nebo se maji take odstranit?
- Ma se zachovat soucasny `FocusedPanel::AiChat` enum variant, nebo ji odstranit spolu s flow?

Tyto body je vhodne explicitne uzavrit pred psanim `33-PLAN.md`, jinak hrozi opakovane preplanujovani scope.
