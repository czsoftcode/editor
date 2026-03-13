# M002: Local History — Context

**Gathered:** 2026-03-13
**Status:** Queued — pending auto-mode execution.

## Project Description

Zprovoznit local history nad projektovým adresářem: smazat starý obsah `.polycredo/history/`, vytvářet snapshoty při uložení souboru, a dodat UI pro náhled aktuální a minulé verze ve dvou editorových panelech vedle sebe s navigací šipkami po historii verzí.

## Why This Milestone

Backend `LocalHistory` existuje (`src/app/local_history.rs`) se základními operacemi `take_snapshot`, `get_history`, `cleanup`, ale:
- Chybí producent — `FsChangeResult::LocalHistory` kanál existuje, ale nikdo do něj neposílá.
- UI pro náhled historie neexistuje (`src/app/ui/workspace/history/` je prázdný adresář).
- Stávající data v `.polycredo/history/` jsou testovací relikty, ne produkční snapshoty.
- Existující `diff_view.rs` umí side-by-side diff v modalu, ale požadovaný UX je inline split v editoru s navigací mezi verzemi.

Toto uzavírá feature gap: editor má safe delete (v1.3.1), ale nemá history browsing pro soubory.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Pravým klikem na tab otevřít "Historie souboru" a vidět dvoupanelový split view s aktuální a minulou verzí
- Šipkami (nebo UI tlačítky) navigovat zpět/vpřed v historii verzí daného souboru
- Vidět diff zvýraznění (přidané/odebrané řádky) mezi aktuální a vybranou historickou verzí
- Spoléhat na to, že každé uložení souboru automaticky vytvoří snapshot (pokud se obsah změnil)

### Entry point / environment

- Entry point: context menu na editorovém tabu → "Historie souboru"
- Environment: desktop editor (eframe/egui), local-first, single-process multi-window
- Live dependencies involved: none — vše je lokální filesystem

## Completion Class

- Contract complete means: `take_snapshot` se volá při každém uložení souboru; `get_history` vrací seřazené verze; cleanup respektuje retenci 50 verzí + 30 dní; split view renderuje dvě verze vedle sebe s diff zvýrazněním.
- Integration complete means: celý tok funguje end-to-end — uložení → snapshot → context menu → split view → navigace šipkami → zavření split view zpět na normální editor.
- Operational complete means: po 100+ uloženích se history nerozroste nad retenci, cleanup proběhne bez UI freeze, watcher nereaguje na `.polycredo/history/` změny.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Uložení souboru 3× s různým obsahem → context menu "Historie" → split view ukazuje aktuální verzi vlevo a předchozí vpravo → šipky přepnou na starší verzi → diff zvýraznění je správné.
- Soubor s 60 verzemi → po cleanup zůstane max 50 nejnovějších → verze starší 30 dní jsou smazány.
- Split view se zavře a editor se vrátí do normálního režimu bez reliktu v UI stavu.

## Risks and Unknowns

- **Split view v egui editoru:** Editor aktuálně nemá koncept split/dual pane. Bude potřeba nový layout mode v editor modulu, kde se renderují dva read-only panely vedle sebe. Riziko: neznámá složitost integrace se stávajícím tab/file systémem.
- **Synchronizované scrollování:** Dva panely s různě dlouhým textem potřebují buď sync scroll, nebo nezávislý scroll. Obojí má UX kompromisy.
- **Výkon při velkých souborech:** Diff (`similar` crate) nad velkými soubory může trvat — potřeba buď lazy diff, nebo background výpočet.
- **Binární soubory:** Snapshot se musí přeskočit pro binární soubory (obrázky, fonty). Detekce binarity musí být spolehlivá.

## Existing Codebase / Prior Art

- `src/app/local_history.rs` — backend service, funkční ale bez producenta. Obsahuje `take_snapshot`, `get_history`, `cleanup`. Používá xxhash pro deduplikaci, timestamp pro řazení.
- `src/app/ui/background.rs` — přijímá `FsChangeResult::LocalHistory`, ale nikdo tento variant neposílá. Tady se musí napojit producent (save hook).
- `src/app/ui/workspace/state/types.rs` — definuje `FsChangeResult` enum s variantou `LocalHistory(PathBuf, String)`.
- `src/app/ui/editor/diff_view.rs` — existující side-by-side diff modal (AI diff). Obsahuje `similar`-based diff rendering, barevné zvýraznění. Lze znovupoužít diff logiku, ne UI kontejner.
- `src/app/ui/editor/files.rs` — file save logika, sem patří save hook pro snapshot.
- `src/app/ui/editor/tabs.rs` — tab rendering, sem patří context menu entry.
- `.polycredo/history/index.json` — mapování hash→cesta. Stávající data jsou testovací relikty, budou smazána.
- `src/app/ui/workspace/history/` — prázdný adresář, připravený pro history UI modul.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- S-3: Neignorovat I/O chyby, propagovat je do UI toastu — history save chyby musí být viditelné.
- S-1: Upravit `FileWatcher::try_recv()`, aby neztrácal eventy — watcher nesmí reagovat na `.polycredo/history/` změny (už je filtrováno, ale ověřit).

## Scope

### In Scope

- Napojení snapshot producenta na save souboru (Ctrl+S / auto-save)
- Smazání stávajícího obsahu `.polycredo/history/` a čistý start
- Retention policy: max 50 verzí na soubor + max 30 dní stáří
- Detekce binárních souborů — přeskočit snapshot pro non-text
- Context menu na tabu: "Historie souboru"
- Split view v editoru: dva read-only panely vedle sebe (aktuální + historická verze)
- Navigace šipkami mezi historickými verzemi
- Diff zvýraznění (přidané/odebrané řádky) mezi verzemi
- i18n pro všechny UI texty (cs, en, de, sk, ru)
- Automatický cleanup při startu nebo periodicky

### Out of Scope / Non-Goals

- OS-level file history nebo integrace s git history
- Editace historické verze (jen read-only náhled)
- Restore historické verze jako aktuální (pro v1 stačí view-only, restore je follow-up)
- Timeline panel nebo samostatné okno pro historii
- History pro binární soubory

## Technical Constraints

- Neblokovat UI vlákno při snapshot I/O — použít existující background task pattern
- Respektovat existující single-process multi-window architekturu
- `cargo check` + `./check.sh` musí projít po každé fázi
- Žádné nové runtime závislosti (similar crate už je v projektu)
- History adresář `.polycredo/history/` musí zůstat ignorovaný watcherem

## Integration Points

- `src/app/ui/editor/files.rs` — save hook: po uložení odeslat `FsChangeResult::LocalHistory` do kanálu
- `src/app/ui/background.rs` — již přijímá `FsChangeResult::LocalHistory` a volá `take_snapshot`
- `src/app/ui/editor/tabs.rs` — nová položka v context menu tabu
- `src/app/ui/editor/mod.rs` — nový stav pro "history split view" mode
- `src/app/local_history.rs` — rozšířit o `get_snapshot_content(file, entry)` pro načtení obsahu historické verze
- Watcher: ověřit, že `.polycredo/history/` je filtrováno z watcher eventů

## Open Questions

- Synchronizovaný vs nezávislý scroll dvou panelů — pravděpodobně nezávislý je jednodušší a praktičtější, ale ověřit při implementaci.
- Umístění navigačních šipek — toolbar nad split view, nebo inline vedle tab baru. Rozhodnout při planning.
