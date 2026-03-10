# PolyCredo Editor

## What This Is

Multiplatformní textový editor s AI podporou napsaný v Rustu (eframe/egui). Nabízí syntaxové zvýraznění s light/dark mode (3 varianty světlé palety), záložky, terminálové panely, git integraci, správu projektů a AI asistenta (Claude panel).

## Core Value

Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## Requirements

### Validated

- v1.0.2 Dark/Light Mode:
  - LightVariant enum (WarmIvory, CoolGray, Sepia) + to_egui_visuals() + syntect_theme_name() bez bliknutí
  - Theme-aware terminály, scrollbar, file tree git barvy — explicit light/dark palety
  - Tři světlé varianty s live picker v Settings, canonical settings.toml persist + legacy migrace
  - Sandbox mode jako perzistentní nastavení s okamžitým apply po Save, multi-window propagace
  - Runtime sandbox apply: restart terminálů, remap tabů, blokace OFF při staged, sync při ON

- v1.0.6 Focus Management:
  - Terminal hover-to-focus odstraněn na všech 4 cestách (docked+float, right+bottom)
  - dialog_open guard na všech terminal focus paths — modály a AI Chat blokují terminal focus
  - Modal overlay backdrop (interactive Area) blokuje interakci za modalem
  - close_on_click_outside — Settings/Plugins se zavírají pouze přes tlačítka
  - Settings discard confirmation při neuložených změnách

- v1.1.0 Sandbox Removal:
  - Kompletní odstranění sandbox.rs modulu a datových struktur (Sandbox, SyncPlan)
  - UI vyčištěno — settings toggle, modální dialogy, build bar label, file tree sandbox prvky
  - Sandbox logika odstraněna z file operations, watcheru a git/build guardů
  - Plugin systém: sandbox_root → project_root, exec_in_sandbox → exec
  - 43+ sandbox i18n klíčů odstraněno ze všech 5 jazyků
  - 26/26 requirements satisfied, zero compile warnings, 57 passing testů

- v1.2.0 AI Chat Rewrite:
  - AiProvider trait + OllamaProvider s NDJSON streaming, auto-detect serveru, model picker
  - AiState konsolidace — ChatState, OllamaState, AiSettings sub-structy
  - Hybrid CLI chat UI se streaming renderingem, markdown, dark/light mode
  - Tool execution — read/write/exec/search/ask-user s approval workflow
  - Security infrastruktura — PathSandbox, SecretsFilter, CommandBlacklist, RateLimiter, AuditLogger
  - Kompletní odstranění WASM plugin systému (~6,500 LOC)
  - Plná i18n lokalizace CLI chatu v 5 jazycích (cs, en, de, ru, sk)
  - 20/20 requirements satisfied, 58,187 LOC Rust

- v1.2.1 Save Modes + Unsaved Changes Guard:
  - Ctrl+S ukládá aktivní tab bez změny fokusu
  - Nastavení Auto/Manual save mode v Settings s okamžitým runtime apply
  - Guard dialog při zavírání neuloženého tabu (Save/Discard/Cancel)
  - Guard dialog při zavírání aplikace s neuloženými soubory
  - Status bar indikace save režimu (Manual/Auto) a dirty stavu
  - Tab dirty indikace (●) s vizuální prioritou před mode markerem
  - Save error toast + inline error zpráva, tab zůstává otevřený
  - 18 regression testů pro save UX kontrakt

### Active

(Další requirements budou definovány v další milestone)

## Current Milestone: v1.3.0 Additional Themes

**Goal:** Přidat 4. světlé téma (mezi sepia a hnědou, ne moc tmavé) a volitelně druhé dark téma.

**Target features:**
- 4. světlé téma: barva mezi sepia a hnědou, příjemná pro oči, ne moc tmavá
- (Volitelně) 2. dark téma jako varianta k existujícímu

---

## Previous Milestone: v1.2.1 Save Modes + Unsaved Changes Guard — SHIPPED

**Goal:** Zpřehlednit a zbezpečnit ukládání v editoru přes výchozí Ctrl+S workflow, přepínání auto/manual režimu a ochranu proti ztrátě neuložené práce při zavírání tabu nebo aplikace.

**Status:** ✅ SHIPPED 2026-03-10

**Výsledky:**
- 3 fáze (24-26), 18 plans, všechny dokončeny
- Save mode runtime key kontrakt oddělen od settings draftu
- MODE-04 regression testy v dedikovaném workspace test modulu
- Save UX priority dirty-first vizuální kontrakt
  - i18n smoke coverage pro 5 jazyků

### Out of Scope

