# AGENTS.md

Tento soubor definuje pracovní rámec pro Codex agenta v repozitáři `rust_editor`.

## Jazyk komunikace

- Komunikuj česky.
- Komentáře taky vypisuj v češtině.
- Pracuj výhradně v adresáři ./ do jiných adresářů jenom se svolením.

## Kontext projektu

- Aplikace: desktop textový editor v Rustu (`eframe/egui`).
- Klíčové moduly:
  - `src/main.rs` - start aplikace, globální inicializace.
  - `src/app/` - UI/workspace logika, dialogy, build runner.
  - `src/ipc.rs` - IPC přes unix socket + recent/session persistence.
  - `src/watcher.rs` - file/project watchery.
  - `src/highlighter.rs` - syntax highlighting.

## Cíl převzetí práce

- Nezačínat velké refaktory, dokud nejsou opravené bezpečnostní a robustnostní problémy.

## Prioritní backlog (v pořadí)

1. `V-1` Přesun environment inicializace (`set_var`) do `main()` před startem vláken.
2. `V-2` Důsledná validace názvu projektu (regex `^[a-zA-Z0-9_-]+$`, zákaz `-` na začátku, zákaz `/` a `\`).
3. `K-1` Validace cest z IPC (`is_absolute()`, `is_dir()`) před použitím.
4. `S-3` Neignorovat I/O chyby, propagovat je do UI (toast).
5. `N-5` Nahradit ruční JSON serializaci za `serde_json`.
6. `S-1` Změnit `FileWatcher::try_recv()` tak, aby neztrácel eventy (např. `Vec<PathBuf>`/`HashSet<PathBuf>`).
7. `S-4` Přidat handling remove eventů ve watcheru.
8. `V-3` Držet file dialog asynchronní (UI nesmí blokovat).

## Implementační pravidla

- Preferuj bezpečné API, `unsafe` používej jen když je to opravdu nutné a zdokumentované.
- Neprováděj blokující operace v UI vlákně.
- Každou změnu ověř minimálně přes `cargo check` a `./check.sh`.
- Pokud jde o výkon v editoru, neklonuj zbytečně velké textové buffery.
- Zachovej existující architekturu single-process multi-window.

## Doporučený workflow pro každou změnu

1. Vyber jednu položku backlogu.
2. Proveď minimální cílený patch.
3. Spusť `cargo check`.
4. Pokud změna zasahuje logiku, doplň/aktualizuj testy, kde je to možné.
5. Stručně zapiš, co bylo opraveno a co zbývá.

## Git workflow

- **Výchozí větev je `develop`**, ne `main`. Všechny slice merge a běžná práce probíhá na `develop`.
- `main` se používá jen pro release tagy.
- Slice branche se vytvářejí z `develop` a mergují zpět do `develop`.
- V pracovním stromu mohou být lokální změny uživatele (např. `CLAUDE.md`, `.build_number`).
- Tyto změny nerevertovat, pokud to není explicitně požadováno.
