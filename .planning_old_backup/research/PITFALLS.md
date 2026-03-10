# Domain Pitfalls: egui/eframe CPU Optimization

**Domain:** Rust desktop editor with eframe/egui — CPU & idle performance optimization
**Researched:** 2026-03-04
**Project:** PolyCredo Editor

---

## Critical Pitfalls

Chyby, které způsobí, že optimalizace vůbec nebude fungovat nebo vytvoří nové problémy.

---

### Pitfall 1: request_repaint_after() volaný každý frame je ignorován

**What goes wrong:**
Pokud voláš `ctx.request_repaint_after(Duration::from_secs(1))` uvnitř `update()` každý frame,
egui (0.22+) toto volání ignoruje z hlediska throttlingu a renderuje dál na plné rychlosti.
Chyba vzniká, protože každý nový frame resetuje timer dříve, než vyprší.

**Why it happens:**
egui v 0.22.0 změnil interní logiku zpracování repaint requestů. Pokud je `request_repaint_after()`
voláno opakovaně každý frame, systém ho zpracuje stejně jako `request_repaint()` bez delay.
Issue #3109 potvrzuje, že na 144Hz monitoru zobrazuje ~90fps i mimo focus, místo 1fps.

**Consequences:**
- CPU zátěž se vůbec nesníží navzdory zdánlivě správnému kódu
- Falešný pocit, že optimalizace proběhla
- Vývojář netrefí skutečnou příčinu

**Prevention:**
Volej `request_repaint_after()` POUZE při výskytu konkrétní události nebo změny stavu —
nikoli bezpodmínečně každý frame. Pattern:

```rust
// SPATNE — volano kazdy frame, throttling se ignoruje
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint_after(Duration::from_secs(1)); // ignorovano!
    // ...
}

// SPRAVNE — volej jen kdyz neni zadna aktivita
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if self.has_pending_changes() {
        ctx.request_repaint(); // okamzite kdyz je co zobrazit
    } else {
        ctx.request_repaint_after(Duration::from_secs(5)); // skutecny idle
    }
}
```

**Detection:**
Pokud po přidání `request_repaint_after()` FPS nezklesne, téměř jistě se jedná o tento problém.
Zkontroluj, zda call nestojí na bezpodmínečném místě kódu.

**Phase:** Fáze 1 (základní repaint throttling)

---

### Pitfall 2: Bezpodmínečné request_repaint() bez důvodu

**What goes wrong:**
Volání `ctx.request_repaint()` kdekoli v `update()` bez podmínky způsobuje
continuous mode rendering — egui neustále překresluje na plné rychlosti procesoru,
i když se na obrazovce nic nemění.

**Why it happens:**
egui je reactive by default — repaintuje pouze při interakci nebo animaci.
Stačí jedno volání `request_repaint()` bez podmínky a celý mechanismus reactive mode
je zrušen.

**Consequences:**
- 30–100% CPU na idle
- Zahřívání notebooku
- Zbytečná spotřeba baterie

**Prevention:**
Audit veškerého kódu na výskyt `request_repaint()`. Každý výskyt musí mít podmínku.
Legitimní důvody pro repaint: nová data z background threadu, animace, změna stavu.

**Detection:**
`grep -rn "request_repaint()" src/` — každý výsledek zkontroluj, zda je podmíněný.

**Phase:** Fáze 1 (audit existujícího kódu)

---

### Pitfall 3: Immediate viewport místo deferred pro sekundární okna

**What goes wrong:**
PolyCredo používá `show_viewport_deferred()` — to je správně. Pokud by bylo použito
`show_viewport_immediate()`, každý repaint root viewportu by způsobil repaint
VŠECH child viewportů, a naopak. S N okny = N× CPU zátěž i pro operace v jednom okně.

**Why it happens:**
`show_viewport_immediate()` je jednodušší na implementaci (nevyžaduje Arc/Mutex),
takže vývojáři ho volí pro jednoduchost. Dokumentace repaint coupling dostatečně nevysvětluje.

**Consequences:**
- S 3 otevřenými projekty: 3× zbytečný repaint při každé interakci v jednom okně
- CPU zátěž roste lineárně s počtem oken

**Prevention:**
Vždy používat `show_viewport_deferred()` pro sekundární okna. PolyCredo toto již dělá správně.
Při refactoringu nepřejít na immediate variantu z důvodu "zjednodušení kódu".

