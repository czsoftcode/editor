# Requirements

Explicitní capability contract pro projekt PolyCredo Editor.

## Active

### R016 — Regex search engine s togglery
- Class: core-capability
- Status: validated
- Description: Search engine s `RegexBuilder`, case-insensitive/sensitive toggle, whole-word toggle. Nevalidní regex pattern vrací inline chybu.
- Why it matters: Table stakes pro programátorský editor — substring match nestačí.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: build_regex() zvládá všech 8 kombinací regex/case/whole-word. 10 unit testů pokrývají všechny kombinace + empty query + invalid pattern. Nevalidní regex vrací Err(String) s prefixem "Neplatný regex:".
- Notes: `regex` crate již v Cargo.toml.

### R017 — Zvýrazněné matchující části ve výsledcích
- Class: primary-user-loop
- Status: validated
- Description: Výsledky project search zobrazují zvýrazněné matchující části textu přes LayoutJob s barevnými TextSection.
- Why it matters: Bez zvýraznění uživatel nevidí kde přesně v řádku je match.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: build_match_layout_job() renderuje match ranges s oranžovým background (rgba(200,130,0,120)). build_context_layout_job() renderuje kontext dim barvou (rgb(140,140,140)). cargo check čistý.
- Notes: Pattern z diff_view.rs a history/mod.rs.

### R018 — Kontextové řádky se sloučením blízkých matchů
- Class: primary-user-loop
- Status: validated
- Description: Výsledky zobrazují ±2 řádky kontextu kolem matchujícího řádku. Blízké matche (≤4 řádky) se slučují do jednoho bloku.
- Why it matters: Standard v grep/VS Code — uživatel vidí kde v kódu se match nachází.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: search_file_with_context() generuje kontext ±2 řádky, sloučení blízkých matchů (distance ≤ 2*context_lines). 3 unit testy (simple match, close matches merged, no match). UI separátor ··· mezi nesouvisejícími bloky.
- Notes: none

### R019 — File type filtr (glob pattern)
- Class: primary-user-loop
- Status: validated
- Description: Uživatel může omezit search scope zadáním glob patternu (např. `*.rs`, `*.toml`, `src/**/*.rs`). Filtrování přes `globset` crate.
- Why it matters: Omezení scope hledání na relevantní soubory.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: globset::Glob file filter v run_project_search() matchuje filename i celou relativní cestu. 2 unit testy (glob matches, glob no match). Nevalidní glob → SearchBatch::Error → toast.
- Notes: `globset` již v Cargo.toml.

### R020 — Project-wide replace s preview a per-file checkboxy
- Class: primary-user-loop
- Status: active
- Description: Uživatel zadá replace text, zobrazí se preview diff všech náhrad per-file s checkboxy. Potvrzení provede nahrazení ve vybraných souborech.
- Why it matters: Destruktivní operace vyžaduje preview a kontrolu.
- Source: user
- Primary owning slice: M005/S02
- Supporting slices: M005/S01
- Validation: pending
- Notes: none

### R021 — Regex error zobrazený inline v dialogu
- Class: failure-visibility
- Status: validated
- Description: Nevalidní regex pattern zobrazí chybovou zprávu inline pod inputem v dialogu. Hledání se nespustí.
- Why it matters: Ne panic, ne toast — inline feedback pro okamžitou opravu.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: build_regex() vrací Err(String) s prefixem "Neplatný regex:" pro nevalidní pattern, "Prázdný vyhledávací dotaz" pro prázdný query. UI zobrazuje regex_error inline červeným textem pod inputem. Unit testy build_regex_invalid_pattern a build_regex_empty_query ověřují.
- Notes: `RegexBuilder::build()` vrací `Err(regex::Error)` s popisnou zprávou.

### R022 — Replace I/O error reporting přes toast per-file
- Class: failure-visibility
- Status: active
- Description: Pokud zápis jednoho souboru při replace selže, chyba se reportuje přes toast a replace pokračuje s dalšími soubory.
- Why it matters: Per-file error handling místo atomic all-or-nothing. Uživatel vidí které soubory selhaly.
- Source: user
- Primary owning slice: M005/S02
- Supporting slices: none
- Validation: pending
- Notes: Konzistentní s S-3 (I/O error propagace do UI).

