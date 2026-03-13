# T01: 02-terminal-git-barvy 01

**Slice:** S04 — **Milestone:** M001

## Description

Zavést plně theme-aware rendering terminálu tak, aby v light mode nebyl tmavý background ani tmavý scrollbar, a přepínání tématu fungovalo za běhu bez restartu backendu.

Purpose: Splnit `TERM-01..TERM-04` na sdílené vrstvě terminálu (`instance`), takže změna pokryje Claude panel i Build terminál zároveň.
Output: `theme.rs` resolver + aplikace `set_theme(...)` v `Terminal::ui(...)` + scrollbar helpery napojené na aktivní téma.

## Must-Haves

- [ ] "Claude panel i Build terminál používají stejnou light-safe paletu v light mode; žádný default dark background `#181818` v light režimu"
- [ ] "Přepnutí dark/light se projeví okamžitě bez restartu PTY procesu, protože theme se aplikuje při každém `Terminal::ui(...)` renderu"
- [ ] "Scrollbar terminálu není hardcoded tmavý; track/thumb barvy se odvozují z aktivního tématu (idle + hover + drag)"
- [ ] "Foreground a klíčové ANSI barvy v light mode mají dostatečný kontrast vůči světlému pozadí"

## Files

- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/terminal/instance/render.rs`
- `src/app/ui/terminal/instance/theme.rs`
