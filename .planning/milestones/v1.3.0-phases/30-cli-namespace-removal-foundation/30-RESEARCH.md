# Phase 30 Research: CLI Namespace Removal Foundation

Datum: 2026-03-11
Faze: 30-cli-namespace-removal-foundation
Scope: CLI-01, CLI-02, CLI-03
Stav: ready-for-planning

## Research Summary
Phase 30 je ciste foundation rez: odstranit navazani na `app::cli`, zavest cilovy namespace `app::ai_core`, udrzet build stabilitu a odstranit mrtve vazby. Behavioral parity AI terminalu (streaming, slash/GSD, safety runtime detaily) je vedome navazujici prace faze 31/32.

## Source Inputs
- `.planning/phases/30-cli-namespace-removal-foundation/30-CONTEXT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `./CLAUDE.md`

## Scope Lock (Phase 30)
- In-scope:
  - Namespace migration z `app::cli` na `app::ai_core`.
  - Odstraneni `src/app/cli/*` po premigrovani importu.
  - Odstraneni mrtvych exportu/modulu vazanych na puvodni CLI vrstvu.
  - Build gate: `cargo check` a `./check.sh`.
- Out-of-scope:
  - Plna runtime/UX parity AI terminalu (patri do faze 31/32).
  - Vetsi redesign architektury mimo cil CLI namespace cleanup.

## Requirement Coverage Strategy
- CLI-01: `src/app/cli/*` bude fyzicky odstraneno az po dokonceni import migration pass.
- CLI-02: V `src/` nezustane zadny import `crate::app::cli::*`; build musi projit bez techto odkazu.
- CLI-03: Probehne dead-code cleanup pass (mrtve `pub mod`, re-exporty, osiřele vazby na starou vrstvu).

## Impacted Areas (planning inventory)
Primarni migrovane body podle kontextu:
- `src/settings.rs`
- `src/app/types.rs`
- `src/app/mod.rs`
- `src/app/ui/background.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/terminal/ai_chat/*`
- `src/app/ui/terminal/right/ai_bar.rs`
- `src/app/ui/widgets/ai/chat/*`
- `src/app/ui/workspace/modal_dialogs/settings.rs`

## Execution Plan for Phase 30
1. Namespace bootstrap
- Vytvorit/ustalit `src/app/ai_core/` se stejnym runtime minimum, ktere je dnes nutne pro kompilaci volajicich vrstev.
- V `src/app/mod.rs` prepnout root export z `pub mod cli;` na `pub mod ai_core;`.
- Nastavit minimalni public surface (jen realne pouzite exporty).

2. Import migration pass (low-risk ordering)
- Nejdriv migrovat sdilene typy a settings call sites (`settings.rs`, `app/types.rs`).
- Pak migrovat workspace state/init (`ui/workspace/state/*`).
- Pak UI napojeni (`ui/terminal/ai_chat/*`, `ui/terminal/right/ai_bar.rs`, `widgets/ai/chat/*`, `modal_dialogs/settings.rs`).
- Nakonec migrovat orchestracni flow (`ui/background.rs`, zbyle call sites).

3. Hard removal pass
- Smazat `src/app/cli/*` az po uspesnem migration pass.
- Odstranit osiřele reference ve `mod.rs` souborech a neplatne re-exporty.

4. Dead-code cleanup pass
- Projit `pub` exporty a odstranit vse, co existovalo jen kvuli puvodnimu `app::cli`.
- Overit, ze nezustaly namespace leaks nebo nepouzite adaptery/aliasy.

5. Evidence pass
- Zaznamenat splneni CLI-01/02/03 s konkretnimi artefakty (grep, build logy, seznam odstraneni).

## Validation Architecture
Validace faze 30 bude dvouvrstva: prubezna (po kazdem baliku zmen) a finalni gate (pred uzavrenim faze).

1. Prubezna validace po kazdem kroku
- Spustit `cargo check` po kazdem migration baliku (types/settings, workspace, UI, background).
- Pri failu okamzity stop-and-fix ve stejnem kroku; nepokracovat do dalsiho baliku s cervenym buildem.

2. Namespace integrity validace
- Spustit grep audit nad `src/`:
  - `rg -n "crate::app::cli|app::cli" src`
  - Ocekavany vysledek: prazdny.
- Spustit export audit:
  - `rg -n "pub mod cli|mod cli" src/app`
  - Ocekavany vysledek: zadny aktivni export/modul puvodni CLI vrstvy.

3. Structural filesystem validace
- Overit, ze `src/app/cli` neexistuje.
- Overit, ze novy namespace `src/app/ai_core` existuje a ma konzistentni `mod.rs` surface.

4. Finalni quality gate
- `cargo check`
- `./check.sh` (povinny gate dle kontextu)
- Opakovany grep audit po finalnim cleanup pass.

5. Requirement-level acceptance
- CLI-01 accepted: `src/app/cli/*` odstranen.
- CLI-02 accepted: build prochazi a grep nenasel zadne `app::cli` importy.
- CLI-03 accepted: bez mrtvych CLI exportu/modulu (potvrzeno export auditem + compile clean stavem).

## Risks and Mitigations
- Riziko: Prilis velky atomicky rez rozbije build a ztezi lokalizaci chyby.
  - Mitigace: migrovat po balicich + prubezny `cargo check` po kazdem kroku.
- Riziko: Scope creep do behavior parity (TERM/SAFE pozadavky) uz ve fazi 30.
  - Mitigace: striktni scope lock na CLI-01/02/03; runtime parity resit az ve fazi 31.
- Riziko: Skryte couplingy v test fixture/state inicializaci.
  - Mitigace: zahrnout workspace state/init do casneho migration pass a hned validovat build.
- Riziko: Zbytky namespace v exportech po fyzickem smazani slozky.
  - Mitigace: explicitni export audit a dead-code cleanup pass pred final gate.

## Planning Recommendations
- Plan rozdelit minimalne na 4 implementacni baliky: bootstrap, import migration, hard removal, cleanup+validation.
- Keep compile-first baseline: neresit v teto fazi UX parity nebo provider redesign.
- Do PLAN.md rovnou vlozit requirement-to-step mapu, aby kazdy krok mel jednoznacny dopad na CLI-01/02/03.
- Do SUMMARY/VERIFICATION artefaktu ulozit konkretni dukazy: prikazy, vysledky grep auditu, build gate stav.

## Ready-to-Plan Checklist
- [x] Scope lock proti phase 30 boundary.
- [x] Pokryti CLI-01, CLI-02, CLI-03.
- [x] Definovana validacni architektura a acceptance pravidla.
- [x] Definovane implementacni kroky, rizika a mitigace.

## RESEARCH COMPLETE
