# Lokalizace — PolyCredo Editor

Překlady používají formát [Project Fluent](https://projectfluent.org/) (soubory `.ftl`).
Překlady jsou embedovány do binárky pomocí `include_str!` — za runtime nevznikají žádné závislosti na externích souborech.

---

## Struktura

```
locales/
├── cs/          ← čeština (výchozí jazyk)
│   ├── menu.ftl       menu bar a všechny položky menu
│   ├── ui.ftl         obecné prvky UI (panely, tlačítka, status bar, hledání…)
│   ├── dialogs.ftl    dialogy (startup, wizard, potvrzení, O aplikaci…)
│   └── errors.ftl     chybové hlášky a toast info
└── en/          ← angličtina
    ├── menu.ftl
    ├── ui.ftl
    ├── dialogs.ftl
    └── errors.ftl
```

---

## Formát FTL — základy

```fluent
# Komentář

# Jednoduchý klíč
menu-file = Soubor

# Klíč s proměnnou
about-version = Verze { $version }

# Klíč s plurálem (pravidla se liší podle jazyka — Fluent to řeší automaticky)
panel-build-errors =
    { $count ->
        [one] Chyba (1)
        [few] Chyby ({ $count })
       *[other] Chyb ({ $count })
    }
```

Úplná dokumentace: <https://projectfluent.org/fluent/guide/>

---

## Použití v kódu Rust

```rust
// Jednoduchý překlad
let text = i18n.get("menu-file");

// S proměnnými — makro tr!
let text = tr!(i18n, "about-version", version = "0.3.0");

// S proměnnými — manuálně
let mut args = FluentArgs::new();
args.set("count", 5u64);
let text = i18n.get_args("panel-build-errors", &args);
```

Zdrojový kód modulu: `src/i18n.rs`

---

## Přidání nového jazyka

1. Vytvořit složku `locales/{kód}/` (BCP 47, např. `de`, `sk`, `fr`).
2. Zkopírovat všechny `.ftl` soubory z `locales/en/`.
3. Přeložit hodnoty — klíče (levá strana `=`) **nikdy neměnit**.
4. Zaregistrovat jazyk v `src/i18n.rs` — přidat větev do `match lang { … }`.
5. Přidat jazyk do ComboBoxu v Settings panelu.
6. Spustit test pokrytí (viz níže).

---

## Přidání nového překladu (klíče)

1. Vymyslet klíč ve formátu `oblast-podoblast-upřesnění` (kebab-case, bez diakritiky).
2. Přidat klíč do **všech** jazyků najednou — nesynchronizované soubory způsobí viditelný fallback v UI.
3. Použít klíč v kódu přes `i18n.get("nový-klíč")`.

### Konvence pojmenování klíčů

| Prefix | Oblast |
|---|---|
| `menu-` | položky menu baru |
| `panel-` | názvy a popisky panelů |
| `btn-` | tlačítka |
| `statusbar-` | status bar |
| `search-` | hledání a nahrazování |
| `goto-` | dialog Přejít na řádek |
| `file-picker-` | rychlé otevření souboru (Ctrl+P) |
| `project-search-` | hledání napříč projektem (Ctrl+Shift+F) |
| `startup-` | startup dialog |
| `open-project-` | dialog výběru okna při otevírání projektu |
| `wizard-` | průvodce novým projektem |
| `close-project-` | dialog zavření projektu |
| `quit-` | dialog ukončení aplikace |
| `about-` | dialog O aplikaci |
| `confirm-` | obecné potvrzovací dialogy |
| `rename-` | dialog přejmenování |
| `ai-` | AI panel |
| `error-` | chybové hlášky (toast error) |
| `info-` | informační hlášky (toast info) |

---

## Kontrola pokrytí (test)

Každý klíč přítomný v `en/` musí existovat i ve všech ostatních jazycích a naopak.
Test je v `src/i18n.rs` (modul `tests`) — spustit:

```bash
cargo test i18n
```

---

## Roadmap integrace

### Fáze 1 — Napojení `I18n` na stav aplikace ✓ HOTOVO
- [x] Přidat `lang: String` do `Settings` (autodetekce ze systému, fallback `"en"`)
- [x] Přidat `i18n: Arc<I18n>` do `AppShared` (concurrent bundle = `Send + Sync`)
- [x] Inicializovat `I18n` z `settings.lang` při startu aplikace
- [x] Unit testy: Send+Sync, fallback, chybějící klíč, parsování locale

### Fáze 2 — Nahrazení hardkódovaných stringů
- [ ] `workspace/panels.rs` — "Soubory", "Build", "Chyby (n)"
- [ ] `app/dialogs.rs` — wizard, startup, quit, zavření projektu
- [ ] `app/mod.rs` — menu bar a všechny menu položky
- [ ] `modules/editor.rs` — hledání, nahrazení, záložky, go-to-line
- [ ] `modules/file_tree.rs` — kontextové menu
- [ ] `workspace/ai_panel.rs` — AI terminál, chybové stavy
- [ ] `workspace/search_picker.rs` — výsledky Ctrl+P a Ctrl+Shift+F
- [ ] `app/types.rs` (Toast) — runtime chybové hlášky

### Fáze 3 — Výběr jazyka v UI ✓ HOTOVO
- [x] ComboBox v Settings panelu (`egui::ComboBox`, `lang_display_name()`)
- [x] Uložit volbu do `settings.json` (přes `Settings.lang`)
- [x] Přetvořit `I18n` instanci po změně okamžitě (bez restartu) — `AppShared.i18n` se nahradí ihned po Uložit

### Fáze 4 — Rozšíření a zajištění kvality ✓ HOTOVO
- [x] Unit testy pokrytí klíčů (`all_lang_keys_match_english` — symetrie cs ↔ en)
- [x] Fallback: chybějící klíč → použij `en` verzi (`I18n.fallback: Option<Bundle>`)
- [x] Přidat další jazyky (sk, de, ru)
