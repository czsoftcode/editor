---
estimated_steps: 9
estimated_files: 9
---

# T03: Keymap dispatch + smazání mrtvých modálů + i18n + cleanup

**Slice:** S01 — Inline project search panel
**Milestone:** M006

## Description

Uzavření slice — keymap dispatch přepojit na show_panel toggle, menu action přesměrovat, smazat mrtvé modální dialogy a jejich volání z workspace, přidat i18n klíče, Escape handling, final quality gate.

## Steps

1. V `src/app/keymap.rs` — najít dispatch pro `CommandId::ProjectSearch`:
   - Změnit z `ws.project_search.show_input = true; ws.project_search.focus_requested = true;`
   - Na: `ws.project_search.show_panel = !ws.project_search.show_panel; if ws.project_search.show_panel { ws.project_search.focus_requested = true; }`
   - Toggle chování — Ctrl+Shift+F otevře nebo zavře panel

2. V `src/app/ui/workspace/menubar/mod.rs` — `process_menu_actions()`:
   - Najít `if actions.project_search { ws.project_search.show_input = true; ws.project_search.focus_requested = true; }`
   - Změnit na: `if actions.project_search { ws.project_search.show_panel = true; ws.project_search.focus_requested = true; }`
   - Menu akce otevírá panel (ne toggle — menu vždy otevírá)

3. V `src/app/ui/workspace/mod.rs`:
   - Smazat volání `render_project_search_dialog(ctx, ws, t)` — modální input dialog nahrazen panelem
   - Smazat/podmínit volání `poll_and_render_project_search_results(ctx, ws, t)` — poll loop přesunut do panelu
   - Ponechat `render_replace_preview_dialog(ctx, ws, t)` — zůstává modální
   - Ponechat workspace handler pro `pending_replace` — snapshot+write flow nezměněn

4. V `src/app/ui/search_picker.rs`:
   - Smazat funkci `render_project_search_dialog()` — mrtvý kód
   - Smazat funkci `poll_and_render_project_search_results()` — mrtvý kód
   - Ponechat `render_replace_preview_dialog()` — stále používána
   - Ponechat všechny engine funkce (build_regex, search_file_with_context, run_project_search, compute_replace_previews, apply_replacements) a unit testy

5. Smazat `show_input` field z `ProjectSearch` v types.rs:
   - Ověřit že nikde není používán (grep)
   - Pokud je stále referencován jinde, přesunutí reference na show_panel

6. Escape handling v render_search_panel():
   - Detekce Escape: `if ui.input(|i| i.key_pressed(Key::Escape))` uvnitř panelu
   - Pouze pokud žádný modální dialog není otevřený (kontrola dialog flags)
   - `ws.project_search.show_panel = false` — ale nezmaže query ani results
   - Close button (✕) v panelu — stejné chování jako Escape

7. i18n — přidat nové klíče do `locales/{cs,en,sk,de,ru}/ui.ftl`:
   - `project-search-panel-title` = "Vyhledávání v projektu" / "Project Search" / ...
   - `project-search-no-results` = "Žádné výsledky" / "No results" / ...
   - `project-search-results-count` = "výsledků" / "results" / ... (nebo existující klíč)
   - Zkontrolovat existující klíče — většinu sdílet (`project-search-searching`, `project-search-results-title`, toggle labely)
   - Přidat chybějící klíče pouze tam kde existující nevyhovují

8. `cargo fmt` + `cargo clippy` — vyřešit warningy z nepoužívaného kódu (smazané funkce)

9. Final quality gate: `./check.sh` — všechny testy pass, fmt OK, clippy OK

## Must-Haves

- [ ] Ctrl+Shift+F toggle show_panel v keymap dispatch
- [ ] Menu Project Search otevírá panel
- [ ] render_project_search_dialog() a poll_and_render_project_search_results() smazány
- [ ] show_input field smazán z ProjectSearch (nebo přejmenován)
- [ ] Escape v panelu zavře panel (show_panel = false) bez ztráty query/results
- [ ] i18n klíče ve všech 5 jazycích
- [ ] `./check.sh` projde čistě

## Verification

- `./check.sh` — "Quality Gate: All checks passed successfully!"
- `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0
- `grep -c 'poll_and_render_project_search_results' src/app/ui/workspace/mod.rs` → 0
- `grep -c 'show_input' src/app/ui/workspace/state/types.rs` → 0
- `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → ≥1
- `grep -c 'project-search-panel-title' locales/cs/ui.ftl` → 1
- `grep -c 'project-search-panel-title' locales/en/ui.ftl` → 1
- Existující 20 unit testů stále pass

## Observability Impact

- **Smazáno:** `ProjectSearch.show_input` — field odstraněn, už se nesleduje. Panel viditelnost je výhradně přes `show_panel`.
- **Změněno:** Keymap dispatch pro `CommandId::ProjectSearch` — nyní toggle `show_panel` (otevře/zavře), ne jednosměrné otevření.
- **Nové:** `project-search-panel-title` i18n klíč — panel heading v UI, inspekce přes lokalizační soubory.
- **Smazáno:** `render_project_search_dialog()` a `poll_and_render_project_search_results()` — mrtvý kód odstraněn, funkčnost přesunuta do `render_search_panel()`.
- **Escape handling:** Detekce `Key::Escape` zavře panel (`show_panel = false`) pokud není otevřený replace preview dialog. Query a výsledky zůstávají zachovány.
- **Budoucí agent:** Zkontroluj `show_panel` v types.rs pro panel stav, `grep 'render_project_search_dialog'` → 0 = mrtvý kód smazán, `grep 'show_input'` → 0 = field odstraněn.

## Inputs

- T02 výstup: plně funkční render_search_panel() s výsledky, klik handlery, replace flow
- `src/app/keymap.rs` — stávající CommandId::ProjectSearch dispatch
- `src/app/ui/workspace/menubar/mod.rs` — stávající process_menu_actions()

## Expected Output

- `src/app/keymap.rs` — dispatch přepojený na show_panel toggle
- `src/app/ui/workspace/menubar/mod.rs` — menu action přesměrovaná na show_panel
- `src/app/ui/workspace/mod.rs` — mrtvé modální volání smazána
- `src/app/ui/search_picker.rs` — mrtvé modální funkce smazány, render_search_panel() s Escape handling
- `src/app/ui/workspace/state/types.rs` — show_input smazán, show_panel canonical
- `locales/{cs,en,sk,de,ru}/ui.ftl` — nové i18n klíče