**Detection:**
Grep `show_viewport_immediate` v codebase — pokud existuje, je to problém.

**Phase:** Existující architektura je správná — pozor při budoucím refactoringu

---

### Pitfall 4: AccessKit feature způsobuje high CPU přes D-Bus

**What goes wrong:**
eframe má `accesskit` jako default feature. Na Linuxu to spouští D-Bus komunikaci (zbus),
která generuje CPU overhead i v idle stavu. Profily ukazují call stack:
`accesskit → zbus → async runtime → stálé CPU aktivity`.

**Why it happens:**
AccessKit pro screen readery musí udržovat živé spojení s accessibility bus.
Na Linuxu jde přes D-Bus, který polluje nebo čeká na zprávy i bez aktivity.

**Consequences:**
- 2–10% CPU idle overhead pouze kvůli accessibility
- Problém je reportován v issue #4527

**Prevention:**
Pokud accessibility není explicitně požadována, deaktivovat feature v `Cargo.toml`:

```toml
eframe = { version = "...", default-features = false, features = [
    "default_fonts",
    "wgpu",  # nebo glow
    # accesskit NENI zahrnut
] }
```

**Detection:**
Spusť profiler (perf/flamegraph) a hledej `zbus` nebo `accesskit` v call stacku při idle.

**Phase:** Fáze 1 — ověř před implementací throttlingu

---

## Moderate Pitfalls

Chyby, které způsobí neoptimální výsledky nebo zkomplikují ladění.

---

### Pitfall 5: Git polling blokuje main thread

**What goes wrong:**
`std::process::Command::output()` pro `git status` volaný na main threadu blokuje
egui renderovací smyčku po dobu běhu git procesu (typicky 20–200ms v závislosti na repo).
Výsledkem jsou frame spiky a lagující UI při každém 5s refreshi.

**Why it happens:**
`Command::output()` čeká synchronně na dokončení subprocesu.
Pokud git repo má mnoho souborů nebo je na pomalém disku/NFS, může to trvat i stovky ms.

**Consequences:**
- Viditelné sekání UI každých 5 sekund
- V horším případě: pokud git čeká na síť, může blokovat i sekundy

**Prevention:**
Git polling přesunout do background threadu s `std::thread::spawn()` nebo `tokio::spawn()`.
Výsledek předat přes `Arc<Mutex<GitState>>` nebo `std::sync::mpsc::channel`.
V `update()` pouze číst výsledek z atomicky sdílené struktury.

```rust
// Pattern: background git refresh
fn schedule_git_refresh(&self, ctx: egui::Context, shared: Arc<Mutex<GitState>>) {
    std::thread::spawn(move || {
        let status = run_git_status(); // blokujici operace v threadu
        *shared.lock().unwrap() = status;
        ctx.request_repaint(); // az hotovo, pozadej repaint
    });
}
```

**Detection:**
Přidej `println!` s timestamps kolem git volání v `workspace.rs`. Pokud gap > 5ms, je to blokující.

**Phase:** Fáze 2 (background task scheduling)

---

### Pitfall 6: std::process::Command výstup > 64KB způsobuje deadlock

**What goes wrong:**
Při čtení stdout/stderr subprocesu přes `Command::output()` nebo `Command::wait_with_output()`
hrozí deadlock, pokud subprocess vypíše > 64KB dat. Rust issue #27152 dokumentuje tento problém.

**Why it happens:**
OS pipe buffer se zaplní, subprocess čeká na čtení, ale hlavní proces čeká na ukončení —
klasický deadlock.

**Consequences:**
- Při velkém cargo build output (mnoho chyb, verbose) může build runner zamrznout
- UI přestane reagovat

**Prevention:**
Pro `build_runner.rs`: číst stdout/stderr průběžně v separátním threadu, nebo použít
`BufReader` s iterací po řádcích. Nikdy nevolat `.output()` na příkazy s potenciálně
velkým výstupem (cargo build s `--verbose`).

**Detection:**
Test: spusť `cargo build --verbose` na velkém projektu a sleduj zda editor zamrzá.

**Phase:** Fáze 2 (background task scheduling) — ovlivňuje build_runner.rs

---

### Pitfall 7: notify FileWatcher generuje repaint požadavky příliš agresivně

