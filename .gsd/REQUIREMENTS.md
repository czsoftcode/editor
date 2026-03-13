# Requirements

Explicitní capability contract pro projekt PolyCredo Editor.

## Active

(none)

## Validated

### R001 — Editovatelný levý panel v history view
- Class: primary-user-loop
- Status: validated
- Description: Levý panel v history split view je editovatelný (TextEdit), ne read-only LayoutJob. Uživatel může přímo upravovat aktuální verzi souboru.
- Why it matters: Uživatel potřebuje porovnat historii a zároveň editovat — přepínání mezi history view a editorem je nepraktické.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: TextEdit::multiline s layouterem v history/mod.rs:526. cargo check + 195 testů pass. Vizuální UAT pending (headless).
- Notes: Plně funkční TextEdit se syntax highlighting a diff overlay.

### R002 — Syntax highlighting v obou panelech
- Class: primary-user-loop
- Status: validated
- Description: Oba panely history split view mají syntax highlighting přes syntect — stejný jako normální editor.
- Why it matters: Bez syntax highlighting je kód nečitelný, zvlášť při porovnávání verzí.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: Highlighter::highlight() v obou panelech — levý přes TextEdit layouter, pravý přes Label+LayoutJob. cargo check čistý.
- Notes: Syntax highlighting se kombinuje s diff barvami — normální řádky plná syntaxe, diff řádky mají diff pozadí + syntax barvy textu.

### R003 — Synchronizovaný scroll obou panelů
- Class: primary-user-loop
- Status: validated
- Description: Rolování jedním panelem automaticky roluje i druhý panel na odpovídající pozici.
- Why it matters: Bez sync scrollu musí uživatel ručně hledat odpovídající místo ve druhém panelu.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: ScrollSource enum + proportionální mapování s epsilon 1.0px (history/mod.rs:615-641). Unit testy pass.
- Notes: Proportionální mapování. Line-based mapování přes Equal řádky odloženo jako potenciální vylepšení.

### R004 — Obnovení historické verze (append, ne replace)
- Class: core-capability
- Status: validated
- Description: Tlačítko "Obnovit" v toolbaru zapíše obsah vybrané historické verze do editoru. Stávající verze mezi obnovenou a poslední se neztratí — nový snapshot se vytvoří jako nejnovější (append na konec fronty).
- Why it matters: Uživatel nechce přijít o mezilehlé verze při obnovení starší. Append zajišťuje kompletní historii.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: none
- Validation: Restore flow v workspace/mod.rs:813-836 — get_snapshot_content → tab.content = historical → take_snapshot (append) → refresh entries. Kompilace + testy pass.
- Notes: Obnovení = zápis obsahu do tab bufferu + vytvoření nového snapshotu + refresh history view.

### R005 — Potvrzovací dialog před obnovením
- Class: failure-visibility
- Status: validated
- Description: Před obnovením historické verze se zobrazí potvrzovací dialog "Opravdu obnovit tuto verzi?" s Ano/Ne.
- Why it matters: Prevence nechtěného přepsání aktuálního obsahu.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: none
- Validation: show_restore_confirm flag + show_modal() confirm dialog (history/mod.rs:373-391). Cancel i confirm cesta implementována. Kompilace čistá.
- Notes: none

### R006 — Editace se propsává zpět do tab bufferu
- Class: primary-user-loop
- Status: validated
- Description: Když uživatel edituje v levém panelu a zavře history view, změny se propsají zpět do tab bufferu a tab se označí jako modified (●).
- Why it matters: Uživatel očekává, že editace v history view se neztrácí.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: workspace/mod.rs:788-795 — content_changed → tab.content = hv_content, tab.modified = true. Průběžný sync každý frame.
- Notes: Editace aktualizuje tab.content průběžně (ne až při zavření), autosave funguje.

### R007 — Výchozí stav panelů podle počtu verzí
- Class: primary-user-loop
- Status: validated
- Description: Pokud existuje jen jedna verze (originál, žádná historie), pravý panel je prázdný. Pokud existuje historie (>1 verze), pravý panel automaticky zobrazí nejnovější historickou verzi.
- Why it matters: Srozumitelný výchozí stav — uživatel nevidí zbytečné "žádné verze" a zároveň vidí nejrelevantnější porovnání.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: workspace/mod.rs — sel_idx = if entries.len() > 1 { Some(0) } else { None }. Podmíněný výchozí stav.
- Notes: Nejnovější historická = entries[0] (pole je seřazené od nejnovější).

### R008 — i18n klíče pro nové UI prvky
- Class: launchability
- Status: validated
- Description: Všechny nové UI texty (tlačítko Obnovit, potvrzovací dialog, stav prázdného panelu) mají i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru).
- Why it matters: Editor je vícejazyčný — nové prvky nesmí být hardcoded.
- Source: inferred
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: grep -c 'history-restore' locales/*/ui.ftl → 5 klíčů × 5 jazyků potvrzeno.
- Notes: none

### R009 — Diff zvýraznění v obou panelech se syntax highlighting
- Class: primary-user-loop
- Status: validated
- Description: Diff zvýraznění (přidané/odebrané řádky, zelená/červená) funguje v obou panelech společně se syntax highlighting. Normální řádky mají plnou syntaxi, diff řádky mají diff pozadí + syntax barvy textu.
- Why it matters: Bez diff zvýraznění je porovnávání verzí nepoužitelné. Kombinace s highlighting zajišťuje čitelnost.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: apply_diff_backgrounds() + Highlighter::highlight() v layouter closure. Oba panely. cargo check + 195 testů pass.
- Notes: Pokračuje v patternu z M002, nyní se kombinuje se syntect highlighting v obou panelech.

## Deferred

(none)

## Out of Scope

### R100 — Editace historické verze
- Class: anti-feature
- Status: out-of-scope
- Description: Historická verze v pravém panelu zůstává read-only. Nelze ji editovat.
- Why it matters: Prevence zmatku — historie je immutable referenční bod.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: Pokud uživatel chce obsah historické verze, použije "Obnovit".

### R101 — Restore jako samostatný soubor
- Class: anti-feature
- Status: out-of-scope
- Description: Obnovení historické verze přepisuje aktuální obsah, nevytváří nový soubor.
- Why it matters: Jednodušší UX — "obnovit" = nahradit obsah, ne duplikovat.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: none

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | primary-user-loop | validated | M003/S01 | none | TextEdit+layouter, cargo check + 195 testů |
| R002 | primary-user-loop | validated | M003/S01 | none | Highlighter::highlight() oba panely |
| R003 | primary-user-loop | validated | M003/S01 | none | ScrollSource + proportionální mapování |
| R004 | core-capability | validated | M003/S02 | none | restore flow end-to-end, take_snapshot append |
| R005 | failure-visibility | validated | M003/S02 | none | show_modal() confirm dialog |
| R006 | primary-user-loop | validated | M003/S01 | none | content_changed → tab sync |
| R007 | primary-user-loop | validated | M003/S01 | none | podmíněný selected_index |
| R008 | launchability | validated | M003/S02 | M003/S01 | 5 klíčů × 5 jazyků |
| R009 | primary-user-loop | validated | M003/S01 | none | apply_diff_backgrounds + highlight |
| R100 | anti-feature | out-of-scope | none | none | n/a |
| R101 | anti-feature | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 0
- Mapped to slices: 9
- Validated: 9
- Unmapped active requirements: 0
