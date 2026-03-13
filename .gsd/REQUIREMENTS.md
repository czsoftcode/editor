# Requirements

Explicitní capability contract pro projekt PolyCredo Editor.

## Active

### R001 — Editovatelný levý panel v history view
- Class: primary-user-loop
- Status: active
- Description: Levý panel v history split view je editovatelný (TextEdit), ne read-only LayoutJob. Uživatel může přímo upravovat aktuální verzi souboru.
- Why it matters: Uživatel potřebuje porovnat historii a zároveň editovat — přepínání mezi history view a editorem je nepraktické.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending
- Notes: Musí být plně funkční TextEdit se syntax highlighting, ne textový vstup bez formátování.

### R002 — Syntax highlighting v obou panelech
- Class: primary-user-loop
- Status: active
- Description: Oba panely history split view mají syntax highlighting přes syntect — stejný jako normální editor.
- Why it matters: Bez syntax highlighting je kód nečitelný, zvlášť při porovnávání verzí.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending
- Notes: Syntax highlighting se kombinuje s diff barvami — normální řádky plná syntaxe, diff řádky mají diff pozadí + syntax barvy textu.

### R003 — Synchronizovaný scroll obou panelů
- Class: primary-user-loop
- Status: active
- Description: Rolování jedním panelem automaticky roluje i druhý panel na odpovídající pozici.
- Why it matters: Bez sync scrollu musí uživatel ručně hledat odpovídající místo ve druhém panelu.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending — proportionální mapování místo line-based
- Notes: Implementováno přes proportionální mapování s epsilon tolerancí. Line-based mapování přes Equal řádky odloženo jako potenciální vylepšení.

### R004 — Obnovení historické verze (append, ne replace)
- Class: core-capability
- Status: active
- Description: Tlačítko "Obnovit" v toolbaru zapíše obsah vybrané historické verze do editoru. Stávající verze mezi obnovenou a poslední se neztratí — nový snapshot se vytvoří jako nejnovější (append na konec fronty).
- Why it matters: Uživatel nechce přijít o mezilehlé verze při obnovení starší. Append zajišťuje kompletní historii.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: none
- Validation: implemented (S02), UAT pending — restore flow propojený od UI po workspace handling, kompilace + testy pass
- Notes: Obnovení = zápis obsahu do tab bufferu + vytvoření nového snapshotu + refresh history view.

### R005 — Potvrzovací dialog před obnovením
- Class: failure-visibility
- Status: active
- Description: Před obnovením historické verze se zobrazí potvrzovací dialog "Opravdu obnovit tuto verzi?" s Ano/Ne.
- Why it matters: Prevence nechtěného přepsání aktuálního obsahu.
- Source: user
- Primary owning slice: M003/S02
- Supporting slices: none
- Validation: implemented (S02), UAT pending — show_modal() confirm dialog integrován do restore flow
- Notes: none

### R006 — Editace se propsává zpět do tab bufferu
- Class: primary-user-loop
- Status: active
- Description: Když uživatel edituje v levém panelu a zavře history view, změny se propsají zpět do tab bufferu a tab se označí jako modified (●).
- Why it matters: Uživatel očekává, že editace v history view se neztrácí.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending — průběžný sync přes HistorySplitResult.content_changed
- Notes: Editace aktualizuje tab.content průběžně (ne až při zavření), autosave funguje.

### R007 — Výchozí stav panelů podle počtu verzí
- Class: primary-user-loop
- Status: active
- Description: Pokud existuje jen jedna verze (originál, žádná historie), pravý panel je prázdný. Pokud existuje historie (>1 verze), pravý panel automaticky zobrazí nejnovější historickou verzi.
- Why it matters: Srozumitelný výchozí stav — uživatel nevidí zbytečné "žádné verze" a zároveň vidí nejrelevantnější porovnání.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending — selected_index podmíněný na entries.len()
- Notes: Nejnovější historická = entries[0] (pole je seřazené od nejnovější).

### R008 — i18n klíče pro nové UI prvky
- Class: launchability
- Status: active
- Description: Všechny nové UI texty (tlačítko Obnovit, potvrzovací dialog, stav prázdného panelu) mají i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru).
- Why it matters: Editor je vícejazyčný — nové prvky nesmí být hardcoded.
- Source: inferred
- Primary owning slice: M003/S02
- Supporting slices: M003/S01
- Validation: implemented (S02), UAT pending — `grep 'history-restore' locales/*/ui.ftl` potvrzuje 5 klíčů × 5 jazyků
- Notes: none

### R009 — Diff zvýraznění v obou panelech se syntax highlighting
- Class: primary-user-loop
- Status: active
- Description: Diff zvýraznění (přidané/odebrané řádky, zelená/červená) funguje v obou panelech společně se syntax highlighting. Normální řádky mají plnou syntaxi, diff řádky mají diff pozadí + syntax barvy textu.
- Why it matters: Bez diff zvýraznění je porovnávání verzí nepoužitelné. Kombinace s highlighting zajišťuje čitelnost.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: implemented (S01), UAT pending — apply_diff_backgrounds() + Highlighter::highlight() v layouter closure
- Notes: Pokračuje v patternu z M002, nyní se kombinuje se syntect highlighting v obou panelech.

## Validated

(none yet)

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
| R001 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R002 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R003 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R004 | core-capability | active | M003/S02 | none | implemented, UAT pending |
| R005 | failure-visibility | active | M003/S02 | none | implemented, UAT pending |
| R006 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R007 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R008 | launchability | active | M003/S02 | M003/S01 | implemented, UAT pending |
| R009 | primary-user-loop | active | M003/S01 | none | implemented, UAT pending |
| R100 | anti-feature | out-of-scope | none | none | n/a |
| R101 | anti-feature | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 9
- Mapped to slices: 9
- Validated: 0
- Unmapped active requirements: 0
