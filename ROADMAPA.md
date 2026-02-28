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
- [x] **Právní rámec (Licencování)** — zavedení AGPLv3 a Dual-Licensing modelu pro komerční udržitelnost.
- [x] **CLA (Contributor License Agreement)** — právní ochrana umožňující budoucí prodej nebo změnu licence.

## Q2 2026 — LSP a inteligentní editor
**Cíl:** Přechod k "opravdovému IDE" bez ztráty výkonu.

- [x] **LSP klient MVP** — podpora rust-analyzer (autocomplete, inline chyby, hover docs).
- [x] **LSP rozšíření** — go-to-definition, find-references pro Rust.
- [x] **AI kontext** — agent v terminálu automaticky vidí otevřené soubory a chyby z buildu.
- [x] **AI diff náhled** — vizuální porovnání navržených změn před aplikací na disk.
- [x] **Klikatelné cesty v terminálu** — výstup cargo s přímým skokem na řádek v editoru.
- [x] **Branding & UX (Gemini CLI)** — barevné logo, metadata a přehledná typografie (120% font, MD syntax).
- [x] **Non-blocking AI Chat** — AI chat asistent již neblokuje editaci souborů a interakci s panely na pozadí.
- [x] **Ollama Stability Fix** — Opraven "Internal Server Error" při prohledávání velkých projektů skrze AI agenty.
- [x] **AI Inspector & Trace** — centrální nástroj pro sledování odesílaného kontextu a JSON payloadů.
- [x] **Conversational Threads** — podpora historie a "vláken" konverzace s AI agenty.

## AI Agenti a Autonomie (Nová vize)
**Cíl:** Transformace editoru na platformu pro plně autonomní AI agenty.

- [x] **Pokročilý Tool Use (MCP)** — agent může aplikovat patche, spouštět testy v sandboxu a dotazovat se LSP na definice.
- [x] **Sémantický kontext (StandardAI RAG)** — centrální implementace lokální vektorové indexace projektu. Sdílené vyhledávání v codebase dostupné pro všechny agenty (Gemini, Claude CLI atd.).
- [x] **Human-in-the-loop 2.0** — side-by-side diff preview pro všechny AI změny a potvrzování "nebezpečných" operací.
- [x] **Automatické zálohování (Safety Net)** — propojení `LocalHistory` s procesem schvalování změn (automatický snapshot původního souboru před přepsáním ze sandboxu).
- [x] **Reasoning Loop** — podpora pro cyklus Plán -> Akce -> Observace -> Korekce přímo v UI (interní monolog).
- [x] **Vlastní CLI Agenti** — možnost definice vlastních AI nástrojů (Claude, Gemini, Aider) přímo v nastavení aplikace s dynamickým UI.
- [x] **Dlouhodobá paměť agenta** — ukládání faktů a kontextu do perzistentního JSON souboru, který přežije restart editoru.
- [ ] **Git & Historie** — integrace s Gitem pro automatické commity a analýzu historie kódu agentem.

## Q3 2026 — Rozšíření platformy
**Cíl:** Příprava na více platforem a plná podpora LSP.

- [ ] **LSP plná sada** — refaktoring, rename symbol, code actions přes LSP.
- [ ] **macOS build** — funkční binárka pro macOS (Apple Silicon + Intel).
- [x] **Distribuce Linuxu** — AppImage, .deb, .rpm, .pkg.tar.zst (Arch), Flatpak, Snap a .tar.gz.
- [x] **Ochrana Buildu** — automatické blokování sestavení při nepromotovaných souborech nebo Sandbox ON.
- [ ] **WASM plugin API (beta)** — první vlna komunitních pluginů.

## Q4 2026 — Ekosystém a v1.0
**Cíl:** Veřejné vydání jako stabilní produkt připravený na komunitu.

- [x] **Windows build (v0.8.0)** — plná podpora Windows 10/11 včetně cross-kompilace, ikon a stabilního IPC.
- [ ] **PolyCredo Hub (lite)** — sdílení snippetů, AI promptů a nastavení (local-first).
- [ ] **v1.0 release** — stabilní API, changelog, dokumentace.

---

## KPI a metriky
- **Cold start:** < 2,5 s
- **Ctrl+P odezva:** < 120 ms
- **Project search:** < 400 ms
- **Crash-free sessions:** ≥ 99,5 %




