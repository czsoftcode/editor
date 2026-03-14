# S02: Chybějící keyboard handlery a oživení command palette — UAT

**Milestone:** M004
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven unit testy + live-runtime GUI ověření)
- Why this mode is sufficient: Dispatch logika je pokryta 13 unit testy. GUI chování (otevření search baru, command palette toggle, menu kliknutí) vyžaduje manuální ověření v běžící aplikaci.

## Preconditions

- `cargo build` kompiluje bez chyb
- `./check.sh` projde čistě (fmt + clippy + všechny testy)
- Editor spuštěn s otevřeným souborem (libovolný .rs soubor) v hlavním editoru

## Smoke Test

1. Stiskni **Ctrl+Shift+P** (nebo Cmd+Shift+P na macOS).
2. **Expected:** Command palette se otevře. Obsahuje seznam příkazů. Stisknutí Ctrl+Shift+P znovu palette zavře.

## Test Cases

### 1. Ctrl+F otevře in-file search bar

1. Otevři libovolný soubor v editoru.
2. Stiskni **Ctrl+F**.
3. **Expected:** Search bar se zobrazí s fokusem na textovém inputu. Kurzor bliká v search fieldu.
4. Začni psát hledaný text.
5. **Expected:** Text se zobrazuje v search fieldu.

### 2. Ctrl+F při otevřeném search baru refocusne input

1. Stiskni **Ctrl+F** — search bar se otevře.
2. Klikni někam do editoru (search bar zůstane viditelný, ale focus je v editoru).
3. Stiskni **Ctrl+F** znovu.
4. **Expected:** Search bar zůstane otevřený a focus se vrátí do search inputu (ne toggle off).

### 3. Ctrl+H otevře search + replace

1. Stiskni **Ctrl+H**.
2. **Expected:** Search bar se zobrazí s replace polem viditelným. Focus je v search inputu.

### 4. Ctrl+G otevře goto line

1. Stiskni **Ctrl+G**.
2. **Expected:** Goto line dialog/bar se zobrazí s fokusem na číslo řádku.

### 5. Ctrl+Shift+P otevře command palette

1. Stiskni **Ctrl+Shift+P**.
2. **Expected:** Command palette se otevře. Zobrazuje seznam příkazů s jejich aktuálními shortcut labely.
3. Ověř, že příkaz "Find" zobrazuje "Ctrl+F" (nebo "⌘F" na macOS).
4. Ověř, že příkaz "Command Palette" zobrazuje "Ctrl+Shift+P" (nebo "⌘⇧P" na macOS).

### 6. F1 otevře command palette

1. Zavři command palette (Escape nebo Ctrl+Shift+P).
2. Stiskni **F1**.
3. **Expected:** Command palette se otevře — stejné chování jako Ctrl+Shift+P.

### 7. Command palette toggle

1. Stiskni **Ctrl+Shift+P** — palette se otevře.
2. Stiskni **Ctrl+Shift+P** znovu.
3. **Expected:** Palette se zavře.
4. Stiskni **F1** — palette se otevře.
5. Stiskni **Ctrl+Shift+P**.
6. **Expected:** Palette se zavře (cross-binding toggle funguje).

### 8. Menu Edit → Find funguje

1. Klikni na menu **Edit**.
2. Klikni na **Find** (nebo lokalizovaný ekvivalent).
3. **Expected:** Menu se zavře a search bar se otevře s fokusem.

### 9. Menu Edit → Replace funguje

1. Klikni na menu **Edit** → **Replace**.
2. **Expected:** Menu se zavře, search bar se otevře s replace polem viditelným.

### 10. Menu Edit → Go to Line funguje

1. Klikni na menu **Edit** → **Go to Line** (nebo lokalizovaný ekvivalent).
2. **Expected:** Menu se zavře, goto line dialog se otevře.

### 11. Menu Edit → Command Palette funguje

1. Klikni na menu **Edit** → **Command Palette**.
2. **Expected:** Menu se zavře, command palette se otevře.

### 12. Dynamické shortcut labely v menu

