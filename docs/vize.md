# PolyCredo Editor — Vize na 12 měsíců

> **Datum:** 2026-02-20  
> **Horizont:** Q1 2026 → Q1 2027  
> **Verze dokumentu:** 1.0  

---

## 1. Kde jsme dnes

PolyCredo Editor je funkční desktop IDE napsaný v Rustu (egui/eframe) s pevnou architekturální základnou:

- Single-process multi-window, IPC přes Unix socket
- i18n pro 5 jazyků (cs, en, sk, de, ru)
- Editor se záložkami, syntax highlighting, autosave, vyhledávání, git integrace
- Dual terminál: AI panel (Claude Code, Gemini CLI) + Build terminál
- Session restore, file watcher, toast notifikace
- Kritické chyby z auditu 2026-02 opraveny (data loss, IPC race, thread leak, PTY orphans)

Aplikace má solidní základ. Teď rozhoduje, jak ho využít.

---

## 2. Identita — kdo PolyCredo je

PolyCredo **není** alternativou VS Code nebo Cursor v šíři funkcí.  
PolyCredo **je** vizuální orchestrátor pro CLI AI agenty s nativní rychlostí Rustu.

**Tři pilíře identity:**

| Pilíř | Popis |
|---|---|
| **AI-native hostitel** | Nejlepší GUI obálka pro Claude Code, Gemini CLI, Aider. Když vyjde nový agent, uživatel ho má okamžitě v PolyCredo. |
| **Rust výkon, ne Electron závaží** | Reaguje okamžitě i na slabém HW. Zlomek RAM oproti Cursor/VS Code. |
| **Local-first soukromí** | Kód uživatele neopouští jeho stroj. Agenti běží lokálně nebo pod jeho kontrolou. |

**Za 12 měsíců** by měl vývojář říct:  
*„PolyCredo je editor, kde mám kód, terminál a AI agenta v jednom okně — a nic z toho nelaguje."*

---

## 3. Roadmapa po kvartálech

### Q2 2026 — Produktivita a základ pro růst *(měsíce 1–3)*

Cíl: z "dobrého editoru" na "editor, který vývojáři skutečně preferují pro denní práci".

**Dodávky:**
- [x] **Command palette** (`Ctrl+Shift+P`) — centrální přístup ke všem akcím klávesnicí
- [x] **Sdílený file index** — jeden inkrementální index pro Ctrl+P, project search a file tree; žádné opakované skenování FS
- [ ] **CI/CD quality gate** — `cargo fmt`, `cargo clippy -D warnings`, `cargo test` jako podmínka merge; lokálně i v pipeline
- [ ] **Settings.toml** — uložení konfigurace do souboru, import/export nastavení
- [ ] **Build runner profily** — vlastní příkazy (docker-compose, npm run dev) definované v projektu, nejen cargo

**Co neděláme:** žádné velké refaktory architektury, žádné nové závislosti nad rámec nutnosti.

---

### Q3 2026 — LSP a inteligentní editor *(měsíce 4–6)*

Cíl: přechod od "rychlého editoru" k "opravdovému IDE" bez ztráty výkonu.

**Dodávky:**
- **LSP klient MVP** — podpora rust-analyzer (autocomplete, inline chyby, hover docs)
- **LSP rozšíření** — go-to-definition, find-references pro Rust; základ pro další jazyky (TypeScript, Python)
- **AI kontext** — agent v terminálu automaticky vidí otevřené soubory a chyby z buildu (bez manuálního kopírování)
- **AI diff náhled** — vizuální porovnání navržených změn před aplikací na disk
- **Klikatelné cesty v terminálu** — výstup cargo/compileru s přímým skokem na řádek v editoru

**Poznámka k LSP:** Syntect (regex highlighting) zůstane pro rychlost při psaní; LSP jede paralelně a doplňuje sémantiku. Nenahrazujeme, kombinujeme.

---

### Q4 2026 — Rozšíření platformy *(měsíce 7–9)*

Cíl: připravit aplikaci na více platforem a první vrstvu rozšiřitelnosti.