**What goes wrong:**
`notify` crate může generovat events ve shlucích (burst) — například při uložení souboru
editorem (atomic write = remove + create) přijde 3–10 events v milisekundách.
Pokud každý event volá `ctx.request_repaint()`, způsobuje zbytečné překreslování.

**Why it happens:**
FileWatcher poslouchá OS events a předává je bez debounce. Debounce musí implementovat
aplikace sama.

**Consequences:**
- Krátký CPU spike po každém uložení souboru
- V large projektech s mnoha soubory (npm install apod.) masivní event flood

**Prevention:**
Implementovat debounce v `watcher.rs`: events nakumulovat do bufferu,
zpracovat v dávce jednou za 100–300ms. `notify` nabízí `notify::RecommendedWatcher`
s `RecursiveMode` — přidat vlastní debounce layer.

```rust
// Debounce pattern: uloz posledni event timestamp
let last_event = Instant::now();
// V update() zpracuj jen kdyz uplynulo dost casu od posledniho eventu
if last_event.elapsed() > Duration::from_millis(200) {
    self.process_file_events();
}
```

**Detection:**
Přidej counter do FileWatcher event handleru. Pokud po uložení jednoho souboru
přijde > 3 events, chybí debounce.

**Phase:** Fáze 2 (background task scheduling)

---

### Pitfall 8: Autosave timer vytváří zbytečné dirty checks každých 500ms

**What goes wrong:**
Autosave logika v `editor.rs` kontroluje každých 500ms zda je soubor dirty a je třeba uložit.
Pokud je dirty check netriviální (porovnání velkých stringů), nebo pokud volá `request_repaint()`,
generuje zbytečnou CPU zátěž.

**Why it happens:**
Timer-based autosave je implementačně jednoduchý, ale v egui reaktivním modelu není ideální.
egui repaintuje na interakci — autosave by měl být řízen změnou stavu, ne časem.

**Prevention:**
Autosave timer: pouze `Instant::elapsed()` check bez repaint. Aktuální architektura
s `last_edit` timestampem je správná — zajisti, že samotný check nealouje ani nerepaintuje.
Pro snížení frekvence: 500ms je vhodné, ale zvažte zvýšit na 1000ms pro méně agresivní polling.

**Detection:**
Profil ukáže periodické wake-upy každých 500ms i v plně idle stavu.

**Phase:** Fáze 1 — nízká priorita, ale snadná optimalizace

---

### Pitfall 9: egui_term vyžaduje continuous repaint pro terminálový výstup

**What goes wrong:**
Terminálový emulátor (egui_term) potřebuje repaintovat, kdykoli subprocess produkuje output.
Bez správného signálování se výstup zobrazuje se zpožděním nebo jen při pohybu myší.
S nesprávnou implementací naopak repaintuje neustále i bez nového výstupu.

**Why it happens:**
egui_term musí číst PTY/pipe v background threadu a signalizovat repaint pouze
při příchodu nových dat. Pokud polling probíhá přes busy loop nebo příliš krátký interval,
CPU zátěž roste.

**Prevention:**
- Background thread čte PTY s blokujícím read (žádný busy loop)
- Po přijetí dat: `ctx.request_repaint()`
- Pokud terminal je neaktivní (žádný subprocess): žádný repaint request

**Detection:**
Zavři všechny terminálové procesy a sleduj CPU. Pokud terminal panel stále způsobuje
repainty, je to problém v implementaci.

**Phase:** Fáze 3 — specifická pro terminál, ověřit až po základním throttlingu

---

## Minor Pitfalls

---

### Pitfall 10: Profiling s puffin dává zavádějící výsledky (red herring)

**What goes wrong:**
Puffin instrumentace je sampling-based a může ukazovat velký čas u tesselatoru nebo
paint funkcí, i když skutečný bottleneck je jinde. Discussion #5199 dokumentuje případ,
kdy vývojář strávil hodiny optimalizací tessellátoru, ale skutečný problém byl jinde.

**Prevention:**
- Použij puffin pro hrubý přehled (co se vůbec volá)
- Pro skutečné bottlenecky použij sampling profiler: `perf` (Linux), Instruments (macOS), nebo `cargo flamegraph`
- Vždy měř end-to-end latency (frame time), ne jen dílčí metriky

