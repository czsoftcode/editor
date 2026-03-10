# Phase 25 Research: Unsaved Close Guard

## Objective
Naplánovat implementaci ochrany proti ztrátě neuložených změn pro požadavky `GUARD-01`, `GUARD-02`, `GUARD-03`, `GUARD-04` tak, aby:
- zavření dirty tabu vždy spustilo rozhodovací flow,
- zavření aplikace/projektu běželo přes stejný guard flow,
- větve `Save` / `Discard` / `Cancel` byly konzistentní napříč všemi spouštěči,
- save fail během zavírání nikdy nevedl k tichému zavření.

Vstupní rozhodnutí z `25-CONTEXT.md` jsou závazná: sekvenční zpracování více dirty tabů, `Cancel` jako safe default, `Esc`/`X` => `Cancel`, save fail = inline + toast + zůstat ve flow.

## Current Baseline (What Already Exists)
- Zavírání tabu dnes neřeší `modified` guard:
  - `Ctrl+W` v `src/app/ui/workspace/mod.rs` volá `ws.editor.clear()`.
  - menu `Close Tab` v `src/app/ui/workspace/menubar/mod.rs` také volá `ws.editor.clear()`.
  - klik na `×` v tab baru přes `TabBarAction::Close(idx)` volá `self.close_tab(idx)` v `src/app/ui/editor/ui.rs`.
- `Editor::close_tab` (`src/app/ui/editor/tabs.rs`) zavře tab bez podmínky.
- Dirty stav existuje (`Tab.modified`) a save API už je připravené:
  - `save(...)` pro aktivní tab,
  - `save_path(...)` pro konkrétní tab i mimo active index (`src/app/ui/editor/files.rs`).
- Save chyby už vrací lokalizovaný string (`error-file-save`) a toasty existují (`Toast::error`).
- Close flow pro projekt/aplikaci už má modal pattern:
  - root close request v `src/app/mod.rs` (`show_close_project_confirm`, `show_quit_confirm`),
  - secondary viewport close používá `close_requested` flag + modal v témže viewportu.
- Modal framework (`src/app/ui/widgets/modal.rs`) už řeší backdrop, `X`, footer tlačítka a lze ho znovu použít.

## Requirement Mapping (GUARD-01..04)
- `GUARD-01` (close dirty tab):
  - Nutná intercept vrstva mezi triggerem close a `editor.close_tab(...)`.
  - Guard musí pokrýt všechny vstupy: `Ctrl+W`, menu, tab `×` (a prakticky i middle-click, protože dnes také zavírá tab).
- `GUARD-02` (close app/project):
  - Root close request, `QuitAll`, `show_close_project_confirm_dialog`, secondary viewport close request musí přejít na nový unsaved guard flow.
  - U více oken je potřeba sběr dirty tabů napříč workspaces.
- `GUARD-03` (`Save`/`Discard`/`Cancel`):
  - Guard dialog musí být jednotný pro tab-close i app/project-close flow.
  - `Cancel` ruší celou close operaci, ne jen aktuální krok.
- `GUARD-04` (save fail během close):
  - Po selhání save nesmí proběhnout close; uživatel musí zůstat ve stejném kroku.
  - Error z `save_path` použít 2x: inline v dialogu + `Toast::error` (s existující deduplikací).

## Recommended Architecture (Minimal Patch, No Big Refactor)
1. Zavést explicitní close guard state do `WorkspaceState`.
- Doplnit strukturu typu `PendingCloseFlow` s módem:
  - `SingleTab { index }` pro GUARD-01,
  - `WorkspaceClose` pro GUARD-02 (v rámci jednoho workspace; root orchestrátor řeší více oken).
- Stav drží frontu `Vec<PathBuf>` (ne indexy), protože indexy tabů se během flow mění.
- Stav drží i `current_path`, `last_error_inline`, `origin_trigger` (diagnostika/telemetrie v budoucnu).

2. Vytvořit jednotný guard orchestrátor na workspace úrovni.
- Nový helper ve `workspace` vrstvě:
  - sestaví frontu dirty tabů (aktivní první, pak stabilní pořadí),
  - otevře guard modal,
  - na `Save`: `editor.save_path(current_path, ...)`,
  - na `Discard`: `editor.close_tabs_for_path(current_path)` nebo cílené zavření,
  - na `Cancel`: ukončí celý flow bez close side-effect.
- Tento helper bude jediná cesta pro všechny close triggery uvnitř workspace.

3. Root orchestrace pro app quit / close project přes fázovaný postup.
- V `EditorApp` přidat fázi "pending global close" namísto okamžitého potvrzení `close_project/quit`.
- Postup:
  - zjistit, zda některý workspace má dirty taby,
  - pokud ne: pokračovat existujícím close flow,
  - pokud ano: spustit unsaved flow v relevantních workspaces sekvenčně,
  - teprve po úspěšném dokončení všech front provést finální close (quit/close project/close secondary).
- Tím zůstane zachovaná architektura single-process multi-window bez velké restrukturalizace.

4. UI guard dialog jako samostatný reusable modul.
- Přidat `show_unsaved_close_guard_dialog(...)` do `src/app/ui/dialogs/` nebo `workspace/modal_dialogs/`.
- Dialog musí zobrazit minimálně: filename + full path + state.
- Footer tlačítka v pořadí konzistentním s projektem, ale focus/default na `Cancel`.
- `Esc` a zavření přes `X` => `Cancel`.

## Integration Points (Concrete Hook List)
- `src/app/ui/workspace/mod.rs`
  - nahradit přímé `ws.editor.clear()` při `Ctrl+W` guard-aware voláním.
