---
status: diagnosed
trigger: "Diagnostikuj root cause pro UAT gap: ctrl+w mi upravi soubor smazanim nejakych znaku, po ulozeni, zavreni a znovuotevreni jsou ty znaky smazany"
created: 2026-03-10T18:04:12Z
updated: 2026-03-10T18:05:39Z
---

## Current Focus

hypothesis: Ctrl+W je detekovano pro close-flow, ale event se neconsume a editor ho stale zpracuje jako textovou operaci (mazani slova), cimz modifikuje obsah souboru.
test: Porovnat handling Ctrl+W ve workspace (key_pressed bez consume) s editor TextEdit code path a modal gating.
expecting: Najdu Ctrl+W check bez consume + aktivni TextEdit, ktery muze stejnou klavesu aplikovat na obsah.
next_action: vratit diagnostiku root cause bez aplikace fixu (goal=find_root_cause_only)

## Symptoms

expected: Stisk Ctrl+W nad neulozenou aktivni kartou otevre guard dialog (Ulozit / Zahodit / Zrusit), karta se hned nezavre.
actual: ctrl+w upravi soubor smazanim nejakych znaku; po ulozeni, zavreni a znovuotevreni jsou znaky smazany.
errors: None reported.
reproduction: Test 1 in UAT.
started: Discovered during UAT.

## Eliminated

- hypothesis: Ctrl+W neni nikde namapovano na close tab.
  evidence: Ve workspace existuje explicitni mapovani `if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) { request_close_active_tab(ws); }`.
  timestamp: 2026-03-10T18:05:39Z

## Evidence

- timestamp: 2026-03-10T18:05:39Z
  checked: src/app/ui/workspace/mod.rs:398-400
  found: Ctrl+W je detekovano pres `key_pressed`, ale bez `consume_key`/`consume_shortcut`.
  implication: Event muze projit i do dalsich widgetu ve stejnem frame.

- timestamp: 2026-03-10T18:05:39Z
  checked: src/app/ui/editor/render/normal.rs:129-137
  found: Editor pouziva `egui::TextEdit::multiline(...).code_editor().interactive(true)`.
  implication: Pri fokusu editoru zpracovava textove shortcuty; Ctrl+W muze projevit textovou editaci (mazani slova) a nastavit `modified=true`.

- timestamp: 2026-03-10T18:05:39Z
  checked: src/app/ui/workspace/mod.rs:420-431
  found: `dialog_open_base` nezahrnuje `ws.pending_close_flow`, guard modal je renderovan az nasledne.
  implication: I pri aktivnim guard flow zustava editor v aktualnim frame interaktivni, takze shortcut event neni striktne izolovan od editoru.

- timestamp: 2026-03-10T18:05:39Z
  checked: src/app/ui/workspace/state/mod.rs:215-243
  found: `build_dirty_close_queue` vraci vsechny dirty taby (aktivni + ostatni), ne jen aktualni tab.
  implication: Potvrzuje souvisejici UAT gap, ze Ctrl+W postupne resi vsechny neulozene soubory.

## Resolution

root_cause:
  Ctrl+W close shortcut je implementovan jako pasivni detekce (`key_pressed`) bez explicitniho consume; ve stejnem UI cyklu zustava editorovy `TextEdit` interaktivni a zpracuje stejnou klavesu jako editacni prikaz (mazani slova/znaku). Proto stisk Ctrl+W muze mutovat obsah souboru misto cisteho spuštění guard flow.
fix:
verification:
files_changed: []