**Detection:**
Pokud puffin ukazuje problem, ale frame time je < 2ms, je to red herring.

**Phase:** Fáze 1 (profilování) — důležité vědět před zahájením

---

### Pitfall 11: wgpu 100ms lag spike na mouse click (issue #5958)

**What goes wrong:**
S wgpu backendem může klik myší způsobit 100ms lag spike místo normálních 16ms.
To je dokumentovaný bug v eframe+wgpu kombinaci.

**Prevention:**
Zkontrolovat verzi eframe/wgpu. Pokud problém existuje, zvážit glow backend jako alternativu
pro rychlejší inicializaci. Sledovat eframe changelog pro fix.

**Detection:**
Měř frame time při kliknutí myší — pokud spike > 50ms, je to tento bug.

**Phase:** Zjistit při profilování, není prioritní

---

### Pitfall 12: Velké scroll areas zpomalují egui layout

**What goes wrong:**
egui provádí full layout každý frame pro celý obsah scroll area, i pro část mimo viewport.
Velké soubory (10000+ řádků) v editoru nebo dlouhý build error log způsobují lagující layout.

**Prevention:**
Implementovat virtual scrolling — renderovat pouze řádky v aktuálním viewportu.
egui nemá built-in virtual scroll list optimalizaci pro text editory.
Pro error list: limitovat zobrazené záznamy (např. max 200 viditelných).

**Detection:**
Otevři soubor s 5000+ řádky a sleduj frame time. Pokud > 5ms, je layout bottleneck.

**Phase:** Fáze 3 nebo pozdější — závisí na reálném dopadu

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Základní repaint throttling | Pitfall 1 (request_repaint_after ignored) | Volat pouze podmíněně, ne každý frame |
| Základní repaint throttling | Pitfall 2 (unconditional request_repaint) | Audit grep před implementací |
| Základní repaint throttling | Pitfall 4 (accesskit CPU) | Zkontrolovat Cargo.toml features |
| Git polling optimalizace | Pitfall 5 (blocking main thread) | Thread + channel pattern |
| Build runner | Pitfall 6 (64KB pipe deadlock) | BufReader streaming |
| FileWatcher | Pitfall 7 (event burst) | Debounce implementace |
| Terminál | Pitfall 9 (egui_term repaint) | Blocking PTY read v threadu |
| Profilování | Pitfall 10 (puffin red herring) | Kombinovat s perf/flamegraph |
| Multi-window | Pitfall 3 (immediate viewport) | Zachovat deferred viewport |

---

## Sources

- [High CPU usage when accesskit enabled — Issue #4527](https://github.com/emilk/egui/issues/4527) — HIGH confidence (official repo issue)
- [request_repaint_after ignored when called each frame — Issue #3109](https://github.com/emilk/egui/issues/3109) — HIGH confidence (official repo issue)
- [Reduce CPU Usage — Discussion #1261](https://github.com/emilk/egui/discussions/1261) — HIGH confidence (official repo discussion)
- [Performance Red herring and profiling with puffin — Discussion #5199](https://github.com/emilk/egui/discussions/5199) — HIGH confidence (official repo discussion)
- [Multi-Viewport System — DeepWiki](https://deepwiki.com/membrane-io/egui/6.2-multi-viewport-system) — MEDIUM confidence (third-party docs)
- [Changes to repaint signal with multi-threaded code — Issue #1379](https://github.com/emilk/egui/issues/1379) — HIGH confidence (official repo issue)
- [How to use threads/atomics properly — Discussion #1428](https://github.com/emilk/egui/discussions/1428) — HIGH confidence (official repo discussion)
- [eframe + wgpu 100ms lag spike — Issue #5958](https://github.com/emilk/egui/issues/5958) — HIGH confidence (official repo issue)
- [Windows High CPU when minimized — Issue #3982](https://github.com/emilk/egui/issues/3982) — HIGH confidence (official repo issue)
- [std::process::Command hangs >64KB output — Issue #27152](https://github.com/rust-lang/rust/issues/27152) — HIGH confidence (official Rust issue)
- [egui Context docs — request_repaint_after](https://docs.rs/egui/latest/egui/struct.Context.html) — HIGH confidence (official docs)
- [eframe NativeOptions docs](https://docs.rs/eframe/latest/eframe/struct.NativeOptions.html) — HIGH confidence (official docs)
