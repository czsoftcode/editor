# Rust Editor

Jednoduchý textový editor napsaný v Rustu s eframe/egui.

## Jazyk komunikace

Komunikuj česky.

## Struktura projektu

```
src/
  main.rs          — vstupní bod, inicializace eframe, nastavení env proměnných
  config.rs        — pojmenované konstanty (font sizes, panel šířky, limity, …)
  app/
    mod.rs         — EditorApp, správa viewportů, session, IPC, startup flow
    types.rs       — sdílené typy (AppShared, AppAction, Toast, PersistentState, …)
    workspace.rs   — WorkspaceState, render_workspace a pomocné funkce panelů
    dialogs.rs     — WizardState, show_project_wizard, show_startup_dialog, helpery
    build_runner.rs — spouštění cargo příkazů, parsování chyb
    modules/
      editor.rs    — editor se záložkami, čísly řádků, syntax highlighting, vyhledáváním, autosave, markdown preview
      file_tree.rs — adresářový strom s kontextovým menu
      terminal.rs  — terminálový widget (egui_term) — Claude panel + Build terminál
  highlighter.rs   — syntax highlighting pomocí syntect
  watcher.rs       — FileWatcher (FileEvent: Changed/Removed) + ProjectWatcher
  ipc.rs           — IPC přes Unix socket, serde_json pro recent/session
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
- Projekt: Otevřít projekt (~/MyProject), Nový projekt (Rust/Symfony), seznam nedávných projektů (max 10, persistováno)
- Zobrazit: Toggle levý panel, build terminál, Claude panel
- Nápověda: O aplikaci s verzí
- Nový projekt — wizard s výběrem typu, názvu, cesty; struktura ~/MyProject/Rust|Symfony/název
- Auto-inkrementace verze při `cargo build --release` (build.rs + .build_number)
- Kontextové menu editoru (kopírovat, vložit)
- PATH fix pro GUI prostředí (~/.cargo/bin, ~/.local/bin)
- Záložky (tabs) — přepínání, zavírání (×, prostřední myš, Ctrl+W), indikátor neuloženého stavu (●)
- Vyhledávání a nahrazování — Ctrl+F hledání, Ctrl+H nahrazování, zvýrazňování výskytů, navigace ▲▼
- Build toolbar — tlačítka Build/Run/Test/Clean, parsování chyb z cargo build, klikatelný error list
- Nedávné projekty — seznam v menu Projekt i ve startup dialogu, persistováno přes eframe storage
- Otevření projektu — dialog „V tomto okně / V novém okně / Zrušit" při otevírání projektu, když je workspace již otevřen
- Multi-instance správa — IPC přes Unix socket (`~/.config/rust_editor/rust_editor.sock`); příkazy: PING, QUERY, REGISTER, UNREGISTER, ADD_RECENT, RECENT; sdílené nedávné projekty v `~/.config/rust_editor/recent.json`
- **Single-process multi-window architektura** — jeden proces, více oken (egui `show_viewport_deferred`); každý projekt se otevře v samostatném okně; `AppShared` (za `Arc<Mutex>`) pro komunikaci mezi viewporty; kořenový viewport renderuje root workspace nebo startup dialog; sekundární workspacy jsou za `Arc<Mutex<WorkspaceState>>`
- **Session restore** — při startu se obnoví všechna předchozí okna ze session souboru (`~/.config/rust_editor/session.json`); při ukončení se session uloží; podobný přístup jako PHPStorm
- **Toast notifikace** — krátkodobé chybové hlášky v pravém dolním rohu (auto-dismiss po 4 s); propagace I/O chyb z file tree do UI
- **Detekce smazání souboru** — FileWatcher zachycuje remove eventy; otevřená záložka se označí ⚠ červeně, autosave se zakáže
- **Async file dialog** — výběr složky běží ve vlastním vlákně (neblokuje UI)
- **Validace názvu projektu** — povoleny pouze `[a-zA-Z0-9_-]`, nesmí začínat `-`
- **Konfigurace konstant** — `src/config.rs` s pojmenovanými konstantami (font sizes, panel šířky, limity); připraveno pro budoucí settings.toml
- **Modulární workspace rendering** — `render_workspace` rozdělena na 8 specializovaných funkcí; sdílená `render_recent_project_list`
- **Go-to-line z build chyby** — klik na chybu v build error listu otevře soubor a skočí na příslušný řádek
- **Ctrl+G (přejít na řádek)** — inline dialog pro zadání čísla řádku; kurzor i scroll se přesunou na cílový řádek
- **Ctrl+B / Ctrl+R zkratky** — spuštění cargo build / cargo run klávesami i z menu Upravit
- **Status bar řádek:sloupec** — aktuální pozice kurzoru zobrazena vpravo ve status baru

## TODO

### Nejbližší kroky

- [ ] Klávesové zkratky — Ctrl+P rychlé otevření souboru (fuzzy)
- [ ] Hledání napříč projektem

### Střední priorita

- [ ] Git integrace — stav souborů ve stromu (barevně), commit/push/pull/diff, indikátor větve ve status baru
- [ ] Vylepšený status bar — kódování, typ souboru, git větev

### Pokročilé

- [ ] LSP integrace — rust-analyzer pro autocomplete, go-to-definition, hover dokumentace, chyby v reálném čase
- [ ] Minimap — zmenšený náhled souboru na pravé straně editoru
- [ ] Konfigurace — settings.toml (font, velikost, téma, klávesové zkratky), dark/light téma
- [ ] Vylepšení file tree — ikony podle typu souboru, drag & drop, filtrování/hledání