- `src/app/ui/workspace/menubar/mod.rs`
  - `actions.close_file` nesmí volat `editor.clear()` napřímo.
- `src/app/ui/editor/ui.rs`
  - při `TabBarAction::Close(idx)` volat guard request místo `close_tab(idx)`.
- `src/app/mod.rs`
  - root close request, `QuitAll`, close project flow, secondary close flow: před existující confirm vrstvu vložit unsaved guard fázi.
- `src/app/ui/workspace/state/mod.rs` + `state/init.rs`
  - přidat a inicializovat guard state.
- `locales/*/*.ftl`
  - nové klíče pro unsaved guard title/message/actions/inline save error text.

## Data Model Notes (Planning-Critical)
- Fronta v guard flow musí používat `PathBuf` místo indexu tabu:
  - index po zavření/skoku mění hodnotu,
  - `save_path` a lookup už pracují nad path.
- Guard flow pro `WorkspaceClose` potřebuje snapshot dirty tabů při startu.
  - Během flow může uživatel otevřít nové taby; ty nepatří do právě běžící close fronty (determinismus).
- Pro více workspace při app quit doporučený root-level snapshot "workspace -> queue dirty paths".

## UX/Behavior Decisions Already Locked (Do Not Re-open in Plan)
- Více dirty tabů: sekvenčně, bez `Save all` / `Discard all`.
- Pořadí: aktivní tab první, zbytek deterministicky.
- `Cancel` je default/focus akce.
- `Esc` i `X` znamenají `Cancel`.
- Save fail: inline + toast, zůstat na stejném tabu, umožnit retry/save i `Discard`/`Cancel`.

## Key Risks and Failure Modes
- Re-entrancy: pokud se guard otevře z více triggerů v jednom frame, může dojít ke dvojímu close requestu.
  - Mitigace: `if pending_close_flow.is_some() { ignore new close trigger }`.
- Nekonzistence mezi triggery: část kódu může stále volat `editor.close_tab` napřímo.
  - Mitigace: centralizovat close API do guard-aware helperu, přímé volání používat jen interně uvnitř guardu.
- Save fail toast spam při retry.
  - Mitigace: použít existující `should_emit_save_error_toast` (už řešeno v projektu).
- Multi-window deadlock/lock contention při současném čtení stavů workspace.
  - Mitigace: snapshot pouze přes krátké locky, žádné modal/render volání v držení mutexu.

## Validation Architecture (Nyquist-Ready)
- Requirement coverage matrix:
  - `GUARD-01`: dirty tab + `Ctrl+W`/menu/`×` -> guard dialog se otevře, tab se nezavře bez rozhodnutí.
  - `GUARD-02`: dirty taby + root window close / Quit / Close Project / secondary close -> guard flow proběhne.
  - `GUARD-03`: ověřit `Save`, `Discard`, `Cancel` větve a že `Cancel` ruší celou operaci.
  - `GUARD-04`: vyvolat save fail (read-only nebo invalid path) -> inline error + toast + close se nedokončí.
- Test layers:
  - Unit: pořadí fronty dirty tabů (active-first + deterministic), reducer rozhodnutí (`Save`/`Discard`/`Cancel`).
  - Integration-lite: trigger routing (`Ctrl+W`, menu action, tab close action) na guard request.
  - Manual UAT: app quit/close project přes více otevřených oken.
- Povinné runtime ověření po implementaci: `cargo check` a `./check.sh`.

## Suggested Plan Shape
1. Plan A: `WorkspaceState` + guard state model + queue builder + jednotné close request API.
2. Plan B: napojení všech tab-close triggerů (`Ctrl+W`, menu, `×`, middle-click) na guard API.
3. Plan C: guard modal UI + save fail inline handling + i18n klíče.
4. Plan D: integrace do `EditorApp` close flow (root/quit/close-project/secondary) + sekvenční více-workspace orchestrace.
5. Plan E: testy + UAT scénáře + final check (`cargo check`, `./check.sh`).

## Open Questions for Planning (Need Explicit Decision in PLAN)
- Kde přesně držet global close orchestrator state v `EditorApp` (samostatný enum vs více bool flagů).
- Jak přesně zavřít konkrétní tab při `Discard` v single-tab flow:
  - by-index snapshot při startu,
  - nebo by-path lookup při každém kroku (doporučeno by-path).
- Má tab-close guard přes middle-click sdílet 100% stejné chování jako `Ctrl+W`/menu/`×`? (technicky dává smysl ANO).
- Jaký přesný text pro inline save fail v dialogu (copy je v kompetenci implementace, ale plán musí rezervovat nové i18n keys).

## Planning Checklist (What You Need To Know)
- Existuje hotový save pipeline (`save_path`) i toast/error infrastruktura; není potřeba nový save subsystém.
- Největší práce je orchestrace flow a sjednocení triggerů, ne samotné ukládání.
- Multi-window close je už dnes speciální (root + deferred viewporty), takže GUARD-02 musí počítat s koordinací mezi workspaces.
- Je nutné explicitně zabránit scope creep (`Save all`/`Discard all` je mimo tuto fázi).
- Implementace má být minimální patch: přidat guard state + guard dialog + přesměrovat existující close body.

## RESEARCH COMPLETE
Výzkum identifikoval přesné integrační body pro `GUARD-01..04`, doporučený minimální architektonický směr (workspace guard + root orchestrátor), hlavní rizika (re-entrancy, multi-window koordinace, toast spam) a konkrétní tvar plánu v 5 krocích, aby šla fáze 25 naplánovat bez dalších produktových otázek mimo vypsané otevřené body.
