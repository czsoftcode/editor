---
status: complete
phase: 25-unsaved-close-guard
source: [.planning/phases/25-unsaved-close-guard/25-01-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-02-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-03-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-04-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-05-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-06-SUMMARY.md, .planning/phases/25-unsaved-close-guard/25-07-SUMMARY.md]
started: 2026-03-10T17:46:17Z
updated: 2026-03-10T19:02:40Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Guard při Ctrl+W na neuložené kartě
expected: Stisk Ctrl+W nad neuloženou aktivní kartou otevře guard dialog (Uložit / Zahodit / Zrušit), karta se hned nezavře.
result: pass

### 2. Guard při zavření karty přes menu
expected: Akce Menu -> Close Tab nad neuloženou aktivní kartou otevře stejný guard dialog jako Ctrl+W.
result: skipped
reason: "Uživatel nenašel položku Menu -> Close Tab; otestoval pouze Soubor -> Zavřít soubor (bez mazání znaků)."

### 3. Guard při kliknutí na X v záložce
expected: Kliknutí na X u neuložené aktivní karty otevře stejný guard dialog, nikoli okamžité zavření.
result: pass

### 4. Čistá karta se zavře bez dialogu
expected: Pokud aktivní karta nemá neuložené změny, zavření karty (Ctrl+W/menu/X) proběhne ihned bez guard dialogu.
result: pass

### 5. Volba Zrušit nic nezavře
expected: V guard dialogu volba Zrušit (i Esc/zavření dialogu) ponechá kartu otevřenou a ukončí close flow bez vedlejších efektů.
result: pass

### 6. Fronta více neuložených karet
expected: Při více neuložených kartách se po Uložit nebo Zahodit posouvá flow na další položku fronty, dokud není hotovo nebo uživatel nedá Zrušit.
result: pass

### 7. Selhání uložení během guard flow
expected: Když Uložit selže, aktuální karta zůstane otevřená, zobrazí se chybová hláška (inline i toast) a uživatel může zkusit Uložit znovu nebo zvolit Zahodit/Zrušit.
result: skipped
reason: "skip"

### 8. Root close guard (Quit/Close Project)
expected: Při Quit/Close Project/root close s neuloženými kartami se nejdřív spustí guard flow nad neuloženými soubory; při Zrušit se globální zavření neprovede.
result: pass

### 9. Lokalizace guard dialogu
expected: Po přepnutí jazyka se titulek, text a tlačítka guard dialogu i hláška při selhání uložení zobrazují v zvoleném jazyce.
result: pass

## Summary

total: 9
passed: 7
issues: 0
pending: 0
skipped: 2

## Gaps

- truth: "Stisk Ctrl+W nad neuloženou aktivní kartou otevře guard dialog (Uložit / Zahodit / Zrušit), karta se hned nezavře."
  status: resolved
  reason: "User reported: ctrl+w mi upravi soubor smazanim nejakych znaku, po ulozeni, zavreni a znovuotevreni jsou ty znaky smazany"
  severity: major
  test: 1
  root_cause: "Ctrl+W shortcut je zpracovaný bez consume_key/consume_shortcut, takže event propadne do TextEdit a vyvolá editaci (mazání znaků) místo čistého close-guard flow."
  artifacts:
    - path: "src/app/ui/workspace/mod.rs"
      issue: "Ctrl+W je čten přes key_pressed bez spotřebování eventu; guard flow také není započten v dialog_open_base."
    - path: "src/app/ui/editor/render/normal.rs"
      issue: "Interaktivní TextEdit přijímá stejný keyboard event a provede editační akci."
  missing:
    - "Použít consume_shortcut/consume_key pro Ctrl+W ve workspace shortcut handleru."
    - "Blokovat editor input během aktivního unsaved guard flow (zahrnout do dialog_open_base nebo ekvivalentu)."
  debug_session: ".planning/debug/ctrl-w-unsaved-guard-mutates-content.md"
- truth: "V guard dialogu volba Zrušit (i Esc/zavření dialogu) ponechá kartu otevřenou a ukončí close flow bez vedlejších efektů."
  status: resolved
  reason: "User reported: Esc nefunguje - fokus ma editor a ne modal, jinak pass"
  severity: major
  test: 5
  root_cause: "Unsaved guard není zahrnutý do dialog_open_base a flow každým frame refocusuje editor přes open_file; modal navíc nemá explicitní Esc handling, takže Esc neukončí guard."
  artifacts:
    - path: "src/app/ui/workspace/mod.rs"
      issue: "dialog_open_base neobsahuje pending_close_flow; process_unsaved_close_guard_dialog volá open_file a vrací fokus editoru."
    - path: "src/app/ui/editor/tabs.rs"
      issue: "open_file nastavuje focus_editor_requested, což přebíjí fokus modalu."
    - path: "src/app/ui/dialogs/confirm.rs"
      issue: "Unsaved guard dialog nemá explicitní mapování Esc -> Cancel."
  missing:
    - "Přidat stav unsaved guard dialogu do globálního dialog/input locku."
    - "Nepředávat fokus zpět editoru během aktivního guard flow."
    - "Implementovat explicitní Esc handling v unsaved guard modalu (Cancel)."
  debug_session: ".planning/debug/unsaved-guard-esc-focus.md"
- truth: "Při více neuložených kartách se po Uložit nebo Zahodit posouvá flow na další položku fronty, dokud není hotovo nebo uživatel nedá Zrušit."
  status: resolved
  reason: "User reported: kdyz chci zavrit nejaky tab krizkem, tak ukazuje na posledni, ze neni ulozeny a ne na ten prave otevreny, nebo ten, ktery chci zavrit; pri neulozenych souborech se ctrl+w chova tak, ze postupne kontroluje a dotazuje se na vsechny neulozene soubory misto na ten jeden, ktery je aktivni"
  severity: major
  test: 6
  root_cause: "SingleTab close flow skládá frontu ze všech dirty tabů přes build_dirty_close_queue; klik na X jen přepne active_tab a použije stejnou cestu, takže guard míří na jiný tab a Ctrl+W prochází všechny dirty soubory."
  artifacts:
    - path: "src/app/ui/workspace/mod.rs"
      issue: "request_close_active_tab používá globální dirty queue; TabBarAction::Close(idx) sdílí stejný flow."
    - path: "src/app/ui/workspace/state/mod.rs"
      issue: "build_dirty_close_queue vrací všechny dirty taby, ne pouze target tab pro SingleTab mode."
    - path: "src/app/ui/workspace/tests/unsaved_close_guard.rs"
      issue: "Testy pro SingleTab implicitně potvrzují multi-item chování."
  missing:
    - "Rozdělit queue builder podle režimu (SingleTab = pouze cílový tab, WorkspaceClose = všechny dirty taby)."
    - "Pro klik na X předat explicitní target tab do close flow, ne jen změnit active_tab."
    - "Upravit testy, aby SingleTab nečekal iteraci přes více souborů."
  debug_session: ".planning/debug/unsaved-queue-target-order.md"
