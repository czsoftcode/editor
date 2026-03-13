---
id: T02
parent: S01
milestone: M002
provides:
  - Context menu na tab bar (pravý klik) s "Historie souboru" a "Zavřít tab"
  - HistoryViewState struct pro stav history panelu
  - render_history_panel() funkce s horizontálním split layoutem (seznam verzí + náhled textu)
  - i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru)
  - TabBarAction::ShowHistory(usize) varianta
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/widgets/tab_bar.rs
  - src/app/ui/editor/render/tabs.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Timestamp formátování bez chrono dependency — vlastní days_to_date() algoritmus (Howard Hinnant) v UTC, vyhnuli jsme se přidání nové dependency kvůli jedné funkci
  - History panel renderován jako overlay v CentralPanel po editor renderingu, ne jako samostatný egui::Window — zachovává layout konzistenci s editorem
  - Context menu na selectable_label response (r.context_menu) — egui nativní API pro right-click menu
patterns_established:
  - Tab context menu pattern přes r.context_menu() s i18n texty a guardem na is_binary
  - HistoryViewState jako Option<> pole v WorkspaceState — None = panel nezobrazen, Some = aktivní
  - render_history_panel() přijímá split reference (&mut HistoryViewState, &LocalHistory) pro partial borrow compatibility
observability_surfaces:
  - Toast "Žádné historické verze" při pokusu o otevření historie pro soubor bez snapshotu
  - Chybová hláška v preview panelu při selhání čtení snapshot souboru z disku
duration: 20min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Tab context menu, HistoryViewState a history panel UI s i18n

**Přidáno context menu na tab bar (pravý klik → "Historie souboru" / "Zavřít tab"), HistoryViewState struct a funkční history panel s horizontálním split layoutem (seznam verzí + monospace náhled textu) ve všech 5 jazycích.**

## What Happened

1. Rozšířen `TabBarAction` enum o variantu `ShowHistory(usize)` v `tab_bar.rs`.
2. Přidáno context menu na `selectable_label` response v tab bar renderingu (`tabs.rs`) — "Historie souboru" (jen pro ne-binární taby) a "Zavřít tab", obě s i18n klíči. Signatura `tab_bar()` rozšířena o `i18n` parametr.
3. Vytvořen nový modul `src/app/ui/workspace/history/mod.rs` s `HistoryViewState` struct a `render_history_panel()` funkcí. Panel má horizontální split: levý (30%) se ScrollArea seznamem verzí (formátovaný timestamp), pravý (70%) s monospace read-only náhledem textu vybrané verze. Horní bar s nadpisem a zavíracím tlačítkem.
4. Přidáno pole `history_view: Option<HistoryViewState>` do `WorkspaceState`, inicializováno na `None` v `init_workspace` a obou testovacích konstruktorech v `app/mod.rs`.
5. Handling `ShowHistory` akce ve workspace: načte tab path, vypočítá relativní cestu, zavolá `get_history()`, pokud prázdný → toast, jinak vytvoří `HistoryViewState`.
6. History panel renderován v CentralPanel jako overlay po editor renderingu.
7. Zavírací tlačítko nastaví `ws.history_view = None`.
8. i18n klíče přidány do všech 5 locale souborů: `tab-context-history`, `tab-context-close`, `history-panel-title`, `history-panel-no-versions`, `history-panel-close`, `history-panel-version-label`.
9. Doplněn match pro `ShowHistory` v terminal/right `apply_tab_action` (no-op pro AI terminal taby).

## Verification

- `cargo check` — kompilace bez chyb ✅
- `cargo fmt --all` — formátování v pořádku ✅
- `cargo clippy` — žádné warningy ✅
- `./check.sh` — 128 unit testů prochází ✅ (1 pre-existující integrační test `phase35_delete_foundation_scope_guard` failuje kvůli chybějícímu `.planning/` souboru — nesouvisí s S01)
- `cargo test -- local_history` — 6 testů snapshot pipeline prochází ✅

### Slice-level verification status (T02 je poslední task v S01):
- `cargo test -- local_history` — ✅ prochází (6 testů)
- `cargo check` — ✅ prochází
- `./check.sh` — ✅ prochází (128/128 unit testů; 1 nesouvisející integrační test fail)
- Manuální ověření: vyžaduje spuštění GUI — nelze automatizovat v agent kontextu

## Diagnostics

- Pravý klik na tab → vizuální ověření context menu
- Toast "Žádné historické verze" při prázdné historii
- Chybová hláška v preview panelu (`Chyba čtení: ...`) při I/O problému
- `ws.history_view.is_some()` — runtime stav panelu

## Deviations

- Timestamp formátování: místo chrono dependency implementován vlastní UTC formátor (days_to_date algoritmus). Plán toto explicitně nespecifikoval, ale přidání chrono pro jednu funkci by bylo zbytečné.
- History panel je renderován po editoru v CentralPanel (overlay pattern), ne místo editoru — obě varianty byly zmíněny v plánu ("místo normálního editor renderingu, nebo jako overlay panel"), zvolen overlay.

## Known Issues

- Timestamp je v UTC, ne v lokálním čase — pro přidání lokálního času by byla potřeba chrono nebo libc timezone. Pro porovnávání verzí je UTC dostatečné.
- Pre-existující integrační test `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` failuje kvůli chybějícímu souboru `.planning/phases/35-trash-foundation-async-safety/35-03-PLAN.md` — nesouvisí s S01.

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — nový soubor: HistoryViewState struct, render_history_panel(), format_timestamp(), days_to_date()
- `src/app/ui/widgets/tab_bar.rs` — přidána varianta ShowHistory(usize) do TabBarAction enum
- `src/app/ui/editor/render/tabs.rs` — context menu na tab selectable_label, rozšířená signatura tab_bar() o i18n
- `src/app/ui/editor/ui.rs` — předání i18n do tab_bar(), propagace ShowHistory akce
- `src/app/ui/workspace/state/mod.rs` — pole history_view: Option<HistoryViewState> v WorkspaceState
- `src/app/ui/workspace/state/init.rs` — inicializace history_view: None
- `src/app/ui/workspace/mod.rs` — mod history, handling ShowHistory akce, renderování history panelu v CentralPanel
- `src/app/ui/terminal/right/mod.rs` — match pro ShowHistory v apply_tab_action (no-op)
- `src/app/mod.rs` — history_view: None v obou testovacích WorkspaceState konstruktorech
- `locales/cs/ui.ftl` — 6 nových i18n klíčů (cs překlady)
- `locales/en/ui.ftl` — 6 nových i18n klíčů (en překlady)
- `locales/sk/ui.ftl` — 6 nových i18n klíčů (sk překlady)
- `locales/de/ui.ftl` — 6 nových i18n klíčů (de překlady)
- `locales/ru/ui.ftl` — 6 nových i18n klíčů (ru překlady)
