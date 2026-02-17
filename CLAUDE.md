# Rust Editor

Jednoduchý textový editor napsaný v Rustu s eframe/egui.

## Jazyk komunikace

Komunikuj česky.

## Struktura projektu

```
src/
  main.rs          — vstupní bod, inicializace eframe
  app.rs           — hlavní aplikace (EditorApp), menu bar, dialogy, layout panelů
  editor.rs        — editor se záložkami, čísly řádků, syntax highlighting, vyhledáváním, autosave, markdown preview
  file_tree.rs     — adresářový strom s kontextovým menu
  highlighter.rs   — syntax highlighting pomocí syntect
  terminal.rs      — terminálový widget (egui_term) — Claude panel + Build terminál
  watcher.rs       — file watcher (notify) pro sledování změn souborů a projektu
build.rs           — auto-inkrementace patch verze při release buildu
.build_number      — čítač buildů pro verzování
```

## Závislosti

- `eframe` / `egui` — GUI framework
- `syntect` — syntax highlighting
- `notify` — sledování změn souborů
- `rfd` — nativní souborové dialogy
- `pulldown-cmark` — markdown rendering
- `egui_term` — terminálový emulátor
- `dirs` — systémové cesty (home dir apod.)
- `arboard` — clipboard

## Hotové funkce

- Otevření projektu/adresáře ze startup dialogu
- Adresářový strom s kontextovým menu (nový soubor, složka, přejmenování, smazání)
- Editor s čísly řádků, zvýrazněním aktuálního řádku a syntax highlightingem
- Markdown editor se split náhledem
- Autosave po 500 ms nečinnosti
- Sledování změn souborů (reload z disku, reload stromu)
- Sledování změn projektu (nové/smazané soubory)
- Dual terminál: Claude panel (vpravo) + Build terminál (vlevo dole)
- Menu bar: Soubor, Projekt, Upravit, Zobrazit, Nápověda
- Soubor: Otevřít složku, Uložit (Ctrl+S), Zavřít soubor, Ukončit
- Projekt: Otevřít projekt (~/MyProject), Nový projekt (Rust/Symfony)
- Zobrazit: Toggle levý panel, build terminál, Claude panel
- Nápověda: O aplikaci s verzí
- Nový projekt — wizard s výběrem typu, názvu, cesty; struktura ~/MyProject/Rust|Symfony/název
- Auto-inkrementace verze při `cargo build --release` (build.rs + .build_number)
- Kontextové menu editoru (kopírovat, vložit)
- PATH fix pro GUI prostředí (~/.cargo/bin, ~/.local/bin)
- Záložky (tabs) — přepínání, zavírání (×, prostřední myš, Ctrl+W), indikátor neuloženého stavu (●)
- Vyhledávání a nahrazování — Ctrl+F hledání, Ctrl+H nahrazování, zvýrazňování výskytů, navigace ▲▼
- Build toolbar — tlačítka Build/Run/Test/Clean, parsování chyb z cargo build, klikatelný error list

## TODO

### Nejbližší kroky

- [ ] Klávesové zkratky — Ctrl+P rychlé otevření souboru (fuzzy), Ctrl+G přejít na řádek
- [ ] Go-to-line při kliknutí na build chybu (scroll na konkrétní řádek)
- [ ] Hledání napříč projektem

### Střední priorita

- [ ] Ctrl+B build, Ctrl+R run zkratky
- [ ] Git integrace — stav souborů ve stromu (barevně), commit/push/pull/diff, indikátor větve ve status baru
- [ ] Vylepšený status bar — řádek:sloupec, kódování, typ souboru, git větev

### Pokročilé

- [ ] LSP integrace — rust-analyzer pro autocomplete, go-to-definition, hover dokumentace, chyby v reálném čase
- [ ] Minimap — zmenšený náhled souboru na pravé straně editoru
- [ ] Konfigurace — settings.toml (font, velikost, téma, klávesové zkratky), dark/light téma
- [ ] Vylepšení file tree — ikony podle typu souboru, drag & drop, filtrování/hledání
