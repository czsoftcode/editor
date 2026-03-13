---
status: diagnosed
trigger: "Investigate issue: uat-01-terminal-frame-black — Po přepnutí dark/light zůstává rám terminálu sytě černý."
created: 2026-03-04T21:03:03Z
updated: 2026-03-04T21:04:54Z
---

## Current Focus

hypothesis: Root cause potvrzena: fixní tmavý `viewer_bg` v `StandardTerminalWindow` drží frame černý i v light mode.
test: Důkaz kompletován (code path + vyřazení alternativy na úrovni terminal theme).
expecting: Diagnóza se vrátí jako `ROOT CAUSE FOUND` bez aplikace fixu (mode `find_root_cause_only`).
next_action: Vrátit strukturovanou diagnózu.

## Symptoms

expected: Přepnutí dark/light v Settings okamžitě přebarví menu, panely a dialogové prvky konzistentně s aktivním tématem.
actual: Uživatel hlásí: "vse se prepise, krome ramu terminalu, je syte cerny".
errors: None reported
reproduction: Test 2 in UAT
started: Discovered during UAT

## Eliminated

<!-- APPEND only -->

- hypothesis: Vnitřní terminálové téma se při přepnutí nezmění a zůstává dark, proto je rám černý.
  evidence: `TerminalView` dostává theme přes `terminal_theme_for_visuals(ui.visuals())` a `instance/theme.rs` obsahuje explicitní light paletu + testy pro light background/readability.
  timestamp: 2026-03-04T21:04:20Z

## Evidence

<!-- APPEND only -->

- timestamp: 2026-03-04T21:03:03Z
  checked: src/app/ui/terminal/instance/mod.rs
  found: `TerminalView` je renderovaný přes `.set_theme(terminal_theme_for_visuals(ui.visuals()))`.
  implication: Vnitřní barvy terminálu reagují na aktivní `egui` visuals; problém bude pravděpodobně mimo samotný terminal content.

- timestamp: 2026-03-04T21:03:03Z
  checked: src/app/ui/terminal/instance/render.rs
  found: Scrollbar barvy jsou odvozené z `ui.visuals()` (`panel_fill`, `widgets.active.fg_stroke`) a v light větvi se míchají směrem k bílé, ne k černé.
  implication: Hlásená sytě černá barva rámu nepochází ze scrollbar renderu.

- timestamp: 2026-03-04T21:03:40Z
  checked: src/app/ui/terminal/window.rs
  found: `StandardTerminalWindow::show()` nastavuje `.frame(egui::Frame::window(&ctx.style()).fill(viewer_bg))`, kde `viewer_bg` je natvrdo `Color32::from_rgb(20, 20, 25)`.
  implication: Floating terminálové okno má tmavý frame bez ohledu na aktuální light/dark režim; to přímo vysvětluje "sytě černý rám".

- timestamp: 2026-03-04T21:04:20Z
  checked: src/app/ui/terminal/instance/theme.rs
  found: `terminal_palette(false)` definuje světlé pozadí (`#f3f5f7`) a `terminal_theme_for_visuals` přepíná podle `visuals.dark_mode`.
  implication: Problém není v paletě terminálu; černý vzhled frame pochází z hardcoded barvy wrapperu.

- timestamp: 2026-03-04T21:04:20Z
  checked: src/app/ui/terminal/right/mod.rs, src/app/ui/terminal/bottom/mod.rs
  found: Obě větve floating terminálů vytvářejí okna přes `StandardTerminalWindow::new(...)`.
  implication: Stejná hardcoded frame barva se projeví konzistentně u AI/build terminálových floating oken.

## Resolution

root_cause:
  Hardcoded tmavá výplň `viewer_bg` (`Color32::from_rgb(20, 20, 25)`) v `StandardTerminalWindow` ignoruje light/dark theme visuals; při přepnutí tématu proto rám terminálového okna zůstává sytě černý.
fix:
verification:
files_changed: []
