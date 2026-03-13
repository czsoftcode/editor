# Project Research Summary

**Project:** PolyCredo Editor — CPU/Memory Performance Optimization
**Domain:** Rust desktop GUI editor (eframe/egui) — idle CPU reduction
**Researched:** 2026-03-04
**Confidence:** HIGH

## Executive Summary

PolyCredo Editor je existující Rust/eframe/egui aplikace, která vykazuje nadměrnou idle CPU zátěž způsobující přehřívání notebooku. Výzkum neřeší výběr stacku — ten je fixní (Rust + eframe 0.33.x + egui). Výzkum se zaměřil na identifikaci konkrétních API, vzorů a pastí, které snižují CPU zátěž při nečinnosti. Klíčovým zjištěním je, že eframe v continuous mode repaintuje na 60+ fps při každém pohybu myší — jedna podmíněná změna (`request_repaint_after` místo `request_repaint`) může snížit idle CPU o desítky procent.

Doporučený přístup je sekvenční a měřitelný: nejprve profiling (puffin nebo cargo flamegraph), aby byl identifikován skutečný viník, a poté aplikace optimalizací v pořadí podle dopadu. Čtyři hlavní zdroje zbytečného renderování jsou: (1) bezpodmínečné volání `request_repaint()` v timerech, (2) git polling na hlavním vlákně každých 5 s, (3) burst events z FileWatcher bez debouncingu, a (4) potenciální busy-loop v egui_term terminálovém emulátoru. Každý z těchto zdrojů lze opravit nezávisle.

Hlavní riziko je oprava "od oka" bez profilu — skutečný viník může být AccessKit feature (D-Bus overhead) nebo egui_term busy-loop, ne git polling. Druhé riziko je chybná implementace `request_repaint_after` (voláno každý frame bez podmínky — pak je ignorováno a CPU se nesníží). Obě tato rizika jsou dobře zdokumentována v egui issue trackeru a mají jasné preventivní vzory.

---

## Key Findings

### Recommended Stack

