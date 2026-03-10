# Architecture Patterns — Rust/egui Performance Optimization

**Domain:** Desktop GUI editor (Rust + eframe/egui) — performance profiling and idle CPU reduction
**Researched:** 2026-03-04
**Overall confidence:** MEDIUM-HIGH (egui API ověřena z docs.rs a GitHub issues; egui_term specifika LOW)

---

## Recommended Architecture

Optimalizace je rozdělena do čtyř vrstev, které na sebe navazují v logickém pořadí.
Každá vrstva je nezávislá — lze je implementovat a testovat samostatně.

```
┌─────────────────────────────────────────────────────────────────┐
│  1. PROFILING LAYER (měření)                                    │
│     puffin + puffin_egui — flamegraph uvnitř editoru            │
│     Identifikuje skutečné hotspoty před optimalizací            │
└────────────────────────┬────────────────────────────────────────┘
                         │ výsledky měření informují vrstvy 2–4
┌────────────────────────▼────────────────────────────────────────┐
│  2. REPAINT GATE (podmíněné překreslování)                      │
│     ctx.request_repaint_after(Duration) místo continuous mode  │
│     Dirty-flag pro text editor — repaint jen při změně          │
└────────────────────────┬────────────────────────────────────────┘
                         │ repaint signál
┌────────────────────────▼────────────────────────────────────────┐
│  3. BACKGROUND TASK THROTTLE (omezení pozadí)                  │
│     Git polling: background thread + mpsc → jen při změně       │
│     FileWatcher: notify debounce → event burst potlačen         │
│     Autosave: dirty-flag → uloží jen pokud jsou změny           │
└────────────────────────┬────────────────────────────────────────┘
                         │ events/data přes mpsc kanály
┌────────────────────────▼────────────────────────────────────────┐
│  4. TERMINAL ISOLATION (egui_term)                              │
│     Podmíněný repaint jen při aktivním výstupu terminálu        │
│     Potlačení busy-loop alacritty backendu                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Boundaries

| Komponenta | Odpovědnost | Komunikuje s |
|------------|-------------|--------------|
| **ProfilingLayer** | Instrumentace kódu, flamegraph UI panel | egui Context, všechny ostatní komponenty |
| **RepaintGate** | Rozhoduje kdy zavolat `ctx.request_repaint()` nebo `request_repaint_after()` | egui Context, WorkspaceState |
| **GitPoller** (background thread) | Spouští `git status` v intervalu, posílá výsledky | mpsc::Sender → WorkspaceState |
| **FileWatcherBridge** | Přijímá notify events, debounce, posílá do UI | notify Watcher → mpsc → EditorApp |
| **AutosaveDirtyFlag** | Sleduje `is_dirty` flag, ukládá jen při změně | TabState v editor.rs |
| **TerminalRepaintGuard** | Detekuje aktivní výstup terminálu, omezuje zbytečné repainty | egui_term widget |

---

## Data Flow — Repaint Decisions

```
Událost vstupuje do systému
         │
         ▼
┌────────────────────┐    Ne     ┌─────────────────────────────┐
│  Přišel vstup      │──────────►│  Plánovaný interval vypršel? │
│  od uživatele?     │           └──────────┬──────────────────┘
└────────┬───────────┘                      │ Ano
         │ Ano                              ▼
         │                    ┌────────────────────────────────┐
         │                    │  Změnila se data v pozadí?     │
         │                    │  (git status, file event, …)   │
         │                    └──────────┬─────────────────────┘
         │                               │ Ano
         │                               ▼
         └──────────────────►  ctx.request_repaint()
                                         │
                                         ▼
                             egui render loop (update())
                                         │
                             ┌───────────▼───────────┐
                             │  Jsou animace aktivní? │
                             │  (cursor blink, toast) │
                             └───────────┬───────────┘
                                         │ Ano
                                         ▼
                             request_repaint_after(~16ms)
                                         │ Ne
                                         ▼
                             request_repaint_after(5s)   ← idle stav
```

**Klíčový princip:** Bez uživatelského vstupu nebo dat z pozadí se neprovede žádný repaint dřív než za 5 sekund. Oproti současnému continuous mode (60fps) jde o dramatické snížení CPU.

---

## Patterns to Follow

### Pattern 1: Conditional Repaint Gate

**Co:** Nahrazení continuous render loop podmíněným repaintem pomocí `request_repaint_after`.
**Kdy:** Vždy — toto je základní optimalizace pro idle CPU.
**Confidence:** HIGH (ověřeno v docs.rs egui, GitHub issues #1261, #2008)

```rust
// V App::update(), na konci frame:
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // ... render UI ...

    // Animace aktivní (cursor blink, toast fadein)?
    if self.has_active_animation() {
        ctx.request_repaint(); // ihned — animace potřebuje každý frame
    } else if self.has_pending_background_data() {
        ctx.request_repaint(); // ihned — přišla nová data
    } else {
        // Idle: repaint max jednou za 5s (catch-all pro git, file events)
        ctx.request_repaint_after(Duration::from_secs(5));
    }
}
```

**Důležité:** `request_repaint_after` akceptuje nejmenší hodnotu ze všech volání v daném frame. Volat s velkým intervalem jako fallback je bezpečné.

---

### Pattern 2: Background Git Poller s mpsc

**Co:** Git polling přesunout do background threadu, výsledky přenést přes `mpsc::channel`.
**Kdy:** Git polling probíhající na hlavním vlákně blokuje render loop.
**Confidence:** MEDIUM (pattern ověřen v egui discussions #1428, konkrétní git integrace LOW)

```rust
// Inicializace (jednou při spuštění workspace):
let (git_tx, git_rx) = mpsc::channel::<GitStatus>();
let ctx_clone = ctx.clone(); // egui Context je Clone