### R023 — Local history snapshot před replace
- Class: core-capability
- Status: active
- Description: Před modifikací každého souboru při replace se vytvoří local history snapshot přes `take_snapshot()`.
- Why it matters: Záchranná síť pro undo destruktivní operace.
- Source: user
- Primary owning slice: M005/S02
- Supporting slices: none
- Validation: pending
- Notes: `take_snapshot()` potřebuje `&mut LocalHistory` — volání v workspace handleru.

### R024 — i18n pro všechny nové UI prvky (5 jazyků)
- Class: launchability
- Status: active
- Description: Všechny nové UI texty (toggle labely, replace UI, error messages, context labels) mají i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru).
- Why it matters: Konzistence s existujícím i18n systémem.
- Source: inferred
- Primary owning slice: M005/S01
- Supporting slices: M005/S02
- Validation: S01 dodal 21 klíčů s prefixem project-search-* × 5 jazyků (grep ověřeno). Zbývají replace-specifické klíče v S02.
- Notes: none

### R025 — Inkrementální streamování výsledků
- Class: primary-user-loop
- Status: validated
- Description: Výsledky se streamují inkrementálně (po dávkách per-soubor), ne jednorázově po dokončení celého searche.
- Why it matters: UX — uživatel vidí výsledky ihned, ne po sekundách čekání.
- Source: user
- Primary owning slice: M005/S01
- Supporting slices: none
- Validation: SearchBatch enum (Results/Done/Error) přes mpsc s per-soubor dávkováním. UI akumuluje výsledky přes try_recv() loop. Spinner + "Hledám..." indikátor během searching == true. cargo check čistý.
- Notes: Rozšíření existujícího mpsc pattern z `run_project_search()`.

## Validated

### R013 — Uživatelská konfigurace keybindings
- Class: primary-user-loop
- Status: validated
- Description: Uživatel může v `[keybindings]` sekci settings.toml přemapovat zkratky na jiné klávesové kombinace. Chybějící sekce = default bindings.
- Why it matters: Různí uživatelé mají různé preference a návyky z jiných editorů.
- Source: user
- Primary owning slice: M004/S03
- Supporting slices: M004/S01, M004/S02
- Validation: `keybindings: HashMap<String, String>` s `#[serde(default)]` v Settings. apply_keybinding_overrides() s validací (reserved, invalid, conflict, unknown id, empty string). Init + save wiring. 10 unit testů pro override logiku + 2 backward compat testy. Menu/palette labely reflektují overrides automaticky.
- Notes: none

### R015 — Sjednocení s VS Code / JetBrains konvencemi
- Class: primary-user-loop
- Status: validated
- Description: Defaultní keybindings odpovídají konvencím VS Code / JetBrains (Ctrl+Shift+P command palette, Ctrl+Tab přepínání tabů, Ctrl+F find, Ctrl+H replace, atd.). Chybějící standardní zkratky jsou doplněny.
- Why it matters: Uživatelé přecházející z jiných editorů očekávají známé zkratky.
- Source: user
- Primary owning slice: M004
- Supporting slices: none
- Validation: S01 centrální dispatch + exkluzivní modifier matching. S02 doplnil Ctrl+F/H/G/Shift+P/F1 konvence, command palette, menu napojení. S03 uživatelská konfigurovatelnost přes [keybindings] sekci. Všechny standardní zkratky implementovány a konfigurovatelné.
- Notes: none

### R012 — Chybějící keyboard handlery
- Class: primary-user-loop
- Status: validated
- Description: Všechny zkratky zobrazené v menu a command palette mají funkční keyboard handler — Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+P, Ctrl+Shift+F, Ctrl+Shift+P.
- Why it matters: Menu zobrazuje zkratky, které ve skutečnosti nefungují — matoucí UX.
- Source: user
- Primary owning slice: M004/S02
- Supporting slices: M004/S01
- Validation: 4 nové CommandId varianty (Find, Replace, GotoLine, CommandPalette) v dispatch pipeline. 5 nových command registrací včetně F1. 4 nové unit testy (test_dispatch_new_commands, test_dispatch_command_palette_ordering). Menu edit.rs napojení na flagy. 13/13 keymap testů pass.
- Notes: none