Stávající stack je správný a není důvod ho měnit. Klíčová doporučení se týkají verze a feature flags. Pokud projekt ještě nepoužívá eframe 0.33.x, upgrade je prioritní — verze 0.22 měla bug v `request_repaint_after` (issue #3109), který způsoboval ignorování throttlingu. Accesskit feature je v eframe povolena defaultně a na Linuxu způsobuje D-Bus overhead; pro vývojářský editor bez požadavku na screen readery ji lze bezpečně vypnout.

Nová závislost doporučená výzkumem: `notify-debouncer-mini` (nebo `notify-debouncer-full`) jako companion crate k existujícímu `notify`. Debouncer kolapuje burst events (3–10 na jedno uložení souboru) do jednoho eventu v 200–300ms okně. Pro profiling: `puffin` + `puffin_egui` za feature flagem `profiling` — nulový overhead v release buildu.

**Klíčové technologie a jejich role:**
- `eframe 0.33.x` — render loop a event handling; upgrade opravuje known bug v `request_repaint_after`
- `ctx.request_repaint_after(Duration)` — centrální API pro idle throttling; nahrazuje continuous mode
- `ctx.input(|i| i.viewport().focused)` — focus-aware throttling; neaktivní okna repaintují jen 1x/s
- `notify-debouncer-mini` — debounce FileWatcher burst events; 300ms okno
- `puffin` + `puffin_egui` — frame-level profiling; identifikuje skutečné hotspoty
- `std::sync::mpsc` — komunikace background thread → UI thread pro git polling a autosave

### Expected Features

Výzkum identifikoval optimalizace v pořadí podle dopadu a složitosti.

**Musí být hotovo (table stakes — bez tohoto CPU zůstane vysoké):**
- Podmíněné volání `request_repaint_after` — nahradit všechna bezpodmínečná `request_repaint()` v timerech a background callbaccích
- Baseline profiling před jakoukoliv změnou — puffin nebo cargo flamegraph; identifikovat skutečný viník
- Git polling přesunout do background threadu — mpsc pattern; repaint pouze při změně dat
- Přeskočení draw pro minimalizovaná/neaktivní okna — `ViewportInfo.focused` + `ViewportInfo.minimized` check
- Ověřit FileWatcher backend — potvrdit `RecommendedWatcher` (inotify), ne `PollWatcher`
- egui_term PTY wake-up audit — blokující čtení v background threadu, repaint pouze při datech

**Mělo by být hotovo (výrazné zlepšení, nízká složitost):**
- Disable accesskit feature v Cargo.toml — 2–10% idle CPU overhead na Linuxu (issue #4527)
- `very_lazy` eframe feature flag — zabraňuje repaintu při pohybu kurzoru nad non-hoverable prvky
- Git status debouncing po Ctrl+S — event-driven refresh + delší background interval (30 s místo 5 s)
- Autosave dirty-flag pouze — 500ms timer existuje správně; zajistit, že check nealokuje ani nerepaintuje
- FPS cap při aktivním psaní — `request_repaint_after(33ms)` místo `request_repaint()` po každém keystroke

**Odložit na pozdější fázi (vysoká složitost, malý dopad):**
- Syntax highlighter per-line cache invalidation — O(n) při změně jen relevantní pro soubory 5000+ řádků
- Scroll-area virtual scrolling — relevantní až nad 10 000 řádků
- Single shared tokio/rayon runtime pro všechny background tasky — refactoring; stdlib primitiva stačí pro první pass
- LRU limit pro syntax highlight cache — paměťová optimalizace, až po CPU optimalizaci

### Architecture Approach

Optimalizace je strukturována do čtyř vrstev, které na sebe navazují v logickém pořadí a lze je implementovat a testovat nezávisle. Klíčový princip: bez uživatelského vstupu nebo nových dat z pozadí se žádný repaint neprovede dříve než za 5 sekund (oproti stávajícímu continuous mode na 60fps). Všechny background operace komunikují s UI threadem výhradně přes `mpsc::channel` nebo `Arc<Mutex>` — nikdy neblokují `update()`.

**Hlavní komponenty a jejich odpovědnost:**
1. **Profiling Layer** — puffin instrumentace za feature flagem; identifikuje hotspoty před optimalizací; nulový dopad na release build
2. **Repaint Gate** — podmíněné rozhodování o `request_repaint` vs `request_repaint_after` v `App::update()`; centrální místo pro throttling logiku
3. **Background Task Throttle** — GitPoller v separátním threadu s mpsc; FileWatcher s debounce vrstvou; Autosave dirty-flag check
4. **Terminal Isolation** — egui_term repaint pouze při aktivním PTY výstupu; blokující čtení v background threadu bez busy-loop

Existující architektura PolyCredo Editoru (`show_viewport_deferred` pro sekundární okna) je správná a musí být zachována — přechod na `show_viewport_immediate` by způsobil lineární růst CPU s počtem otevřených oken.

### Critical Pitfalls

1. **`request_repaint_after` volaný každý frame je ignorován** — egui issue #3109: pokud se volá každý frame, throttling se nenastaví a CPU zůstane vysoké. Prevence: volat pouze podmíněně (`if !has_pending_changes`), nikdy bezpodmínečně v `update()`.

2. **Bezpodmínečné `request_repaint()` ruší celý reactive mode** — jedno nehlídané volání kdekoliv v kódu způsobuje continuous mode. Prevence: `grep -rn "request_repaint()" src/` a audit každého výskytu před implementací.

3. **AccessKit D-Bus overhead** — eframe zapíná accesskit defaultně; na Linuxu způsobuje 2–10% idle CPU přes D-Bus. Prevence: `default-features = false` v Cargo.toml a explicitní výčet features bez accesskit.

4. **Git polling blokuje main thread** — `Command::output()` na UI threadu blokuje render loop 20–200ms každých 5 s. Prevence: `std::thread::spawn` + `mpsc::channel`; `try_recv()` v `update()` nikdy nesmí blokovat.

5. **`Command::output()` deadlock při >64KB stdout** — cargo build s mnoha chybami nebo `--verbose` může zaplnit OS pipe buffer a zmrazit editor. Prevence: `build_runner.rs` musí číst stdout/stderr průběžně přes `BufReader` s iterací po řádcích.

---

## Implications for Roadmap

Na základě výzkumu je doporučena tato struktura fází:

### Phase 1: Profiling a Audit
**Rationale:** Bez dat nevíme, kde je skutečný problém. Všechny ostatní optimalizace závisí na výsledcích měření. Fáze trvá 1–2 hodiny, ale může ušetřit dny optimalizace na špatném místě.
**Delivers:** Identifikovaný hlavní viník CPU zátěže; baseline metriky (idle CPU%, FPS, frame time); seznam bezpodmínečných `request_repaint()` volání
**Addresses:** Baseline profiling (table stakes z FEATURES.md)
**Avoids:** Pitfall 10 (puffin red herring — kombinovat s cargo flamegraph), Pitfall 4 (AccessKit — zkontrolovat hned)

### Phase 2: Repaint Gate (základní throttling)
**Rationale:** Největší dopad na idle CPU s nejmenším rizikem. Jedná se o změnu volání API, ne architektury. Musí přijít před přesunem git polleru, protože repaint gate je infrastruktura, na které ostatní opravy závisí.
**Delivers:** Podmíněný `request_repaint_after` v `App::update()`; skip draw pro minimalizovaná/neaktivní okna; focus-aware throttling (1x/s pro neaktivní okna); vypnutý accesskit
**Uses:** `ctx.request_repaint_after(Duration)`, `ctx.input(|i| i.viewport().focused)`, eframe Cargo.toml feature flags
**Implements:** Repaint Gate vrstva z ARCHITECTURE.md
**Avoids:** Pitfall 1 (volání každý frame), Pitfall 2 (bezpodmínečné repaint), Pitfall 4 (accesskit)

### Phase 3: Background Task Throttle
**Rationale:** Po nastavení repaint gate má smysl opravit zdroje, které gate probouzejí zbytečně. Git polling a FileWatcher burst jsou druhý a třetí největší zdroj podle výzkumu.
**Delivers:** Git polling v background threadu (mpsc pattern); FileWatcher s notify-debouncer-mini (300ms okno); Autosave dirty-flag bez zbytečného repaintu; max 10 file events za frame
**Uses:** `std::thread::spawn`, `mpsc::channel`, `notify-debouncer-mini 0.4`
**Implements:** Background Task Throttle vrstva + GitPoller + FileWatcherBridge z ARCHITECTURE.md
**Avoids:** Pitfall 5 (git blocking main thread), Pitfall 6 (64KB deadlock v build_runner), Pitfall 7 (file event burst), Pitfall 8 (autosave polling)

### Phase 4: Terminal Isolation (egui_term audit)
**Rationale:** egui_term je high-uncertainty komponenta (alacritty backend, potenciální busy-loop). Řeší se až po fázi 3, protože (a) nejprve potřebujeme profil z fáze 1 k potvrzení, zda je viníkem, a (b) je to největší risk ve smyslu složitosti a neznámé implementace.
**Delivers:** egui_term repaint pouze při PTY výstupu; blokující čtení v background threadu; CPU v idle při zavřeném terminálu = 0 extra zátěž
**Implements:** Terminal Isolation vrstva z ARCHITECTURE.md
**Avoids:** Pitfall 9 (egui_term continuous repaint)

### Phase 5: Advanced Optimizations (volitelné)
**Rationale:** Po fázích 1–4 by měl být idle CPU dramaticky nižší. Fáze 5 jsou nice-to-have optimalizace pro speciální scénáře (velké soubory, mnoho oken) nebo pro produkční zralost.
**Delivers:** `very_lazy` feature flag; puffin dev profiler za feature flagem; FPS cap při psaní (33ms místo immediate); git refresh interval 30 s + event-driven po Ctrl+S
**Avoids:** Pitfall 11 (wgpu lag spike — zvážit glow backend), Pitfall 12 (velké scroll areas — virtual scrolling)

### Phase Ordering Rationale

- **Profiling musí být první** — bez dat hrozí optimalizace na špatném místě (accesskit může být hlavní viník, ne git polling)
- **Repaint gate před background tasks** — background task opravy musí správně volat `request_repaint_after`, ne `request_repaint`; pokud gate není hotová, opravy tasků nic nezmění
- **Terminal až nakonec** — HIGH uncertainty, potřebuje profil z fáze 1; může vyžadovat zásah do egui_term internals
- **Advanced optimalizace odloženy** — virtual scrolling a per-line syntax cache mají vysokou složitost a relevantní jsou pouze pro soubory s tisíci řádky

### Research Flags

Fáze vyžadující hlubší průzkum při implementaci:
- **Phase 4 (Terminal Isolation):** egui_term implementace není plně zdokumentována; alacritty backend specifika jsou LOW confidence; nutné prozkoumat konkrétní PTY signaling mechanismus v egui_term API

Fáze se standardními vzory (průzkum není potřeba):
- **Phase 1 (Profiling):** puffin + cargo flamegraph jsou dobře zdokumentovány
- **Phase 2 (Repaint Gate):** egui `request_repaint_after` API je HIGH confidence, vzory ověřeny
- **Phase 3 (Background Tasks):** std mpsc + notify-debouncer jsou standardní, dobře zdokumentované

---

## Confidence Assessment

| Oblast | Confidence | Poznámky |
|--------|------------|---------|
| Stack | HIGH | eframe/egui API ověřeno z docs.rs; verze 0.33.x potvrzena na crates.io; acceskit bug ověřen v issue #4527 |
| Features | HIGH (core), MEDIUM (egui_term) | Základní repaint optimalizace HIGH; egui_term specifika MEDIUM — implementace ne plně zdokumentována |
| Architecture | MEDIUM-HIGH | Repaint Gate a background thread vzory HIGH; egui_term isolation LOW confidence |
| Pitfalls | HIGH | Většina pitfalls ověřena v oficiálním egui GitHub issue trackeru; Rust stdlib deadlock issue #27152 |

**Celková confidence:** HIGH pro fáze 1–3, MEDIUM pro fázi 4

### Gaps to Address

- **egui_term PTY signaling:** Konkrétní API pro detekci "jsou nová data na PTY?" v egui_term není v dokumentaci jasně popsáno. Při implementaci fáze 4 nutné prozkoumat zdrojový kód egui_term nebo komunitní issues.
- **egui issue #4945 (animation + deferred viewport bug):** PolyCredo používá `show_viewport_deferred`; existuje bug s animacemi v tomto režimu. Při implementaci cursor blink nebo toast animací v fázi 2 nutné důkladně testovat.
- **wgpu vs glow backend:** Pokud projekt používá wgpu a trpí issue #5958 (100ms lag spike na click), může být nutné zvážit přechod na glow backend. Zjistit při profilu v fázi 1.
- **Skutečná míra CPU úspory:** Výzkum uvádí vzory a pořadí, ale konkrétní % úspora závisí na profilu konkrétní instalace. Baseline měření v fázi 1 definuje realistický cíl pro fáze 2–4.

---

## Sources

### Primary (HIGH confidence)
- [egui::Context docs — request_repaint_after](https://docs.rs/egui/latest/egui/struct.Context.html)
- [egui::Options docs — repaint_on_widget_change](https://docs.rs/egui/latest/egui/struct.Options.html)
- [eframe::NativeOptions docs](https://docs.rs/eframe/latest/eframe/struct.NativeOptions.html)
- [egui Issue #3109 — request_repaint_after ignored when called each frame](https://github.com/emilk/egui/issues/3109)
- [egui Issue #4527 — High CPU with accesskit enabled](https://github.com/emilk/egui/issues/4527)
- [egui Issue #3982 — Windows High CPU when minimized](https://github.com/emilk/egui/issues/3982)
- [egui Discussion #1261 — Reduce CPU Usage](https://github.com/emilk/egui/discussions/1261)
- [egui Discussion #1428 — How to use threads/atomics](https://github.com/emilk/egui/discussions/1428)
- [egui Issue #5958 — eframe + wgpu 100ms lag spike](https://github.com/emilk/egui/issues/5958)
- [Rust Issue #27152 — Command hangs when output > 64KB](https://github.com/rust-lang/rust/issues/27152)
- [EmbarkStudios/puffin GitHub](https://github.com/EmbarkStudios/puffin)
- [puffin_egui 0.29.0 on crates.io](https://crates.io/crates/puffin_egui)
- [notify-debouncer-mini docs](https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/)

### Secondary (MEDIUM confidence)
- [egui Discussion #4062 — Expected CPU load of empty window](https://github.com/emilk/egui/discussions/4062)
- [egui Discussion #4956 — FPS affected by mouse movement](https://github.com/emilk/egui/discussions/4956)
- [egui Issue #4945 — Animation + deferred viewport bug](https://github.com/emilk/egui/issues/4945)
- [egui PR #4880 — very_lazy feature flag](https://github.com/emilk/egui/pull/4880)
- [egui Issue #5092 — 100% CPU wgpu 0.20](https://github.com/emilk/egui/issues/5092)
- [egui_term GitHub](https://github.com/Harzu/egui_term)
- [alacritty busy-loop issue #8413](https://github.com/alacritty/alacritty/issues/8413)
- [Multi-Viewport System — DeepWiki](https://deepwiki.com/membrane-io/egui/6.2-multi-viewport-system)

### Tertiary (LOW confidence)
- egui_term PTY signaling internals — potřeba ověřit ze zdrojového kódu při implementaci fáze 4

---
*Research completed: 2026-03-04*
*Ready for roadmap: yes*