1. Otevři menu **Edit**.
2. **Expected:** Vedle "Find" je zobrazeno "Ctrl+F" (ne hardcoded string — na macOS by se mělo zobrazit "⌘F").
3. Vedle "Replace" je zobrazeno "Ctrl+H".
4. Vedle "Go to Line" je zobrazeno "Ctrl+G".
5. Vedle "Command Palette" je zobrazeno "Ctrl+Shift+P".

### 13. Ctrl+B stále spouští build (ne conflict s novými zkratkami)

1. Stiskni **Ctrl+B**.
2. **Expected:** Spustí se cargo build. Nové zkratky nerozbily existující.

### 14. Ctrl+Alt+B stále focusne build panel

1. Stiskni **Ctrl+Alt+B**.
2. **Expected:** Fokus se přepne na build panel. Build se NESPUSTÍ.

### 15. Unit testy — artifact-driven verifikace

1. Spusť `cargo test --bin polycredo-editor app::keymap`.
2. **Expected:** 13/13 testů prošlo, včetně:
   - `test_parse_shortcut_f1` — F1 bez modifikátoru parsuje správně
   - `test_parse_shortcut_escape` — Escape bez modifikátoru parsuje správně
   - `test_dispatch_new_commands` — Ctrl+F → Find, Ctrl+H → Replace, Ctrl+G → GotoLine
   - `test_dispatch_command_palette_ordering` — Ctrl+Shift+P → CommandPalette (ne jiný command)

## Edge Cases

### F1 neduplikuje jiný command

1. Stiskni **F1** s otevřeným souborem.
2. **Expected:** Otevře se command palette, NE file dialog, NE help.

### Ctrl+Shift+P vs Ctrl+P

1. Stiskni **Ctrl+Shift+P**.
2. **Expected:** Otevře se command palette (ne open file dialog, pokud Ctrl+P existuje).

### Ctrl+F/H/G nerozbily TextEdit Ctrl+C/V/A

1. V editoru vyber text.
2. Stiskni **Ctrl+C** → **Ctrl+V**.
3. **Expected:** Copy/paste funguje normálně. Centrální dispatch neinterceptuje clipboard zkratky.

### Ctrl+F v prázdném editoru (žádný otevřený soubor)

1. Zavři všechny taby.
2. Stiskni **Ctrl+F**.
3. **Expected:** Buď se nic nestane (žádný editor kde hledat), nebo se search bar otevře bez crash.

## Failure Signals

- Ctrl+F/H/G/Shift+P neotevírá příslušné UI → dispatch pipeline nepropojený
- Command palette se neotvírá → CommandPaletteState::new() selhává nebo toggle logika je chybná
- Menu kliknutí na Find/Replace/GotoLine nemá efekt → actions flagy se nenastavují v edit.rs
- Shortcut labely v menu jsou prázdné → keymap.get_shortcut_for_command() nenachází command
- Raw i18n klíč viditelný v UI (např. "command-name-find") → chybějící překlad v locale souboru
- F1 nedělá nic → parse_shortcut whitelist nefunguje pro standalone klávesy
- Ctrl+B spouští find místo build → dispatch ordering chyba

## Requirements Proved By This UAT

- R012 — Chybějící keyboard handlery: testy 1–11 a 15 dokazují, že všechny nové zkratky mají funkční handlery
- R015 — VS Code/JetBrains konvence: testy 5, 6, 7, 12 dokazují Ctrl+Shift+P, F1, a dynamické labely odpovídají VS Code konvencím

## Not Proven By This UAT

- R013 — Uživatelská konfigurace keybindings: scope S03 — přemapování v settings.toml zatím nefunguje
- R015 plně: Ctrl+Tab přepínání tabů není v scope tohoto slice
- macOS Cmd chování: nelze ověřit na Linux prostředí — závisí na egui Modifiers::COMMAND mapování

## Notes for Tester

- Na Linuxu jsou shortcut labely "Ctrl+F" atd. Na macOS by měly být "⌘F" díky Modifiers::COMMAND a format_shortcut().
- Command palette zobrazuje příkazy z command registry — pokud je registry prázdná (bug), palette bude prázdná.
- i18n klíče závisí na aktuálním jazyce editoru. Testuj v angličtině i češtině pro ověření překladů.
- Pokud goto_line dialog nereaguje, zkontroluj, zda `show_goto_line` flag existuje v WorkspaceState (mohl být přidán v dřívějším milestone).
