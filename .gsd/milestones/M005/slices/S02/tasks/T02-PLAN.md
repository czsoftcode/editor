---
estimated_steps: 6
estimated_files: 7
---

# T02: Replace UI, preview dialog, snapshot wiring a i18n

**Slice:** S02 — Project-wide replace s preview a local history
**Milestone:** M005

## Description

UI pro replace flow: replace input v search dialogu, preview dialog s per-file diff a checkboxy, snapshot volání přes `take_snapshot()` v workspace handleru před zápisem, error handling přes toast, refresh otevřených tabů, a i18n pro všech 5 jazyků.

## Steps

1. Rozšířit search dialog o replace UI: toggle button vedle query inputu přepíná zobrazení replace input pole. "Replace All" button pod replace inputem — spustí `compute_replace_previews()` a nastaví `show_replace_preview = true`.
2. Implementovat replace preview dialog (nová funkce `render_replace_preview_dialog()`): modal s nadpisem "Replace Preview". Per-file collapsible sekce: checkbox + filename + match count. Uvnitř sekce: diff view — starý text červeně, nový text zeleně přes LayoutJob (použít pattern z diff_view.rs). Nahoře: "Select All" / "Deselect All" buttons. Dole: "Potvrdit" a "Zrušit" buttons.
3. Potvrzení replace: nastavit flag `pending_replace: bool` na ProjectSearch. V workspace/mod.rs po renderingu: pokud `pending_replace` → pro každý vybraný `ReplacePreview`: (a) `local_history.take_snapshot(relative_path, &preview.original_content)` — pokud selže → toast error, skip soubor; (b) `fs::write(&preview.file, &preview.new_content)` — pokud selže → toast error, pokračovat. Resetovat `pending_replace`.
4. Summary toast po dokončení: "Nahrazeno v {success_count} souborech" pokud žádné chyby. "Nahrazeno v {success_count} z {total} souborů ({error_count} chyb)" pokud chyby.
5. Refresh otevřených tabů: po replace projít `ws.tabs` — pokud `tab.file_path` odpovídá modifikovanému souboru, načíst nový obsah z disku do `tab.content` a nastavit `tab.modified = false`.
6. i18n: přidat replace-specific klíče do všech 5 locale souborů: `project-search-replace-preview-title`, `project-search-replace-confirm`, `project-search-replace-cancel`, `project-search-replace-select-all`, `project-search-replace-deselect-all`, `project-search-replace-success`, `project-search-replace-partial-success`, `project-search-replace-snapshot-error`, `project-search-replace-write-error`.

## Must-Haves

- [ ] Replace input toggle v search dialogu
- [ ] Preview dialog s per-file diff a checkboxy
- [ ] Select all / Deselect all funkčnost
- [ ] `take_snapshot()` volaný před každým zápisem v workspace handleru
- [ ] Per-file error handling — snapshot selhání skip, write selhání toast + pokračovat
- [ ] Summary toast po dokončení
- [ ] Refresh otevřených tabů po replace
- [ ] i18n klíče ve všech 5 jazycích
- [ ] `./check.sh` pass

## Verification

- `cargo check` — kompilace čistá
- `./check.sh` — fmt, clippy, všechny testy pass
- `grep -c 'project-search-replace' locales/*/ui.ftl` — ≥7 per jazyk

## Inputs

- T01 output: `ReplacePreview` struct, `compute_replace_previews()`, `apply_replacements()`
- `src/app/local_history.rs` — `take_snapshot()` API
- `src/app/types.rs` — Toast API
- `src/app/ui/editor/diff_view.rs` — LayoutJob diff rendering pattern

## Observability Impact

- **New runtime signals:** replace summary toast po dokončení (úspěch/partial), per-file error toast pro snapshot/write selhání
- **Inspection surfaces:** `ProjectSearch.pending_replace` flag — indikuje probíhající replace operaci; `ProjectSearch.replace_previews` — aktuální preview data s per-file `selected` stavem; `ProjectSearch.show_replace_preview` — viditelnost preview dialogu
- **Failure visibility:** snapshot selhání → toast s cestou a chybou, soubor přeskočen (nezapsán); write selhání → toast s cestou a chybou, pokračuje s dalšími; summary toast zobrazí success/error count
- **Future agent inspection:** zkontrolovat `replace_previews` vektor — `selected` flag per-file; toast queue v `ws.toasts` pro error messages

## Expected Output

- `src/app/ui/search_picker.rs` — replace UI (input toggle, preview dialog)
- `src/app/ui/workspace/mod.rs` — replace execution flow (snapshot + write + tab refresh + toasty)
- `locales/{cs,en,sk,de,ru}/ui.ftl` — replace i18n klíče
