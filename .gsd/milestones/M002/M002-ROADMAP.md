# M002: Local History

**Vision:** Uživatel může po uložení souboru procházet historické verze v split view s diff zvýrazněním přímo v editoru, navigovat šipkami mezi verzemi a spoléhat na automatický snapshot + cleanup.

## Success Criteria

- Po uložení souboru 3× s různým obsahem se v `.polycredo/history/` vytvoří 3 snapshoty (ověřitelné na FS).
- Pravý klik na tab → "Historie souboru" otevře split view se dvěma panely: aktuální verze vlevo, historická vpravo.
- Šipky (UI tlačítka) přepínají mezi historickými verzemi — diff zvýraznění (přidané/odebrané řádky) se aktualizuje.
- Zavření history view vrátí editor do normálního režimu bez reliktů v UI stavu.
- Soubor s 60 verzemi → po cleanup zůstane max 50 nejnovějších; verze starší 30 dní jsou smazány.
- Binární soubory (obrázky, fonty) nespouští snapshot.
- I/O chyby při snapshotování se propagují do UI toastu (per S-3).

## Key Risks / Unknowns

- **Split view v editoru** — Editor nemá dual-pane koncept. Nový rendering path pro dva read-only panely vedle sebe s potenciální interakcí se stávajícím tab/scroll state.
- **Mrtvý kanál** — `background_io_tx` neexistuje, `background_io_rx` je vždy `None`. Celý pipeline producent→background handler je mrtvý kód, který musí být oživen.
- **Diff výkon** — `similar::TextDiff::from_lines()` nad velkými soubory může být pomalý. Diff se musí cachovat, ne počítat per-frame.

## Proof Strategy

- **Split view** → retire v S02 postavením reálného dual-pane renderu nad live daty ze S01. Proven = dva panely se renderují vedle sebe s diff barvami, navigace šipkami funguje, zavření vrátí normální editor mode.
- **Mrtvý kanál** → retire v S01 propojením senderu do save hooku. Proven = po uložení souboru se snapshot objeví na FS.
- **Diff výkon** → retire v S02 cachováním diff výsledku per history entry. Proven = opakovaný frame render nerekompuuje diff.

## Verification Classes

- Contract verification: `cargo check` + `./check.sh` po každé slice. Unit testy pro snapshot pipeline, cleanup retenci, binární detekci.
- Integration verification: end-to-end tok save → snapshot → context menu → split view → navigace → zavření. Ověřeno v UAT scénářích.
- Operational verification: retence 50 verzí + 30 dní. Cleanup běží na pozadí bez UI freeze. Watcher ignoruje `.polycredo/history/`.
- UAT / human verification: split view layout, diff zvýraznění čitelnost, navigace šipkami UX — vyžaduje vizuální kontrolu v běžícím editoru.

## Milestone Definition of Done

This milestone is complete only when all are true:

- Všechny 3 slice jsou dokončené a verifikované.
- Save hook pokrývá `save()` i `save_path()` — autosave i manuální uložení vytváří snapshoty.
- Context menu na tabu obsahuje "Historie souboru" položku.
- Split view renderuje dva read-only panely s diff zvýrazněním a navigací šipkami.
- Cleanup respektuje retenci (50 verzí, 30 dní) a běží bez blokování UI.
- Binární taby nespouští snapshot.
- I/O chyby se propagují do toastu.
- i18n klíče existují pro všech 5 jazyků (cs, en, sk, de, ru).
- `cargo check` + `./check.sh` prochází.
- Finální UAT scénáře z M002-CONTEXT.md (3 uložení → split view → navigace → diff; 60 verzí → cleanup; zavření split view) prochází.

## Requirement Coverage

> ⚠️ No `.gsd/REQUIREMENTS.md` found — operating in legacy compatibility mode. Active requirements sourced from `PROJECT.md`.

