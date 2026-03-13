# S02: Obnovení historické verze s potvrzením a i18n — Research

**Date:** 2026-03-13

## Summary

Scope S02 je relativně přímočarý — přidává tlačítko "Obnovit" do toolbaru history split view, potvrzovací dialog přes existující `show_modal` pattern (`egui::Modal`), restore logiku (zápis historického obsahu do tab bufferu + synchronní snapshot + refresh entries), a i18n klíče ve všech 5 jazycích.

Hlavní architektonický constraint: `render_history_split_view()` dostává `&LocalHistory` (immutable ref), ale `take_snapshot()` vyžaduje `&mut self`. Restore logiku proto nelze provést uvnitř render funkce — musí se signalizovat přes rozšířený `HistorySplitResult` a provést v `workspace/mod.rs`, kde máme `&mut ws.local_history`. Confirm dialog (immediate-mode) zůstává uvnitř render funkce.

Riziko je nízké — žádné nové UI patterny, existující `show_modal` je osvědčený (5 použití v file_tree), `take_snapshot` + `get_history` jsou synchronní a spolehlivé.

## Recommendation

Implementovat jako jeden task s těmito kroky:

1. **Datový model:** Přidat `show_restore_confirm: bool` do `HistoryViewState`. Rozšířit `HistorySplitResult` o `restore_confirmed: bool`.
2. **Toolbar:** Přidat tlačítko "Obnovit" do right-to-left layout toolbaru (vedle navigačních šipek). Enabled jen když `selected_index.is_some()`.
3. **Confirm dialog:** Při kliknutí na "Obnovit" → `show_restore_confirm = true`. Volat `show_modal()` v render funkci. Confirmed → `restore_confirmed = true` + `show_restore_confirm = false`. Cancelled → `show_restore_confirm = false`.
4. **Restore logika (workspace/mod.rs):** Když `hv_result.restore_confirmed` → načíst obsah historické verze (`get_snapshot_content`), zapsat do `tab.content`, `take_snapshot()` (append), `get_history()` pro refresh, update `HistoryViewState` (entries, selected_index, invalidace diff cache).
5. **i18n:** Přidat klíče do `locales/XX/ui.ftl` pro všech 5 jazyků.
6. **Verifikace:** `cargo check` + `cargo test` + `./check.sh`.

Použít `show_modal` pattern (ne `StandardModal`) — je jednodušší a odpovídá confirm dialog potřebě (malý dialog, ok/cancel, žádný custom form).

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Confirm dialog | `show_modal()` v `widgets/modal.rs` | Osvědčený pattern (5 use sites), `egui::Modal` backdrop, ok/cancel/enter/escape handling |
| Snapshot creation | `LocalHistory::take_snapshot()` | Existující, testovaný, handles dedup (stejný hash → skip) |
| History refresh | `LocalHistory::get_history()` | Existující, sorted newest-first |
| Snapshot content read | `LocalHistory::get_snapshot_content()` | Existující, path construction z entry |

## Existing Code and Patterns

- `src/app/ui/widgets/modal.rs` — `show_modal()`: immediate-mode confirm dialog s `egui::Modal`. Parametry: `ctx, id, title, ok_label, cancel_label, content_fn`. Vrací `ModalResult<T>` (Confirmed/Cancelled/Pending). Content closure vrací `Option<T>` — `Some` = confirm button enabled, `None` = disabled.
- `src/app/ui/file_tree/dialogs.rs:195-240` — Delete confirm dialog jako vzor: state flag `delete_confirm: Option<PathBuf>` řídí zobrazení, `show_modal` se volá každý frame když flag je Some, `Confirmed` → akce + clear, `Cancelled` → clear.
- `src/app/ui/workspace/history/mod.rs:270-350` — Toolbar rendering: `ui.horizontal(|ui| { heading, space, version_info, right_to_left: { close, space, →, ← } })`. Tlačítko "Obnovit" se přidá do right-to-left bloku před navigační šipky (vizuálně vlevo od nich).
- `src/app/ui/workspace/mod.rs:760-800` — Post-render handling: `hv_result.close` → `ws.history_view = None`, `hv_result.content_changed` → tab sync. Restore handling půjde sem.
- `src/app/local_history.rs:67-118` — `take_snapshot(&mut self, relative_file_path, content) -> Result<Option<PathBuf>>`. Deduplikuje přes hash. Synchronní filesystem I/O.
- `src/app/local_history.rs:120-150` — `get_history(&self, relative_file_path) -> Vec<HistoryEntry>`. Seřazeno nejnovější první.
- `src/app/local_history.rs:152-165` — `get_snapshot_content(&self, relative_file_path, entry) -> io::Result<String>`.

