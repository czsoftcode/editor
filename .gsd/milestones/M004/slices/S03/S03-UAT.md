# S03: Uživatelská konfigurace keybindings a dynamické labely — UAT

**Milestone:** M004
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven unit testy + live-runtime GUI ověření)
- Why this mode is sufficient: Override logika a validace pokryta 12 unit testy. GUI labely (menu, palette) vyžadují vizuální ověření v běžícím editoru.

## Preconditions

- Editor zkompilován (`cargo build --release` nebo `cargo run`)
- `./check.sh` projde čistě (172+ testů)
- Existující `settings.toml` v config adresáři (nebo editor vytvoří default)
- Žádná `[keybindings]` sekce v settings.toml (výchozí stav)

## Smoke Test

1. Spustit editor bez `[keybindings]` sekce v settings.toml
2. Stisknout Ctrl+S — soubor se uloží
3. **Expected:** Editor uloží soubor, menu zobrazuje "Ctrl+S" (nebo "⌘S" na macOS) u položky Save

## Test Cases

### 1. Základní override — přemapování Save

1. Zavřít editor
2. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Ctrl+Shift+S"
   ```
3. Spustit editor, otevřít soubor, provést změnu
4. Stisknout Ctrl+Shift+S
5. **Expected:** Soubor se uloží. Menu položka Save zobrazuje "Ctrl+Shift+S" místo "Ctrl+S"
6. Stisknout Ctrl+S
7. **Expected:** Nic se nestane — Ctrl+S již není bindován na Save

### 2. Command palette reflektuje override

1. S override z testu 1 aktivním, otevřít command palette (Ctrl+Shift+P nebo F1)
2. Najít příkaz "Save" / "Uložit" v paletě
3. **Expected:** Vedle příkazu je zobrazen "Ctrl+Shift+S" (ne "Ctrl+S")

### 3. Backward compat — žádná [keybindings] sekce

1. Odebrat celou `[keybindings]` sekci ze settings.toml
2. Spustit editor
3. **Expected:** Všechny zkratky fungují s defaultními bindings (Ctrl+S uloží, Ctrl+F otevře search, atd.). Žádné chyby v konzoli.

### 4. Override přes Settings dialog

1. Otevřít Settings dialog (menu nebo zkratka)
2. Ručně editovat settings.toml a přidat `[keybindings]` s `"editor.save" = "Ctrl+Shift+S"`
3. Uložit settings v dialogu
4. **Expected:** Override se aplikuje okamžitě — Ctrl+Shift+S uloží, menu label se aktualizuje. Není nutný restart editoru.

### 5. Prázdný override odstraní shortcut

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = ""
   ```
2. Spustit editor
3. **Expected:** Save nemá žádnou klávesovou zkratku. Menu položka Save nemá shortcut label. Uložit lze pouze přes menu kliknutí.

### 6. Více overrides najednou

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Ctrl+Shift+S"
   "editor.close_tab" = "Ctrl+Shift+W"
   "build.build" = "F5"
   ```
2. Spustit editor
3. **Expected:** Všechny tři zkratky přemapovány. Menu labely reflektují nové bindings.

## Edge Cases

### Nevalidní shortcut string

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Foo+Bar"
   ```
2. Spustit editor
3. **Expected:** Override se ignoruje, Save zůstane na Ctrl+S. V stderr se objeví `[keybinding] invalid shortcut for editor.save: Foo+Bar`.

### Neexistující command id

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "nonexistent.command" = "Ctrl+Shift+X"
   ```
2. Spustit editor
3. **Expected:** Override se ignoruje, žádný crash. V stderr: `[keybinding] unknown command: nonexistent.command`.

### Reserved key override (Ctrl+C)

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Ctrl+C"
   ```
2. Spustit editor
3. **Expected:** Override se odmítne, Save zůstane na Ctrl+S. Ctrl+C stále kopíruje text v editoru. V stderr: `[keybinding] reserved key for editor.save: Ctrl+C`.

### Ctrl+Shift+C (ne reserved)

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Ctrl+Shift+C"
   ```
2. Spustit editor
3. **Expected:** Override se aplikuje — Ctrl+Shift+C uloží. Ctrl+C stále kopíruje (reserved je jen přesně Ctrl+C bez dalších modifikátorů).

### Konflikt — dvě akce na stejnou zkratku

1. Přidat do settings.toml:
   ```toml
   [keybindings]
   "editor.save" = "Ctrl+Shift+X"
   "editor.close_tab" = "Ctrl+Shift+X"
   ```
2. Spustit editor
3. **Expected:** Obě overrides se aplikují (warning-only). V stderr: `[keybinding] conflict: Ctrl+Shift+X already assigned to editor.save, overriding for editor.close_tab`. Chování závisí na iteraci — poslední override „vyhraje" v dispatch.

## Failure Signals

- Menu nebo palette zobrazují defaultní shortcut label po nastavení override → override se neaplikoval nebo keymap se nerebuildla
- Override na reserved klávesu (Ctrl+C) projde → is_reserved_shortcut() nefunguje
- Nevalidní shortcut string crashne editor → chybí validace v apply_keybinding_overrides()
- Settings bez `[keybindings]` sekce crashne deserializaci → chybí `#[serde(default)]`
- Po uložení settings v dialogu se override neprojeví → save flow nevolá apply_overrides + keymap rebuild

## Requirements Proved By This UAT

- R013 — uživatelská konfigurace keybindings: testy 1, 2, 3, 4, 5, 6 + edge cases pokrývají celý scope
- R015 — sjednocení s VS Code / JetBrains konvencemi: defaultní bindings + konfigurovatelnost

## Not Proven By This UAT

- Cross-platform Cmd↔Ctrl chování na macOS (testováno na Linux, macOS mapping přes Modifiers::COMMAND ověřen unit testy v S01)
- Vizuální rendering shortcut labelů v různých OS — fonty a glyphy (⌘/⇧) závisí na platformě

## Notes for Tester

- Warningy v stderr: spustit editor z terminálu (`cargo run`) pro viditelnost `[keybinding]` warningů
- Config cesta: settings.toml je typicky v `~/.config/polycredo-editor/settings.toml` (Linux) nebo `~/Library/Application Support/polycredo-editor/settings.toml` (macOS)
- Conflict detection je warning-only, ne hard error — toto je záměrné chování (ne bug)
- Reserved keys zahrnují přesně Ctrl+A/C/V/X/Z/Y (bez dalších modifikátorů). Ctrl+Shift+C/V/A NEJSOU reserved.
