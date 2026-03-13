---
status: diagnosed
trigger: "Diagnostikuj root cause pro UAT gap."
created: 2026-03-10T19:04:12+01:00
updated: 2026-03-10T19:05:38+01:00
---

## Current Focus

hypothesis: Potvrzeno: flow `SingleTab` je implementacne navazan na globalni queue builder a zpracovava vsechny dirty taby.
test: Dokonceno statickou analyzou triggeru, queue builderu a testu pro `unsaved_close_guard`.
expecting: N/A (hypoteza potvrzena).
next_action: Vratit strukturovanou diagnozu root cause bez aplikace fixu.

## Symptoms

expected: Při více neuložených kartách se po Uložit nebo Zahodit posouvá flow na další položku fronty, dokud není hotovo nebo uživatel nedá Zrušit.
actual: kdyz chci zavrit nejaky tab krizkem, tak ukazuje na posledni, ze neni ulozeny a ne na ten prave otevreny, nebo ten, ktery chci zavrit; pri neulozenych souborech se ctrl+w chova tak, ze postupne kontroluje a dotazuje se na vsechny neulozene soubory misto na ten jeden, ktery je aktivni
errors: None reported
reproduction: Test 6 in UAT
started: Discovered during UAT

## Eliminated

## Evidence

- timestamp: 2026-03-10T19:05:02+01:00
  checked: src/app/ui/workspace/mod.rs (`request_close_active_tab`)
  found: Funkce pro close aktivniho tabu sklada `queue = build_dirty_close_queue(Some(&active_path), &tabs_snapshot)` ze vsech dirty tabu; `mode` je sice `SingleTab`, ale queue neni omezena na cilovy tab.
  implication: `Ctrl+W` i close z tab baru zpracovavaji vic souboru, kdyz je otevreno vice neulozenych tabu.
- timestamp: 2026-03-10T19:05:02+01:00
  checked: src/app/ui/workspace/mod.rs (obsluha `TabBarAction::Close(idx)`)
  found: Klik na `X` jen prepne `ws.editor.active_tab = Some(idx)` a vola stejnou `request_close_active_tab`.
  implication: Klik na `X` sdili stejny globalni queue flow jako `Ctrl+W`, misto aby resil pouze kliknuty tab.
- timestamp: 2026-03-10T19:05:17+01:00
  checked: src/app/ui/workspace/tests/unsaved_close_guard.rs (`unsaved_close_guard_tab_triggers`)
  found: Test konstruuje `PendingCloseMode::SingleTab` s dvoupolozkovou `queue` a povazuje postup `Save -> Discard` pres obe polozky za korektni.
  implication: Chybna semantika je zafixovana i v testech; flow pro "single tab close" je navrzeny jako multi-tab queue.
- timestamp: 2026-03-10T19:05:38+01:00
  checked: `cargo test unsaved_close_guard_tab_triggers -- --nocapture`
  found: Test nebylo mozne spustit v tomto sandboxu kvuli `sccache: Operation not permitted` pri build skriptu `ring`.
  implication: Runtime verifikace v tomto prostredi neni dostupna, ale root cause je urceny primym ctenim produkcniho kodu a testove specifikace.
## Resolution

root_cause: `request_close_active_tab` pouziva `build_dirty_close_queue` (vsechny dirty taby) i pro `PendingCloseMode::SingleTab`. Tj. `Ctrl+W` i klik na `X` nespousti guard pouze pro cilovy tab, ale serialne pro celou dirty frontu. U kliknuti na `X` se navic cil urci neprimo prepnutim `active_tab = idx`, a pokud vybrany tab neni dirty, guard zacne na prvnim/serazenem jinem dirty tabu.
fix:
verification:
files_changed: []