thread::spawn(move || {
    loop {
        let status = compute_git_status(&project_path);
        if git_tx.send(status).is_err() { break; }
        ctx_clone.request_repaint(); // probudí UI thread
        thread::sleep(Duration::from_secs(5));
    }
});

// V update():
if let Ok(status) = self.git_rx.try_recv() { // NESMÍ blokovat!
    self.git_status = status;
}
```

**Omezení:** `egui::Context` lze bezpečně klonovat a volat `request_repaint()` z jiného vlákna (ověřeno v egui docs). Nevolat `ctx.request_repaint()` z threadu příliš agresivně — stačí jednou po dokončení operace.

---

### Pattern 3: Dirty Flag pro Autosave

**Co:** Autosave timer kontroluje `is_dirty` flag místo vždy ukládat.
**Kdy:** Autosave timer běží každých 500ms — optimalizace zredukuje I/O operace.
**Confidence:** HIGH (standardní game/editor pattern, docs.rs egui `auto_save_interval`)

```rust
struct TabState {
    content: String,
    is_dirty: bool,          // nastaveno na true při každé editaci
    last_saved: Instant,
}

// V update(), autosave sekce:
if tab.is_dirty && tab.last_saved.elapsed() > Duration::from_millis(500) {
    save_to_disk(&tab.path, &tab.content)?;
    tab.is_dirty = false;
    tab.last_saved = Instant::now();
    // Nepotřebujeme request_repaint — stav se vizuálně mění jen u '●' indikátoru
}
```

---

### Pattern 4: FileWatcher Debounce

**Co:** Použít `notify-debouncer-full` nebo `notify-debouncer-mini` místo raw notify events.
**Kdy:** Raw notify emituje burst events při každém uložení souboru (Changed + DataChange + Metadata).
**Confidence:** HIGH (ověřeno v notify docs a Rust forum)

```rust
// Místo raw Watcher použít debouncer s 200ms oknem:
let (tx, rx) = mpsc::channel();
let mut debouncer = new_debouncer(Duration::from_millis(200), None, tx)?;
debouncer.watcher().watch(path, RecursiveMode::Recursive)?;

// V update():
while let Ok(events) = self.watcher_rx.try_recv() {
    for event in events {
        self.handle_file_event(event);
    }
    ctx.request_repaint(); // jen pokud přišly eventy
}
```

---

### Pattern 5: Puffin Profiling (Feature Flag)

**Co:** Instrumentace hotspotů pomocí puffin maker, vizualizace přes puffin_egui.
**Kdy:** Před zahájením optimalizací — měřit nejdřív, pak opravovat.
**Confidence:** HIGH (EmbarkStudios/puffin, crates.io puffin_egui 0.29.0)

```toml
# Cargo.toml — profiling za feature flagem (bez dopadu na release build)
[features]
profiling = ["puffin", "puffin_egui"]

[dependencies]
puffin = { version = "0.19", optional = true }
puffin_egui = { version = "0.29", optional = true }
```

```rust
// V update():
#[cfg(feature = "profiling")]
puffin::GlobalProfiler::lock().new_frame();