### R010 — Centrální keymap dispatch
- Class: core-capability
- Status: validated
- Description: Všechny klávesové zkratky procházejí centrálním dispatch systémem napojeným na command registry. Žádné ad-hoc `ctx.input()` handlery roztroušené po kódu.
- Why it matters: Údržba, konzistence, konfigurovatelnost — přidání nové zkratky nesmí vyžadovat editaci 3+ souborů.
- Source: user
- Primary owning slice: M004/S01
- Supporting slices: none
- Validation: Keymap::dispatch() v render_workspace(), 9 unit testů, grep na absenci ad-hoc handlerů (0 výskytů "i.modifiers.ctrl" v workspace/mod.rs). 156 testů pass.
- Notes: none

### R011 — Exkluzivní modifier matching
- Class: core-capability
- Status: validated
- Description: Ctrl+B matchne pouze Ctrl+B, ne Ctrl+Alt+B ani Ctrl+Shift+B. Trojkombinace nespouští dvoukombinace.
- Why it matters: Současný kód spouští cargo build i při Ctrl+Alt+B (focus build panel).
- Source: user
- Primary owning slice: M004/S01
- Supporting slices: none
- Validation: test_dispatch_ordering unit test — Ctrl+Alt+B → FocusBuild (ne Build). Bindings seřazeny sestupně dle modifier_count().
- Notes: none

### R014 — Cross-platform Ctrl↔Cmd
- Class: launchability
- Status: validated
- Description: Na macOS se místo Ctrl používá Cmd pro všechny zkratky. Editor automaticky mapuje Ctrl↔Cmd dle platformy.
- Why it matters: macOS uživatelé očekávají Cmd, ne Ctrl.
- Source: user
- Primary owning slice: M004/S01
- Supporting slices: none
- Validation: Modifiers::COMMAND použit ve všech registracích. parse_shortcut mapuje "Ctrl"/"Cmd" → COMMAND. format_shortcut() wrapper přes egui s platform-aware výstupem. Unit testy pass.
- Notes: none

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
| R010 | core-capability | validated | M004/S01 | none | Keymap dispatch, 9 unit testů, 0 ad-hoc handlerů |
| R011 | core-capability | validated | M004/S01 | none | test_dispatch_ordering, modifier_count řazení |
| R012 | primary-user-loop | validated | M004/S02 | M004/S01 | 4 nové CommandId, 5 command registrací, 4 unit testy, menu flagy, 13/13 pass |
| R013 | primary-user-loop | validated | M004/S03 | M004/S01, M004/S02 | apply_keybinding_overrides(), 10 unit testů, backward compat, menu/palette labely |
| R014 | launchability | validated | M004/S01 | none | Modifiers::COMMAND, parse_shortcut Ctrl/Cmd→COMMAND |
| R015 | primary-user-loop | validated | M004 | none | S01 dispatch + S02 konvence + S03 konfigurovatelnost |
| R016 | core-capability | validated | M005/S01 | none | build_regex() 10 unit testů, všech 8 kombinací |
| R017 | primary-user-loop | validated | M005/S01 | none | LayoutJob multi-section s oranžovým bg |
| R018 | primary-user-loop | validated | M005/S01 | none | search_file_with_context() 3 unit testy, sloučení |
| R019 | primary-user-loop | validated | M005/S01 | none | globset filtr 2 unit testy, filename+path |
| R020 | primary-user-loop | active | M005/S02 | M005/S01 | pending |
| R021 | failure-visibility | validated | M005/S01 | none | inline regex error, 2 unit testy |
| R022 | failure-visibility | active | M005/S02 | none | pending |
| R023 | core-capability | active | M005/S02 | none | pending |
| R024 | launchability | active | M005/S01 | M005/S02 | S01: 21 klíčů × 5 jazyků, S02 doplní replace |
| R025 | primary-user-loop | validated | M005/S01 | none | SearchBatch enum, per-soubor dávkování |
| R100 | anti-feature | out-of-scope | none | none | n/a |
| R101 | anti-feature | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 4 (R020, R022, R023, R024)
- Mapped to slices: 28
- Validated: 21 (R001–R019, R021, R025)
- Unmapped active requirements: 0