**Dodávky:**
- **LSP plná sada** — refaktoring, rename symbol, code actions přes LSP
- **Plugin foundation (interní)** — definice extension bodů (command registry, panel registry, on_save/on_build hooks); zatím jen interní Rust moduly, ne external ABI
- **macOS build** — funkční binárka pro macOS (Apple Silicon + Intel), distribuce přes GitHub Releases
- **Distribuce Linuxu** — AppImage / .deb balíček; Flatpak jako bonus

---

### Q1 2027 — Ekosystém a v1.0 *(měsíce 10–12)*

Cíl: veřejné vydání jako stabilní produkt připravený na komunitu.

**Dodávky:**
- **WASM plugin API (alpha)** — externí pluginy v sandboxu (WASM runtime); první referenční plugin (custom theme nebo jazykový highlighter)
- **Windows build alpha** — základní funkčnost na Windows 11
- **PolyCredo Hub (lite)** — sdílení snippetů, AI promptů a nastavení přes soubory/git repo (ne cloud service); Local-first, žádný backend
- **v1.0 release** — stabilní API, changelog, dokumentace "getting started" a "pro vývojáře pluginů"
- **Release channels** — stable / beta; automatický changelog z git tagů

---

## 4. Co záměrně odkládáme za horizont 12 měsíců

| Funkce | Důvod odložení |
|---|---|
| Real-time kolaborace | Vyžaduje backend infrastrukturu; nejdřív stabilní jednoduché jádro |
| PolyCredo Sync (cloud) | Cloud service = ops zátěž; místo toho Local-first Hub přes soubory |
| AI Command Pilot (bash z přirozeného jazyka) | Zajímavé, ale nesouvisí s hlavním USP; riziko scope creep |
| Minimap | Nízká priorita uživatelů, vizuální šum |
| Drag & drop v file tree | Technicky náročné v egui, nízká denní hodnota |

---

## 5. Architektonické směřování

Žádná velká přestavba. Postupné zavedení hranic:

```
core/        — editor, session, I/O (stávající základ)
indexing/    — sdílený file/symbol index (nový Q2)
lsp/         — LSP klient (nový Q3)
ai/          — provider adaptéry, kontext pipeline (rozšíření Q3)
plugins/     — extension body (nový Q4, interní; WASM Q1 2027)
```

Event model: sjednotit na jasné zprávy mezi moduly místo ad-hoc channel handoffů (postupně, ne najednou).

---

## 6. KPI — jak poznáme, že jsme na správné cestě

### Technické

| Metrika | Cíl |
|---|---|
| Crash-free sessions | ≥ 99,5 % |
| Cold start | < 2,5 s |
| Ctrl+P odezva | < 120 ms (střední projekt) |
| Project search první výsledek | < 400 ms (teplý cache) |
| LSP autocomplete latence | < 200 ms |

### Produktové

| Metrika | Cíl |
|---|---|
| D7 retence | Roste MoM od Q2 |
| Podíl uživatelů aktivně využívajících AI panel | > 50 % |
| Počet otevřených bug reportů kategorie "data loss" | 0 |

### Procesní

| Metrika | Cíl |
|---|---|
| Release cadence | ≥ 1 stabilní release / měsíc |
| PR prochází quality gate bez opakování | ≥ 90 % |
| Průměrná doba od bug reportu po fix | < 7 dní (kritické) |

---

## 7. Zásady, které budeme dodržovat

1. **Stability first.** Žádná nová feature bez toho, aby stávající věci fungovaly spolehlivě.
2. **Small safe increments.** Malé patche, meritelný dopad, rychlá verifikace. Žádné velké "big bang" refaktory.
3. **i18n od začátku.** Každý nový string jde přes i18n klíč. Nikdy hardcoded text v kódu.
4. **Local-first.** Uživatelova data a kód zůstávají na jeho stroji. Cloudové funkce jen opt-in.
5. **USP neředit.** PolyCredo je hostitel AI agentů, ne vlastní AI chatbot. Neduplikujeme to, co Claude Code nebo Gemini CLI dělají líp.

---

## 8. Závěr

Za 12 měsíců bude PolyCredo Editor platforma s jasnou identitou:

**Rychlý jako CLI. Pohodlný jako IDE. S AI agentem po ruce — bez kompromisů v soukromí a výkonu.**

Cesta tam vede přes pořadí: *stabilita → produktivita → LSP → rozšiřitelnost*. Bez zkratek.