- Covers: **S-3** (I/O chyby propagovat do UI toastu — history save chyby budou toastovány, primární owner S01)
- Partially covers: **S-1** (watcher neztrácí eventy — ověření, že `.polycredo/` filtr funguje, ale samotná S-1 oprava je mimo scope)
- Leaves for later: **V-1**, **V-2**, **K-1**, **N-5**, **S-4**, **V-3** (backlog items mimo scope M002)
- Orphan risks: none

## Slices

- [ ] **S01: Snapshot Pipeline a Tab Context Menu** `risk:medium` `depends:[]`
  > After this: Uložení souboru 3× vytvoří 3 snapshoty v `.polycredo/history/`. Pravý klik na tab ukáže "Historie souboru" — zatím otevře jednoduchý history panel s výpisem verzí a náhledem vybrané verze v jednom panelu. Binární soubory jsou přeskočeny, I/O chyby se zobrazí v toastu. Ověřeno unit testy a manuálně v běžícím editoru.
- [ ] **S02: History Split View s Diff a Navigací** `risk:high` `depends:[S01]`
  > After this: "Historie souboru" otevře plný split view — aktuální verze vlevo, historická vpravo s diff zvýrazněním (zelená/červená). Šipky přepínají mezi verzemi. Diff je cachovaný (ne per-frame). Zavření vrátí normální editor. Ověřeno vizuálně v běžícím editoru.
- [ ] **S03: Cleanup, Edge Cases a Finální Integrace** `risk:low` `depends:[S01,S02]`
  > After this: Cleanup při startu workspace smaže verze nad retenci (50) a starší 30 dní. Stará testovací data v `.polycredo/history/` jsou vyčištěna. Zavření tabu v history mode vyčistí stav. Finální UAT scénáře prochází end-to-end. Kompletní i18n audit pro všech 5 jazyků.

## Boundary Map

### S01 → S02

Produces:
- Funkční `background_io_tx` sender propojený do save logiky — po uložení souboru se `FsChangeResult::LocalHistory(rel_path, content)` odešle do kanálu.
- `LocalHistory::get_snapshot_content(rel_path, entry) -> io::Result<String>` metoda pro načtení obsahu historické verze.
- `LocalHistory::get_history(rel_path) -> Vec<HistoryEntry>` vrací seřazené verze od nejnovější.
- `HistoryViewState` struct v editor modulu — drží stav history panelu (vybraný soubor, vybraná verze, seznam verzí).
- Context menu na tab baru s "Historie souboru" položkou — spouští otevření history view.
- Toast propagace I/O chyb z background snapshot handleru.
- i18n klíče pro context menu a history panel ve všech 5 jazycích.

Consumes:
- nothing (first slice)

### S02 → S03

Produces:
- Split view rendering v editoru: dva read-only ScrollArea panely vedle sebe s resize handle (pattern z `render/markdown.rs`).
- Diff zvýraznění pomocí `similar::TextDiff::from_lines()` s cachovaným výsledkem per history entry.
- Navigační UI (šipky zpět/vpřed) pro přepínání mezi historickými verzemi.
- Zavírací mechanismus — history mode → normální editor mode přepnutí.
- i18n klíče pro split view navigaci.

Consumes:
- `HistoryViewState` struct a context menu trigger z S01.
- `LocalHistory::get_snapshot_content()` a `get_history()` z S01.
- Funkční snapshot pipeline z S01 (reálná data pro zobrazení).

### S03 (final)

Produces:
- `cleanup()` s `max_age` parametrem (30 dní) volaný při startu workspace v background threadu.
- Vyčištění `.polycredo/history/` od testovacích reliktu.
- Edge case handling: zavření tabu v history mode, history mode na posledním tabu.
- Finální i18n audit a případné chybějící překlady.
- Verifikace watcher filtru pro `.polycredo/` adresář.

Consumes:
- Kompletní split view z S02.
- Snapshot pipeline z S01.
- `LocalHistory::cleanup()` z existujícího kódu (rozšířená o max_age v S01).