## Constraints

- **`render_history_split_view` dostává `&LocalHistory` (immutable)** — nemůže volat `take_snapshot(&mut self)`. Restore musí probíhat ve `workspace/mod.rs` kde je `&mut ws.local_history`.
- **`show_modal` potřebuje `ui.ctx()`** — k dispozici v render funkci přes `ui.ctx()`.
- **Borrow checker v workspace/mod.rs** — S01 vyřešila extrakcí tab metadata do lokálních proměnných a disjoint borrows. Restore flow potřebuje: `&mut ws.local_history`, `&ws.history_view` (pro entries/relative_path), `&mut ws.editor.tabs` (pro tab.content). To jsou tři různé fieldy WorkspaceState — disjoint borrows by měly fungovat, ale nutné ověřit.
- **Immediate mode UI** — confirm dialog se renderuje každý frame když je aktivní. State flag `show_restore_confirm` řídí, kdy se volá `show_modal`.
- **i18n klíče musí existovat ve všech 5 jazycích** — cs, en, sk, de, ru. Soubory: `locales/XX/ui.ftl`.
- **`take_snapshot` deduplikuje** — pokud obsah historické verze je stejný jako aktuální `tab.content`, snapshot se neprovede (vrátí `Ok(None)`). V restore flow to závisí na pořadí: nejdřív zapsat historický obsah do tab, pak snapshot. Ale — uživatel mohl editovat v levém panelu, takže `tab.content` se liší od historické verze. Po zápisu historického obsahu a snapshotu bude snapshot deduplikován jen pokud se shoduje s posledním existujícím snapshotem (unlikely po editaci).

## Common Pitfalls

- **Borrow conflict na workspace/mod.rs** — Restore flow potřebuje současný přístup k `ws.history_view` (read entries, selected_index), `ws.local_history` (take_snapshot), a `ws.editor.tabs` (write content). Řešení: extrahovat potřebná data z `history_view` do lokálních proměnných (entries[idx], relative_path, historický obsah) PŘED mutable operacemi.
- **Nedostatečná invalidace po restore** — Po restore se mění `current_content` v `HistoryViewState`, ale diff cache je stále platný pro starý obsah. Nutné: resetovat `diff_for_index = None`, `content_hash = 0` (nebo nový hash) → vynutí přepočet diffu příští frame.
- **Entries refresh po snapshot** — `get_history()` čte filesystem. Nový snapshot se musí fyzicky zapsat PŘED voláním `get_history()`. `take_snapshot()` je synchronní, takže pořadí je zaručené.
- **Selected index po refresh** — Po refresh entries (nový snapshot na pozici 0) se stávající `selected_index` posune o 1 (ukazuje na starší verzi). Pro UX: po restore nastavit `selected_index = Some(0)` (nejnovější = právě obnovený snapshot).
- **show_modal ID kolize** — ID musí být unikátní v rámci egui context (sdílený mezi viewporty). Použít `"history_restore_modal"` + salt z `history_view.file_path` pro unikátnost.

## Open Risks

- **Borrow checker překvapení** — I když disjoint borrows na WorkspaceState by měly fungovat, restore flow má složitější data flow (read history_view → write tab → write local_history → write history_view). Může vyžadovat dočasné `.clone()` nebo `take()` + put back pattern.
- **Synchronní I/O v UI vlákně** — `take_snapshot()` dělá `fs::write()`. Pro malé/střední soubory (<1MB) je to sub-ms. Pro velké soubory by mohlo zpomalit UI frame. Akceptovatelné — restore je explicitní uživatelská akce (ne per-keystroke), a autosave snapshot taky používá synchronní I/O v background.rs (kde se ale volá z background processing, ne přímo z UI render). Pokud by se ukázalo jako problém, lze přesunout do background_io_tx kanálu (ale pak ztratíme synchronní refresh entries).

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | none found | Žádná relevantní skill pro egui UI |
| Rust (general) | jeffallan/claude-skills@rust-engineer | available (1.1K installs), ale pro tento scope nepotřebné |

## Sources

- `show_modal` pattern z `src/app/ui/widgets/modal.rs` (řádky 278–322)
- Delete confirm dialog z `src/app/ui/file_tree/dialogs.rs` (řádky 195–240) jako referenční pattern
- `HistorySplitResult` z `src/app/ui/workspace/history/mod.rs` (řádky 56–61)
- Toolbar layout z `src/app/ui/workspace/history/mod.rs` (řádky 270–350)
- Restore signál handling z `src/app/ui/workspace/mod.rs` (řádky 760–800)
- egui Modal API z Context7 docs (/emilk/egui)
