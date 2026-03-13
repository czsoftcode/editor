# T04: 01-zaklad 04

**Slice:** S02 — **Milestone:** M001

## Description

Uzavřít UAT gap #6: status bar text je v light mode příliš světlý a špatně čitelný.

Purpose: Nahradit fixní status bar text paletu za theme-aware barvy, aby byl kontrast stabilní v dark i light režimu.
Output: `src/app/ui/editor/ui.rs` s theme-aware status bar barvami bez hardcoded světle-modrých konstant.

## Must-Haves

- [ ] "Status bar text barvy nejsou hardcoded světlé RGB konstanty; odvozují se z aktivního tématu (`ui.visuals()`)."
- [ ] "V light mode je text status baru (path, line/col, encoding, file type, save/LSP stavy) čitelný vůči backgroundu."
- [ ] "Při runtime přepnutí dark/light se barvy status baru aktualizují okamžitě bez restartu."

## Files

- `src/app/ui/editor/ui.rs`
