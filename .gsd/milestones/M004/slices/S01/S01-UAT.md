# S01: Centrální keymap dispatch a oprava modifier filtrování — UAT

**Milestone:** M004
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed
- Why this mode is sufficient: Kontrakt (dispatch ordering, parse/format, modifier exkluzivita) je ověřen 9 unit testy. Integrační wiring (ad-hoc handlery smazány, dispatch napojen) je ověřen grepem + Ctrl+W testem. GUI interakce (focus panelů, build nespuštění při Ctrl+Alt+B) vyžaduje manuální ověření v živém editoru.

## Preconditions

- `./check.sh` projde čistě (156 testů, 0 warningů)
- Editor je spuštěný (`cargo run`)
- Otevřený alespoň jeden soubor v editoru
- Build terminál a AI panel dostupné (viditelné nebo skrývatelné)

## Smoke Test

1. Stiskni Ctrl+S v editoru s otevřeným souborem
2. **Expected:** Soubor se uloží (tab ztratí ● indikátor modified stavu) — ověřuje, že nejzákladnější zkratka funguje přes centrální dispatch

## Test Cases

### 1. Ctrl+B spustí build

1. Otevři editor s projektem
2. Stiskni Ctrl+B
3. **Expected:** Spustí se cargo build/check — build terminál se aktivuje, zobrazí build output

### 2. Ctrl+Alt+B přepne fokus na build panel BEZ spuštění buildu

1. Otevři editor s projektem
2. Ujisti se, že build NEBĚŽÍ
3. Stiskni Ctrl+Alt+B
4. **Expected:** Fokus se přepne na build panel (build terminál se zobrazí). Cargo build se NESPUSTÍ — žádný nový build output. Toto je klíčový test pro R011 (exkluzivní modifier matching).

### 3. Ctrl+Alt+E přepne fokus na editor

1. Klikni do build panelu (fokus mimo editor)
2. Stiskni Ctrl+Alt+E
3. **Expected:** Fokus se přepne zpět na editor panel

### 4. Ctrl+Alt+A přepne fokus na AI panel

1. Stiskni Ctrl+Alt+A
2. **Expected:** AI panel se zobrazí a fokus se na něj přepne

### 5. Ctrl+W zavře tab

1. Otevři soubor, ujisti se že tab je uložený (bez ●)
2. Stiskni Ctrl+W
3. **Expected:** Tab se zavře

### 6. Ctrl+R spustí run

1. Stiskni Ctrl+R
2. **Expected:** Spustí se cargo run příkaz

### 7. Ctrl+, otevře settings

1. Stiskni Ctrl+, (čárka)
2. **Expected:** Otevře se settings dialog/panel

### 8. Menu shortcut labely jsou dynamické

1. Otevři File menu v menubaru
2. Podívej se na shortcut label u "Save"
3. **Expected:** Label zobrazuje "Ctrl+S" (na Linuxu) — generovaný z Keymap dat, ne hardcoded string
4. Otevři Edit menu
5. **Expected:** Build a Run shortcut labely jsou také dynamicky generované

### 9. Copy/Paste/Undo fungují v editoru

1. Napiš text do editoru
2. Vyber text, stiskni Ctrl+C
3. Klikni jinam, stiskni Ctrl+V
4. Stiskni Ctrl+Z
5. **Expected:** Copy, paste a undo fungují normálně — centrální dispatch je NEINTERCEPTOVAL (Ctrl+A/C/V/X/Z/Y nejsou v keymapu)

## Edge Cases

### Ctrl+Alt+B při běžícím buildu

1. Spusť build (Ctrl+B)
2. Zatímco build běží, stiskni Ctrl+Alt+B
3. **Expected:** Fokus se přepne na build panel. Nový build se NESPUSTÍ — běží jen původní build z kroku 1.

### Ctrl+W na modifikovaném tabu

1. Otevři soubor, uprav ho (tab zobrazí ●)
2. Stiskni Ctrl+W
3. **Expected:** Zobrazí se unsaved changes guard dialog — ne okamžité zavření

### Rychlé sekvenční stisky

1. Rychle za sebou stiskni Ctrl+B, pak okamžitě Ctrl+Alt+E
2. **Expected:** Build se spustí, pak se fokus přepne na editor. Žádné záměny příkazů.

### Neznámá klávesová kombinace

1. Stiskni Ctrl+Alt+Shift+K (není registrovaná)
2. **Expected:** Nic se nestane — žádný crash, žádná chyba. Dispatch vrátí None.

## Failure Signals

- Ctrl+Alt+B spustí cargo build → R011 (exkluzivní modifier matching) je broken — dispatch neřadí bindings správně
- Ctrl+S/W/B nefungují → dispatch není napojen v render_workspace() nebo execute_command chybí handling
- Copy/Paste nefungují v editoru → Ctrl+C/V jsou omylem registrovány v keymapu
- Menu shortcut labely zobrazují "Ctrl+S" místo "⌘S" na macOS → format_shortcut nerespektuje platformu
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` vrací > 0 → ad-hoc handlery nebyly smazány

## Requirements Proved By This UAT

- R010 — centrální dispatch: test cases 1-7 ověřují že všechny zkratky fungují přes centrální Keymap
- R011 — exkluzivní modifier matching: test case 2 + edge case "Ctrl+Alt+B při běžícím buildu" ověřují že trojkombinace nespouští dvoukombinaci
- R014 — cross-platform: test case 8 (label generování) ověřuje format_shortcut; unit testy pokrývají Ctrl/Cmd→COMMAND mapování
- R015 — VS Code konvence: test cases 1, 5, 7 ověřují standardní VS Code zkratky (Ctrl+S, Ctrl+W, Ctrl+,)

## Not Proven By This UAT

- R012 (chybějící handlery — Ctrl+F, H, G, Shift+P, Shift+F) — scope S02
- R013 (uživatelská konfigurace keybindings) — scope S03
- macOS Cmd behavior — vyžaduje macOS hardware/VM pro manuální ověření
- Command palette shortcut labely — S02 oživí command palette

## Notes for Tester

- Focus panel trojkombinace (Ctrl+Alt+E/B/A) jsou nové — dříve neexistovaly nebo nefungovaly správně kvůli modifier filtrování
- Klíčový test je **test case 2** (Ctrl+Alt+B) — toto byl hlavní bug motivující celý M004 milestone
- i18n klíče pro focus commandy (`command-name-focus-*`) ještě neexistují — command palette může zobrazit fallback text
- Editor-interní zkratky (Ctrl+F, H, G) nejsou zatím v keymapu — jejich handlery přidá S02
