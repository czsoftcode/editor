# T02: 02-terminal-git-barvy 02

**Slice:** S04 — **Milestone:** M001

## Description

Nahradit heuristické darkening git barev explicitní semantickou light/dark paletou a garantovat čitelnost `??` na světlém pozadí.

Purpose: Splnit `TREE-01` a `TREE-02` s testovatelným mapováním statusů, které nebude závislé na náhodném odstínu vstupní barvy.
Output: Sdílený git status resolver + upravený file tree render s explicitní light paletou.

## Must-Haves

- [ ] "Git stavy M/A/??/D mají explicitní dark i light paletu; light mode nepoužívá globální `* 0.55` násobení"
- [ ] "`??` (untracked) je v light mode samostatná modro-azurová barva s dostatečným kontrastem"
- [ ] "Mapování git statusu na semantický typ je testovatelné a sdílené, aby barvy byly stabilní i při rename/copy statu"
- [ ] "Soubory bez git statusu dál používají standardní `ui.visuals().text_color()`"

## Files

- `src/app/ui/background.rs`
- `src/app/ui/file_tree/mod.rs`
- `src/app/ui/file_tree/render.rs`
- `src/app/ui/git_status.rs`
- `src/app/ui/workspace/state/mod.rs`
