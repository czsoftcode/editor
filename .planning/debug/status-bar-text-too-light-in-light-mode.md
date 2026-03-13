---
status: diagnosed
trigger: "Investigate issue: uat-02-statusbar-low-contrast"
created: 2026-03-04T21:02:39Z
updated: 2026-03-04T21:04:31Z
---

## Current Focus

hypothesis: Root cause je hardcoded dark-oriented paleta ve `Editor::status_bar()`, která se používá i v light mode bez adaptace na `ui.visuals().dark_mode`.
test: Ověřit výskyty těchto barev a vazbu, že `workspace/mod.rs` volá právě tuto implementaci.
expecting: Potvrzení, že status bar text používá fixní světlé RGB hodnoty v každém režimu.
next_action: Vrátit strukturovanou diagnózu s konkrétními řádky a návrhem směru opravy.

## Symptoms

expected: Text ve status baru a indikátor neuložených změn (●) v záložkách zůstávají dobře čitelné v dark i light mode.
actual: Uživatel hlásí: "pismo ve status baru v light mode je spatne citelne - moc svetle, splyva se status barem".
errors: None reported
reproduction: Test 6 in UAT
started: Discovered during UAT

## Eliminated

- hypothesis: Nízký kontrast status baru způsobuje explicitně obarvený indikátor `●` v záložkách.
  evidence: `src/app/ui/editor/render/tabs.rs` skládá `●` do labelu, ale pro modified tab nenastavuje vlastní světlou barvu; kromě `deleted` se používá defaultní textová barva tématu.
  timestamp: 2026-03-04T21:04:31Z

## Evidence

- timestamp: 2026-03-04T00:00:00Z
  checked: .planning/phases/01-zaklad/01-UAT.md
  found: Test 6 je reprodukovaný UAT nález, klasifikovaný jako `issue` (cosmetic), bez vyplněného root cause.
  implication: Problém je potvrzený funkčním testem a je potřeba trasovat čistě render/styl logiku.
- timestamp: 2026-03-04T00:00:00Z
  checked: src/app/ui/widgets/status_bar.rs
  found: Soubor v aktuálním stromu neexistuje (`No such file or directory`).
  implication: Implementace status baru byla přesunuta nebo přejmenována; je nutné nejdřív lokalizovat aktuální render cestu.
- timestamp: 2026-03-04T21:03:20Z
  checked: fulltext search v `src/app`
  found: `src/app/ui/editor/ui.rs` obsahuje `pub fn status_bar(...)` + panel `TopBottomPanel::bottom(\"status_bar\")` + barvy `status_warn_color`/`status_ok_color`; tab indikátor `●` je v `src/app/ui/editor/render/tabs.rs` a `src/app/ui/editor/render_tabs.rs`.
  implication: Hlavní kandidát root cause je konkrétní barevná volba ve `status_bar()` a/nebo tab rendereru.
- timestamp: 2026-03-04T21:04:31Z
  checked: `src/app/ui/editor/ui.rs` + `src/app/ui/workspace/mod.rs` + `src/app/ui/editor/mod.rs`
  found: Aktivní status bar je `Editor::status_bar()` z `ui.rs` (modul je připojen přes `mod ui;` a volán z `workspace/mod.rs`); textové barvy jsou fixní světlé RGB: `primary(235,240,248)`, `secondary(195,205,220)`, `warn(255,200,120)`, `ok(170,230,185)` bez větvení na dark/light.
  implication: V light mode se použijí stejné světlé hodnoty jako v dark mode, což vede k nízkému kontrastu proti světlému pozadí status baru.
- timestamp: 2026-03-04T21:04:31Z
  checked: `src/settings.rs`
  found: Light režim nastavuje `egui::Visuals::light()` (`to_egui_visuals()`), tedy světlé UI pozadí.
  implication: Kombinace světlého pozadí + fixně světlého textu ve status baru vysvětluje hlášené splývání.

## Resolution

root_cause: "Editor status bar používá natvrdo světlé textové barvy optimalizované pro dark režim a neadaptuje je podle aktivního tématu. Tyto barvy se aplikují i v light mode, kde mají nedostatečný kontrast."
fix: ""
verification: ""
files_changed: []
