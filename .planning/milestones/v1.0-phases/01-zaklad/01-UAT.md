---
status: diagnosed
phase: 01-zaklad
source:
  - 01-01-SUMMARY.md
  - 01-02-SUMMARY.md
started: 2026-03-04T20:54:51Z
updated: 2026-03-04T21:05:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Startup bez tmavého flash při uloženém light mode
expected: Po restartu aplikace s uloženým light mode se UI vykreslí rovnou světlé bez úvodního tmavého frame.
result: pass

### 2. Přepnutí dark/light aktualizuje celé UI
expected: Přepnutí dark/light v Settings okamžitě přebarví menu, panely a dialogové prvky konzistentně s aktivním tématem.
result: issue
reported: "vse se prepise, krome ramu terminalu, je syte cerny"
severity: cosmetic

### 3. Syntax highlighting je v light mode čitelný
expected: V light mode je syntax highlighting čitelný (zejména světlé tokeny jako žlutá nebudou splývat s pozadím).
result: pass

### 4. Runtime změna tématu aktualizuje highlighting bez restartu
expected: Při přepnutí tématu za běhu se barvy syntaxe okamžitě přepnou na odpovídající syntect téma bez restartu aplikace.
result: pass

### 5. Starý settings.json bez light_variant aplikaci nerozbije
expected: Po načtení starší konfigurace bez `light_variant` aplikace normálně naběhne a použije defaultní variantu.
result: skipped
reason: user skipped

### 6. Záložky a status bar jsou čitelné v obou režimech
expected: Text ve status baru a indikátor neuložených změn (●) v záložkách zůstávají dobře čitelné v dark i light mode.
result: issue
reported: "pismo ve status baru v light mode je spatne citelne - moc svetle, splyva se status barem"
severity: cosmetic

## Summary

total: 6
passed: 3
issues: 2
pending: 0
skipped: 1

## Gaps

- truth: "Přepnutí dark/light v Settings okamžitě přebarví menu, panely a dialogové prvky konzistentně s aktivním tématem."
  status: failed
  reason: "User reported: vse se prepise, krome ramu terminalu, je syte cerny"
  severity: cosmetic
  test: 2
  root_cause: "Floating terminal window frame používá hardcoded dark fill `Color32::from_rgb(20, 20, 25)` v `window.rs`, který ignoruje light/dark visuals."
  artifacts:
    - path: "src/app/ui/terminal/window.rs"
      issue: "Hardcoded dark `viewer_bg` pro frame fill místo theme-aware barvy."
    - path: "src/app/ui/terminal/right/mod.rs"
      issue: "Používá `StandardTerminalWindow`, takže problém se projeví v AI floating terminálu."
    - path: "src/app/ui/terminal/bottom/mod.rs"
      issue: "Používá `StandardTerminalWindow`, takže problém se projeví i v build floating terminálu."
  missing:
    - "Nahradit fixní `viewer_bg` barvou odvozenou z `ui.visuals()`/`Frame::window`."
    - "Ověřit, že frame fill se korektně mění při runtime dark/light switchi."
  debug_session: ".planning/debug/terminal-frame-remains-pure-black-in-light-mode.md"
- truth: "Text ve status baru a indikátor neuložených změn (●) v záložkách zůstávají dobře čitelné v dark i light mode."
  status: failed
  reason: "User reported: pismo ve status baru v light mode je spatne citelne - moc svetle, splyva se status barem"
  severity: cosmetic
  test: 6
  root_cause: "Status bar render v `Editor::status_bar()` používá fixní světlé RGB text barvy bez dark/light větvení, takže v light mode text splývá s pozadím."
  artifacts:
    - path: "src/app/ui/editor/ui.rs"
      issue: "Hardcoded text palette (`235,240,248`, `195,205,220`, ...) bez theme adaptace."
    - path: "src/settings.rs"
      issue: "Light mode nastavuje světlé visuals; bez adaptace textu klesá kontrast status baru."
  missing:
    - "Odvodit status bar text barvy z `ui.visuals().text_color()`/`weak_text_color()` nebo explicitně větvit light/dark paletu."
    - "Zkontrolovat čitelnost všech status bar segmentů (mode, cursor, dirty state) v light i dark."
  debug_session: ".planning/debug/status-bar-text-too-light-in-light-mode.md"