// V konkrétních funkcích:
fn render_file_tree(&mut self, ui: &mut egui::Ui) {
    #[cfg(feature = "profiling")]
    puffin::profile_function!();
    // ...
}
```

Spuštění: `cargo run --features profiling`

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Continuous Mode bez potřeby

**Co se děje:** `ctx.request_repaint()` voláno každý frame bez podmínky.
**Proč špatně:** Nutí GPU/CPU renderovat 60fps i v idle stavu. Na notebooku způsobuje zahřívání a zkrácení výdrže baterie.
**Místo toho:** Použít Pattern 1 (Conditional Repaint Gate).

---

### Anti-Pattern 2: Blokující operace v update()

**Co se děje:** `git status`, I/O operace, nebo `thread::sleep()` v render loop.
**Proč špatně:** Blokuje UI thread — frame drop, stuttering, nereaktivní UI.
**Místo toho:** Přesunout do background threadů s mpsc komunikací (Pattern 2).

---

### Anti-Pattern 3: try_recv() bez limitu v update()

**Co se děje:** Čtení všech zpráv z kanálu v jednom frame bez omezení počtu.
**Proč špatně:** Pokud kanál přetéká (file watcher burst), update() tráví příliš mnoho času zpracováním zpráv místo renderování.
**Místo toho:** Číst max N zpráv za frame nebo použít debounce na straně producenta.

```rust
// Max 10 file events za frame:
let mut processed = 0;
while processed < 10 {
    match self.watcher_rx.try_recv() {
        Ok(event) => { self.handle_file_event(event); processed += 1; }
        Err(_) => break,
    }
}
```

---

### Anti-Pattern 4: Optimalizovat bez měření

**Co se děje:** Přepisovat git polling, autosave nebo FileWatcher "od oka" bez dat.
**Proč špatně:** Skutečný viník může být jinde (egui_term busy loop, syntect parsing, AccessKit).
**Místo toho:** Puffin profiling jako první krok, pak cílená optimalizace.

---

## Suggested Implementation Order

Toto pořadí minimalizuje riziko a maximalizuje měřitelný dopad:

| Krok | Komponenta | Rationale |
|------|------------|-----------|
| 1 | Puffin profiling setup | Bez dat nevíme co opravit — měřit nejdřív |
| 2 | Conditional Repaint Gate | Největší dopad na idle CPU, nejméně riskantní změna |
| 3 | Git Poller → background thread | Druhý nejvyšší viník, přímá implementace |
| 4 | FileWatcher debounce | Nízké riziko, crates.io řešení existuje |
| 5 | Autosave dirty flag | Jednoduché, ale malý dopad pokud je I/O rychlé |
| 6 | egui_term CPU audit | HIGH riziko — alacritty backend; nejdřív změřit |

---

## Scalability Considerations

| Concern | Malý projekt (~100 souborů) | Velký projekt (~10K souborů) |
|---------|------------------------------|------------------------------|
| FileWatcher overhead | Zanedbatelný | notify event burst může být výrazný — debounce nutný |
| Git polling | git status rychlý (<50ms) | git status může trvat 100–500ms — background thread kritický |
| Syntax highlighting | OK | Syntect parsování velkých souborů může blokovat UI — zvážit lazy highlight |
| Fuzzy file picker (Ctrl+P) | OK | Potenciálně pomalý pro 10K souborů — indexování na pozadí |

---

## Known Risks and Unknowns

### egui_term (HIGH uncertainty)

egui_term je postavený na alacritty backendu, který má zdokumentované problémy s busy-loop při čekání na výstup terminálu (alacritty/alacritty issue #8413). Bez profilování nelze říci, zda je to největší viník CPU zátěže v PolyCredo Editoru. Doporučeno: změřit puffinem jako prioritu.

### AccessKit

Pokud je AccessKit feature povolena, způsobuje vysoké idle CPU (egui issue #4527). Zkontrolovat Cargo.toml a případně vypnout/podmínit.

### egui request_repaint_after + deferred viewports

Existuje bug (egui issue #4945): animace s `request_repaint()` nefunguje správně při otevřeném deferred viewport. PolyCredo Editor používá `show_viewport_deferred` — toto může být relevantní při implementaci. Důkladně testovat.

---

## Sources

- [egui Context docs — request_repaint_after](https://docs.rs/egui/latest/egui/struct.Context.html) — HIGH confidence
- [egui Discussion #1261 — Reduce CPU Usage](https://github.com/emilk/egui/discussions/1261) — MEDIUM confidence
- [egui Issue #2008 — High CPU when window focused](https://github.com/emilk/egui/issues/2008) — MEDIUM confidence
- [egui Issue #4527 — High CPU with accesskit](https://github.com/emilk/egui/issues/4527) — MEDIUM confidence
- [egui Issue #4945 — Animation + deferred viewport bug](https://github.com/emilk/egui/issues/4945) — MEDIUM confidence
- [egui Issue #3109 — request_repaint_after ignored if called each frame](https://github.com/emilk/egui/issues/3109) — HIGH confidence (known gotcha)
- [egui Discussion #1428 — How to use threads/atomics](https://github.com/emilk/egui/discussions/1428) — HIGH confidence
- [EmbarkStudios/puffin GitHub](https://github.com/EmbarkStudios/puffin) — HIGH confidence
- [puffin_egui 0.29.0 on crates.io](https://crates.io/crates/puffin_egui) — HIGH confidence
- [notify-debouncer-full docs](https://docs.rs/notify-debouncer-full) — HIGH confidence
- [alacritty busy-loop issue #8413](https://github.com/alacritty/alacritty/issues/8413) — MEDIUM confidence
- [Dirty Flag pattern — Game Programming Patterns](https://gameprogrammingpatterns.com/dirty-flag.html) — HIGH confidence (obecný pattern)
