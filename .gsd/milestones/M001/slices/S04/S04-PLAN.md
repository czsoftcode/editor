# S04: Terminal Git Barvy

**Goal:** Zavést plně theme-aware rendering terminálu tak, aby v light mode nebyl tmavý background ani tmavý scrollbar, a přepínání tématu fungovalo za běhu bez restartu backendu.
**Demo:** Zavést plně theme-aware rendering terminálu tak, aby v light mode nebyl tmavý background ani tmavý scrollbar, a přepínání tématu fungovalo za běhu bez restartu backendu.

## Must-Haves


## Tasks

- [x] **T01: 02-terminal-git-barvy 01** `est:5min`
  - Zavést plně theme-aware rendering terminálu tak, aby v light mode nebyl tmavý background ani tmavý scrollbar, a přepínání tématu fungovalo za běhu bez restartu backendu.

Purpose: Splnit `TERM-01..TERM-04` na sdílené vrstvě terminálu (`instance`), takže změna pokryje Claude panel i Build terminál zároveň.
Output: `theme.rs` resolver + aplikace `set_theme(...)` v `Terminal::ui(...)` + scrollbar helpery napojené na aktivní téma.
- [x] **T02: 02-terminal-git-barvy 02** `est:9min`
  - Nahradit heuristické darkening git barev explicitní semantickou light/dark paletou a garantovat čitelnost `??` na světlém pozadí.

Purpose: Splnit `TREE-01` a `TREE-02` s testovatelným mapováním statusů, které nebude závislé na náhodném odstínu vstupní barvy.
Output: Sdílený git status resolver + upravený file tree render s explicitní light paletou.

## Files Likely Touched

- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/terminal/instance/render.rs`
- `src/app/ui/terminal/instance/theme.rs`
- `src/app/ui/background.rs`
- `src/app/ui/file_tree/mod.rs`
- `src/app/ui/file_tree/render.rs`
- `src/app/ui/git_status.rs`
- `src/app/ui/workspace/state/mod.rs`
