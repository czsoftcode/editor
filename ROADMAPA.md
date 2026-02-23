# Roadmapa PolyCredo Editor

Tento soubor slouží jako operativní přehled úkolů a cílů projektu. Čerpá z dokumentu `docs/vize.md`.

## Q1 2026 — Architektura a pluginy (Právě probíhá)
**Cíl:** Příprava na rozšiřitelnost a integraci externích modulů.

- [x] **Command palette** (`Ctrl+Shift+P`) — centrální přístup ke všem akcím klávesnicí.
- [x] **Plugin foundation (interní)** — definice extension bodů (command registry, panel registry).
- [x] **WASM Plugin Manager (POC)** — bezpečné načítání externích `.wasm` modulů přes Wasmtime/Extism.
- [ ] **Plugin SDK for Rust** — knihovna pro snadný vývoj pluginů v Rustu.
- [x] **Capability-based Security** — definice oprávnění pro pluginy (např. přístup k souborům, síti).
- [x] **Sdílený file index** — jeden inkrementální index pro Ctrl+P, project search a file tree.
- [x] **Settings.toml** — uložení konfigurace do souboru, import/export nastavení.

## Q2 2026 — LSP a inteligentní editor
**Cíl:** Přechod k "opravdovému IDE" bez ztráty výkonu.

- [x] **LSP klient MVP** — podpora rust-analyzer (autocomplete, inline chyby, hover docs).
- [x] **LSP rozšíření** — go-to-definition, find-references pro Rust.
- [x] **AI kontext** — agent v terminálu automaticky vidí otevřené soubory a chyby z buildu.
- [x] **AI diff náhled** — vizuální porovnání navržených změn před aplikací na disk.
- [x] **Klikatelné cesty v terminálu** — výstup cargo s přímým skokem na řádek v editoru.

## Q3 2026 — Rozšíření platformy
**Cíl:** Příprava na více platforem a plná podpora LSP.

- [ ] **LSP plná sada** — refaktoring, rename symbol, code actions přes LSP.
- [ ] **macOS build** — funkční binárka pro macOS (Apple Silicon + Intel).
- [ ] **Distribuce Linuxu** — AppImage / .deb balíček; Flatpak.
- [ ] **WASM plugin API (beta)** — první vlna komunitních pluginů.

## Q4 2026 — Ekosystém a v1.0
**Cíl:** Veřejné vydání jako stabilní produkt připravený na komunitu.

- [ ] **Windows build alpha** — základní funkčnost na Windows 11.
- [ ] **PolyCredo Hub (lite)** — sdílení snippetů, AI promptů a nastavení (local-first).
- [ ] **v1.0 release** — stabilní API, changelog, dokumentace.

---

## KPI a metriky
- **Cold start:** < 2,5 s
- **Ctrl+P odezva:** < 120 ms
- **Project search:** < 400 ms
- **Crash-free sessions:** ≥ 99,5 %




