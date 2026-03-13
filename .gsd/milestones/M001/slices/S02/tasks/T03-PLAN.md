# T03: 01-zaklad 03

**Slice:** S02 — **Milestone:** M001

## Description

Uzavřít UAT gap #2: floating terminal rám zůstává čistě černý v light mode.

Purpose: Odstranit hardcoded tmavý frame u standardního terminálového okna a navázat barvu rámu na aktivní téma.
Output: `src/app/ui/terminal/window.rs` s theme-aware frame fill, který funguje pro oba floating terminály.

## Must-Haves

- [ ] "Floating terminal window frame nepoužívá hardcoded tmavou výplň; fill je odvozený z aktivních `egui::Visuals`."
- [ ] "Při runtime přepnutí dark/light se frame floating terminálu přebarví konzistentně s tématem bez restartu."
- [ ] "Build i AI floating terminál (přes `StandardTerminalWindow`) sdílí stejný theme-aware frame behavior."

## Files

- `src/app/ui/terminal/window.rs`
