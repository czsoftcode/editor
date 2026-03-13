---
estimated_steps: 8
estimated_files: 11
---

# T02: Tab context menu, HistoryViewState a history panel UI s i18n

**Slice:** S01 — Snapshot Pipeline a Tab Context Menu
**Milestone:** M002

## Description

Uživatel potřebuje způsob jak otevřít historii souboru. Tento task přidá context menu na tab bar (pravý klik → "Historie souboru" / "Zavřít tab"), vytvoří `HistoryViewState` struct pro stav history panelu, implementuje jednoduchý history panel (seznam verzí + náhled textu vybrané verze) a pokryje i18n pro všech 5 jazyků.

## Steps

1. **Rozšířit `TabBarAction`:** V `src/app/ui/widgets/tab_bar.rs` přidat variantu `ShowHistory(usize)` do `TabBarAction` enum.
2. **Context menu v tab baru:** V `src/app/ui/editor/render/tabs.rs` — na `selectable_label` response (`r`) přidat `.context_menu(|ui| { ... })`. Uvnitř: položka "Historie souboru" (i18n klíč `tab-context-history`) — emituje `TabBarAction::ShowHistory(idx)`, ale jen pokud tab není binární (jinak položku nezobrazit). Položka "Zavřít tab" (i18n klíč `tab-context-close`) — emituje `TabBarAction::Close(idx)`. Pro přístup k i18n je potřeba předat `i18n` a `tabs` referenci do `tab_bar()` metody (rozšířit signaturu).
3. **Vytvořit `HistoryViewState`:** V novém souboru `src/app/ui/workspace/history/mod.rs` definovat struct `HistoryViewState` s poli: `file_path: PathBuf` (absolutní), `relative_path: PathBuf`, `entries: Vec<crate::app::local_history::HistoryEntry>`, `selected_index: Option<usize>`, `preview_content: Option<String>`, `scroll_to_selected: bool`. Přidat `mod history;` do `src/app/ui/workspace/mod.rs` (nebo parent). Přidat `pub use` pro re-export.
4. **Přidat `history_view` do `WorkspaceState`:** V `src/app/ui/workspace/state/mod.rs` přidat pole `pub history_view: Option<HistoryViewState>`. Inicializovat na `None` v `init_workspace` a obou konstrukčních bodech v `app/mod.rs`.
5. **Handle `ShowHistory` action ve workspace:** V `src/app/ui/workspace/mod.rs` — v matchi `editor_res.tab_action` přidat handling pro `TabBarAction::ShowHistory(idx)`: načíst tab path, vypočítat relativní cestu vůči `ws.root_path`, zavolat `ws.local_history.get_history(&relative_path)`, vytvořit `HistoryViewState` a přiřadit do `ws.history_view`. Pokud je seznam prázdný, zobrazit toast "Žádné verze" místo otevření panelu.
6. **Renderovat history panel:** V `src/app/ui/workspace/history/mod.rs` přidat `pub fn render_history_panel(ws: &mut WorkspaceState, ui: &mut egui::Ui, i18n: &I18n)`. Layout: horizontální split — levý ScrollArea (30% šířky) se seznamem verzí (timestamp formátovaný jako datum+čas, selectable_label), pravý ScrollArea (70%) s monospace read-only náhledem textu vybrané verze. Horní bar: nadpis "Historie: {filename}" + zavírací tlačítko. Klik na verzi → `get_snapshot_content()` → update `preview_content`. Volat tento render z workspace `CentralPanel` pokud `ws.history_view.is_some()` — místo normálního editor renderingu pro daný soubor, nebo jako overlay panel.
7. **Zavírání history panelu:** Zavírací tlačítko (× nebo i18n `history-panel-close`) nastaví `ws.history_view = None` a vrátí normální editor rendering.
8. **i18n klíče ve všech 5 jazycích:** Přidat do `locales/*/ui.ftl` klíče: `tab-context-history`, `tab-context-close`, `history-panel-title` (s parametrem `name`), `history-panel-no-versions`, `history-panel-close`, `history-panel-version-label` (s parametrem `date`). Překlady: cs, en, sk, de, ru.

## Must-Haves

- [ ] Pravý klik na tab zobrazí context menu s "Historie souboru" (ne-binární tab) a "Zavřít tab".
- [ ] Binární tab nemá "Historie souboru" v context menu.
- [ ] "Historie souboru" otevře history panel s výpisem verzí a náhledem vybrané verze.
- [ ] `HistoryViewState` struct nese kompletní stav panelu.
- [ ] Klik na verzi v seznamu zobrazí náhled jejího textu.
- [ ] Zavírací tlačítko vrátí normální editor rendering.
- [ ] i18n klíče existují ve všech 5 jazycích.
- [ ] `cargo check` + `./check.sh` prochází.

## Verification

- `cargo check` — kompilace bez chyb.
- `./check.sh` — celkový projekt check.
- Manuální test v běžícím editoru: pravý klik na ne-binární tab → context menu s "Historie souboru" a "Zavřít tab". Klik na "Historie souboru" → history panel. Klik na verzi → náhled. Zavírací tlačítko → zpět do editoru. Pravý klik na binární tab → pouze "Zavřít tab".

## Inputs

- `src/app/local_history.rs` — `LocalHistory::get_history()`, `get_snapshot_content()`, `HistoryEntry` struct (z T01).
- `src/app/ui/workspace/state/mod.rs` — `WorkspaceState` s `background_io_tx` (z T01).
- `src/app/ui/editor/render/tabs.rs` — stávající tab bar rendering.
- `src/app/ui/widgets/tab_bar.rs` — `TabBarAction` enum.
- `src/app/ui/workspace/mod.rs` — handling tab akcí ve workspace.
- Stávající i18n `.ftl` soubory v 5 locale adresářích.

## Expected Output

- `src/app/ui/widgets/tab_bar.rs` — `TabBarAction::ShowHistory(usize)` varianta.
- `src/app/ui/editor/render/tabs.rs` — context menu na tab response s i18n texty.
- `src/app/ui/workspace/history/mod.rs` — nový soubor s `HistoryViewState` struct a `render_history_panel()`.
- `src/app/ui/workspace/state/mod.rs` — pole `history_view: Option<HistoryViewState>`.
- `src/app/ui/workspace/mod.rs` — handling `ShowHistory` akce + volání `render_history_panel`.
- `src/app/ui/editor/mod.rs` — případné úpravy Editor struct/UI flow pro koexistenci s history panelem.
- `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl` — nové i18n klíče.