- LSP integrace — jiný projekt
- Minimap — patří do jiné oblasti
- OS auto-detect dark/light — experimentální v egui, záměrně vynecháno (v1.0.2 decision)
- Vlastní theme editor — mimo rozsah
- Animované přechody mezi tématy — nedostupné v egui
- Centrální focus manager / FocusStack — over-engineering, dialog_open pattern stačí (v1.0.6 decision)
- OpenAI-compatible endpoint pro Ollama — nativní API je spolehlivější pro streaming tool calling
- Auto-execute bez approval — bezpečnostní riziko
- RAG / vector DB — over-engineering, existující semantic search stačí

## Context

**Shipped:** v1.2.0 AI Chat Rewrite (2026-03-06)
- 6 fází, 19 plánů, 74 commitů (42 feat/fix)
- 20/20 requirements satisfied
- 133 files changed, +12,405/-9,193 lines (net +3,212)
- 58,187 řádků Rust

**Previous:** v1.1.0 Sandbox Removal (2026-03-06), v1.0.6 Focus Management (2026-03-05), v1.0.2 Dark/Light Mode (2026-03-05)

**Tech stack:** Rust + eframe/egui, syntect, egui_term, fluent (i18n), notify, rfd, pulldown-cmark, ureq, globset

**Known tech debt:**
- Nyquist VALIDATION.md: fáze ve stavu draft
- Warning text kontrast v light mode (nahlášeno při UAT fáze 5)
- UI-02: záložkový indikátor nemá dedikovaný kontrast test
- Syntax highlighting v AI chatu nefunguje (egui_commonmark code blocky černobílé)

## Constraints

- **Tech stack**: Rust + eframe/egui — řešení musí být kompatibilní s tímto stackem
- **Zpětná kompatibilita**: Funkčnost editoru nesmí být narušena
- **Bez externích závislostí**: Nechceme přidávat nové heavy dependencies jen kvůli optimalizaci

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| settings_version AtomicU64 jako integrační sběrnice | Jedno místo propagace změny tématu do všech viewportů | ✓ Funguje pro theme i sandbox apply |
| ui.visuals() pro live theming v render path | Zero-lag propagace, vždy aktuální | ✓ Standard pro terminál, file tree |
| warm_ivory_bg() heuristika přes r-b threshold | Detekce varianty bez enum přenosu do render vrstvy | ✓ Elegantní, bez coupling |
| settings.toml jako canonical, settings.json jako legacy | Čistá migrace bez breaking change | ✓ Roundtrip testy pokrývají oba formáty |
| Profilovat před optimalizací | Neopravovat naslepo — změřit skutečné hotspoty | — Pending (future milestone) |
| dialog_open pattern místo FocusStack | Rozšíření existujícího vzoru stačí, FocusStack je over-engineering | ✓ Pokryto 9 requirements jednou fází |
| Backdrop jako interactive Area (Order::Middle) | layer_painter blokoval interakci s modalem | ✓ Modal plně interaktivní |
| close_on_click_outside builder pattern | Rozlišení info modalů (About) vs interaktivních (Settings) | ✓ Clean API, žádný breaking change |
| Simplified Toast to message-only | Removed ToastAction/ToastActionKind — sandbox was only consumer | ✓ Cleaner API, zero sandbox coupling |
| sandbox_root → project_root rename | Semantic clarity after sandbox removal | ✓ Plugin registry uses correct naming |
| exec_in_sandbox → exec rename | No sandbox context needed | ✓ AI tools and WASM plugins updated |

| Nativní providery místo WASM | Jednodušší, rychlejší, bez WASM runtime overhead | ✓ OllamaProvider nativně, ~6,500 LOC WASM odstraněno |
| Ollama first, trait abstrakce | Centralizovaný design rozšiřitelný pro Claude/Gemini | ✓ AiProvider trait, extensible |
| Postupné mazání starého kódu | Nový chat funguje paralelně, starý se odstraní až po dokončení | ✓ Paralelní provoz → čisté odstranění |
| Hybrid CLI UI | Prompt dole jako CLI, odpovědi nahoře s vizuálním oddělením | ✓ Funguje s dark/light mode |
| ureq + std::thread místo reqwest/tokio | Odpovídá threading modelu codebase | ✓ Jednodušší, bez async runtime |
| Collect-then-process v background polling | Borrow checker safety pro StreamEvent zpracování | ✓ Čistý pattern |
| Security-first tool execution | PathSandbox + approval workflow před jakýmkoli nástrojem | ✓ 28 security testů |
| Status bar save mode z runtime, ne draft | Oddělení runtime kontraktu od settings draftu | ✓ MODE-04 baseline |
| Dirty-first vizuální priorita v status baru | Dirty stav je primární signál, mode marker sekundární | ✓ Čitelnost zachována |
| TDD test-first pro UI kontrakt | Red/green commits s explicitními regression testy | ✓ MODE-04 testy pokrývají edge cases |
| i18n smoke test pro phase-specific keys | phase_26_save_ux_keys helper pro lokalizaci | ✓ 5 jazyků ověřeno |

---
*Last updated: 2026-03-10 after v1.3.0 milestone started*
