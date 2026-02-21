# Roadmapa PolyCredo Editor

Tento soubor slouží jako operativní přehled úkolů a cílů projektu. Čerpá z dokumentu `docs/vize.md`.

## Q2 2026 — Produktivita a základ pro růst (měsíce 1–3)
**Cíl:** Přechod na editor preferovaný pro denní práci.

- [x] **Command palette** (`Ctrl+Shift+P`) — centrální přístup ke všem akcím klávesnicí.
- [x] **Sdílený file index** — jeden inkrementální index pro Ctrl+P, project search a file tree.
- [x] **CI/CD quality gate** — `cargo fmt`, `cargo clippy -D warnings`, `cargo test` jako podmínka merge.
- [x] **Settings.toml** — uložení konfigurace do souboru, import/export nastavení.
- [x] **Build runner profily** — vlastní příkazy (docker-compose, npm run dev) definované v projektu.

## Q3 2026 — LSP a inteligentní editor (měsíce 4–6)
**Cíl:** Přechod k "opravdovému IDE" bez ztráty výkonu.

- [x] **LSP klient MVP** — podpora rust-analyzer (autocomplete, inline chyby, hover docs).
- [ ] **LSP rozšíření** — go-to-definition, find-references pro Rust.
- [ ] **AI kontext** — agent v terminálu automaticky vidí otevřené soubory a chyby z buildu.
- [ ] **AI diff náhled** — vizuální porovnání navržených změn před aplikací na disk.
- [ ] **Klikatelné cesty v terminálu** — výstup cargo s přímým skokem na řádek v editoru.

## Q4 2026 — Rozšíření platformy (měsíce 7–9)
**Cíl:** Příprava na více platforem a rozšiřitelnost.

- [ ] **LSP plná sada** — refaktoring, rename symbol, code actions přes LSP.
- [ ] **Plugin foundation (interní)** — definice extension bodů (command registry, panel registry).
- [ ] **macOS build** — funkční binárka pro macOS (Apple Silicon + Intel).
- [ ] **Distribuce Linuxu** — AppImage / .deb balíček; Flatpak.

## Q1 2027 — Ekosystém a v1.0 (měsíce 10–12)
**Cíl:** Veřejné vydání jako stabilní produkt připravený na komunitu.

- [ ] **WASM plugin API (alpha)** — externí pluginy v sandboxu.
- [ ] **Windows build alpha** — základní funkčnost na Windows 11.
- [ ] **PolyCredo Hub (lite)** — sdílení snippetů, AI promptů a nastavení (local-first).
- [ ] **v1.0 release** — stabilní API, changelog, dokumentace.

---

## KPI a metriky
- **Cold start:** < 2,5 s
- **Ctrl+P odezva:** < 120 ms
- **Project search:** < 400 ms
- **Crash-free sessions:** ≥ 99,5 %




