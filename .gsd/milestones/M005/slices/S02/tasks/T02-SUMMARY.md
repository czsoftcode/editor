---
id: T02
parent: S02
milestone: M005
provides:
  - Replace input toggle v search dialogu (↔ button)
  - Replace preview dialog s per-file inline diff a checkboxy
  - Select all / Deselect all funkčnost
  - Replace execution flow v workspace handleru (snapshot → write → tab refresh → toast)
  - i18n klíče pro replace UI (14 klíčů × 5 jazyků)
  - Unit test pro apply_replacements s neexistující cestou
key_files:
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/state/types.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Replace toggle jako ↔ button v řadě s regex/case/word togglery — konzistentní s existujícím pattern
  - pending_replace flag na ProjectSearch — workspace handler provede snapshot+write po renderingu, odděleně od UI
  - Preview dialog default-open collapsing headers pro ≤5 souborů, collapsed pro >5
  - Tab refresh po replace — načtení z disku + reset modified flag + sync last_saved_content
patterns_established:
  - Replace execution flow: UI nastaví pending_replace → workspace handler iteruje previews → snapshot → write → toast → tab refresh → cleanup
observability_surfaces:
  - pending_replace flag — resetuje se na false po dokončení replace operace
  - replace_previews vektor — per-file selected stav, inspektovatelný před/po potvrzení
  - Toast queue — per-file snapshot/write chyby s názvem souboru a error message
  - Summary toast — success count (nebo success/total/errors při partial failure)
duration: 25m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Replace UI, preview dialog, snapshot wiring a i18n

**Replace input toggle, preview dialog s per-file diff/checkboxy, snapshot+write execution v workspace handleru, tab refresh, summary/error toasty, a 14 i18n klíčů v 5 jazycích.**

## What Happened

1. Search dialog rozšířen o replace toggle (↔ button) vedle regex/case/word togglerů. Toggle zobrazí replace input pole pod query. "Replace All" button ve footeru spustí `compute_replace_previews()` a otevře preview dialog.
2. Implementován `render_replace_preview_dialog()` — modal s per-file collapsible sekcemi. Každá sekce: checkbox + filename + match count. Uvnitř: inline diff přes `similar::TextDiff` s červeným/zeleným zvýrazněním (pattern z diff_view.rs). Select All / Deselect All buttons nahoře, selection info counter. Confirm/Cancel buttons ve footeru.
3. Replace execution v workspace/mod.rs: po renderingu, pokud `pending_replace` → pro každý selected preview: (a) `take_snapshot()` s original_content, (b) `fs::write()` s new_content. Snapshot selhání → toast + skip soubor. Write selhání → toast + pokračovat.
4. Summary toast: "Nahrazeno v {N} souborech" při úspěchu, "Nahrazeno v {N} z {M} souborů ({K} chyb)" při partial failure.
5. Tab refresh: po replace projde `ws.editor.tabs` — pokud cesta odpovídá modifikovanému souboru, načte nový obsah z disku a nastaví `modified = false` a `last_saved_content` (prevence falešné "unsaved" indikace).
6. Přidáno `pending_replace: bool` field do `ProjectSearch` struct.
7. i18n: 14 replace-specific klíčů ve všech 5 jazycích (cs, en, sk, de, ru): preview-title, confirm, cancel, select-all, deselect-all, selection-info, success, partial-success, snapshot-error, write-error.
8. Nový unit test `test_apply_replacements_nonexistent_file_error` — ověřuje per-file error handling s neexistující cestou.

## Verification

- `cargo check` — čistá kompilace
- `cargo fmt --all` — formátování OK
- `./check.sh` — fmt, clippy, 192+ testů pass, quality gate passed
- `cargo test --bin polycredo-editor app::ui::search_picker` — 20 testů pass (15 search + 5 replace)
- `grep -c 'project-search-replace' locales/*/ui.ftl` — 14 per jazyk (požadováno ≥7)

### Slice-level verification:
- ✅ `cargo test --bin polycredo-editor app::ui::search_picker` — replace unit testy pass (20/20)
- ✅ `cargo check` — kompilace čistá
- ✅ `./check.sh` — fmt, clippy, všechny testy pass
- ✅ `apply_replacements()` s neexistující cestou vrací `Err(String)` per-file — test ověřuje neblokování ostatních + chybová hláška obsahuje název souboru
- ✅ Diagnostic: `pending_replace` flag resetován po dokončení, `replace_previews` vyčištěny

## Diagnostics

- `ProjectSearch.pending_replace` — `true` jen po kliknutí Confirm, reset na `false` po zpracování
- `ProjectSearch.replace_previews` — vektor s per-file `selected` stavem, vyčištěn po replace
- `ProjectSearch.show_replace_preview` — viditelnost preview dialogu
- Toast queue (`ws.toasts`) — per-file error toasty s cestou a chybou pro snapshot/write selhání
- Summary toast — success count nebo success/total/errors ratio

## Deviations

- Přidán `project-search-replace-selection-info` i18n klíč (counter "{selected} z {total} vybráno") — nebylo v plánu, ale zlepšuje UX preview dialogu.
- Heading search dialogu se mění na "Nahradit v projektu" když je replace toggle aktivní — logické rozšíření.

## Known Issues

None.

## Files Created/Modified

- `src/app/ui/search_picker.rs` — replace toggle UI, `render_replace_preview_dialog()` funkce, 1 nový unit test
- `src/app/ui/workspace/mod.rs` — replace execution flow (snapshot + write + tab refresh + toast), import `render_replace_preview_dialog`
- `src/app/ui/workspace/state/types.rs` — `pending_replace: bool` field na `ProjectSearch`
- `locales/cs/ui.ftl` — 11 nových replace i18n klíčů
- `locales/en/ui.ftl` — 11 nových replace i18n klíčů
- `locales/sk/ui.ftl` — 11 nových replace i18n klíčů
- `locales/de/ui.ftl` — 11 nových replace i18n klíčů
- `locales/ru/ui.ftl` — 11 nových replace i18n klíčů
