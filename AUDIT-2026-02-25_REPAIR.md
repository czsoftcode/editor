# Návrh oprav — PolyCredo Editor (CPU Audit)

**Datum:** 2026-02-25
**Verze:** 0.7.9
**Zdroje:** AUDIT-2026-02-25_01.md, AUDIT-2026-02-25_02.md, AUDIT-2026-02-25_03.md
**Cíl:** Snížit idle CPU zátěž z ~10 % na < 1 %

---

## Prioritizace oprav

| # | Soubor | Dopad | Obtížnost | Priorita |
|---|--------|-------|-----------|----------|
| 1 | `vendor/egui_term/src/backend/mod.rs:196` | Velmi vysoký | Nízká | KRITICKÁ |
| 2 | `vendor/egui_term/src/view.rs:132` | Střední | Střední | VYSOKÁ |
| 3 | `src/app/ui/workspace/mod.rs:84–93` | Vysoký | Nízká | VYSOKÁ |
| 4 | `src/app/ui/workspace/mod.rs:99` | Nízký | Nízká | STŘEDNÍ |
| 5 | `src/app/ui/workspace/state.rs:296–304` | Vysoký | Střední | VYSOKÁ |
| 6 | `src/app/ui/background.rs:342` | Střední | Nízká | STŘEDNÍ |
| 7 | `src/app/ui/workspace/state.rs:282–283` | Nízký | Nízká | NÍZKÁ |

---

## Oprava 1 — egui_term: throttlovat `request_repaint()` (KRITICKÁ)

**Soubor:** `vendor/egui_term/src/backend/mod.rs:196`

**Problém:**
Každý PTY event (idle shell generuje desítky za sekundu: cursor blink, readline,
escape sekvence) volá `request_repaint()` bez zpoždění. Se dvěma terminály
(Claude panel + build terminál) to způsobuje desítky repaintů za sekundu i v naprostém klidu.

**Stávající kód:**
```rust
app_context.clone().request_repaint();
```

**Navrhovaná oprava:**
```rust
app_context.clone().request_repaint_after(std::time::Duration::from_millis(16));
```

**Odůvodnění:**
16 ms odpovídá 60 fps — dostačující odezva pro terminál. Místo stovek repaintů
za sekundu se UI překreslí nejvýše 60×, ale pouze pokud skutečně přišel PTY event.
Vizuálně není změna patrná.

**Odhadované snížení CPU:** 60–70 %

---

## Oprava 2 — egui_term: resize pouze při skutečné změně velikosti

**Soubor:** `vendor/egui_term/src/view.rs:132`

**Problém:**
`resize()` je voláno každý frame v rámci renderovací smyčky (`show()` metoda).
Uvnitř zavolá `process_command(BackendCommand::Resize(...))`, které nastaví
`dirty = true` bezpodmínečně (`backend/mod.rs:217`). Tím se označí obsah
terminálu jako změněný v každém framu, i když terminál stojí.

**Stávající kód:**
```rust
fn resize(self, layout: &Response) -> Self {
    self.backend.process_command(BackendCommand::Resize(
        Size::from(layout.rect.size()),
        self.font.font_measure(&layout.ctx),
    ));
    self
}
```

**Navrhovaná oprava:**
Uložit poslední velikost do `TerminalView` (nebo `TerminalViewState`) a volat
`process_command` pouze pokud se `rect.size()` změnilo oproti předchozímu framu:

```rust
fn resize(self, layout: &Response) -> Self {
    let new_size = Size::from(layout.rect.size());
    let font_measure = self.font.font_measure(&layout.ctx);
    // Volat Resize pouze pokud se velikost skutečně změnila:
    if self.backend.last_size() != new_size {
        self.backend.process_command(BackendCommand::Resize(new_size, font_measure));
    }
    self
}
```

Do `TerminalBackend` přidat pole `last_size: Size` a getter `last_size()`.
`process_command` ho aktualizuje při `BackendCommand::Resize`.

**Odhadované snížení CPU:** 10–20 % (eliminace zbytečného `dirty=true` každý frame)

---

## Oprava 3 — Lazy inicializace terminálů podle viditelnosti panelů (VYSOKÁ)

**Soubor:** `src/app/ui/workspace/mod.rs:84–93`

**Problém:**
Oba terminálové procesy (shell pro AI CLI, shell pro build terminál) jsou
spouštěny automaticky při každém otevřeném workspace — bez ohledu na to,
zda jsou příslušné panely vůbec viditelné. Každý shell generuje PTY eventy
i když uživatel terminál nevidí ani nepoužívá.

**Stávající kód:**
```rust
if ws.claude_tabs.is_empty() {
    ws.claude_tabs.push(Terminal::new(id, ctx, &root, None));
}
if ws.build_terminal.is_none() {
    ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
}
```

**Navrhovaná oprava:**
```rust
if ws.show_right_panel && ws.claude_tabs.is_empty() {
    ws.claude_tabs.push(Terminal::new(id, ctx, &root, None));
}
if ws.show_build_terminal && ws.build_terminal.is_none() {
    ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
}
```

**Poznámka:**
Terminál se po prvním otevření panelu inicializuje a dál zůstane živý — tj. opakované
skrývání/zobrazování panelu terminál nevraždí ani neznovuspouští. To je žádoucí chování.

**Odhadované snížení CPU při výchozím stavu se skrytými panely:** 30–40 %
Pokud jsou panely viditelné ve výchozím stavu (výchozí hodnoty v `PersistentState`),
dopad je nulový — viz oprava 3b.

---

## Oprava 3b — Výchozí stav: terminálové panely skryté

**Soubor:** `src/app/types.rs:100–101`

**Problém:**
`show_right_panel: true` a `show_build_terminal: true` jsou výchozí hodnoty,
takže terminály se spustí okamžitě i po opravě č. 3.

**Navrhovaná oprava:**
```rust
impl Default for PersistentState {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_right_panel: false,   // ← změna: AI panel defaultně skryt
            show_build_terminal: false, // ← změna: build terminál defaultně skryt
            ...
        }
    }
}
```

**Odůvodnění:**
Uživatel si panely otevře, když je potřebuje. Výchozí otevřené terminály
způsobují zbytečnou zátěž od startu. Preference se uloží do `settings.json`
a po prvním otevření zůstane nastavení zachováno.

---

## Oprava 4 — Podmíněný repaint místo pevného intervalu 2 sekundy

**Soubor:** `src/app/ui/workspace/mod.rs:99–101`

**Problém:**
`request_repaint_after(2000ms)` nutí egui překreslit UI každé 2 sekundy
i při nulové aktivitě. Samotné je to přijatelné, ale v kombinaci s terminály
přidává zálohu pro případ, že by egui přestalo reagovat. Pokud jsou terminály
opraveny, může být interval prodloužen.

**Navrhovaná oprava:**
Podmínit repaint aktivními operacemi na pozadí. Pokud nic neběží, nechat
egui v klidu (egui само překreslí při uživatelské interakci):

```rust
let has_active_work = ws.build_runner.is_running()
    || ws.semantic_index.lock().unwrap().is_indexing()
    || ws.git_status_rx.is_some()
    || ws.git_branch_rx.is_some()
    || ws.ai_cli.is_waiting_for_response();

if has_active_work {
    ctx.request_repaint_after(std::time::Duration::from_millis(250));
}
```

Pokud žádná aktivní operace neběží, egui přirozeně překreslí pouze při
uživatelské interakci (pohyb myši, klávesa).

---

## Oprava 5 — Lazy + debounced sémantický indexer

**Soubory:**
- `src/app/ui/workspace/state.rs:296–504`
- `src/app/ui/workspace/semantic_index.rs`

**Problém:**
Sémantická indexace (BERT embeddingy přes `candle` na CPU) se spouští
automaticky při každém startu workspace. U větších projektů to trvá desítky
sekund a vytíží CPU na 100 %. Navíc `std::thread::sleep(2ms)` mezi chunky
(řádek 491) tlumí zátěž jen minimálně.

**Navrhované opravy (seřazeno podle priorit):**

### 5a — Toggle sémantické indexace v nastavení

Do `Settings` přidat pole:
```rust
pub semantic_index_enabled: bool, // default: true
```

Na začátku indexovacího vlákna přidat kontrolu. Uživatel může indexaci
vypnout v nastavení aplikace, pokud ji nepotřebuje.

### 5b — Prodloužit sleep mezi chunky

Aktuální `sleep(2ms)` je příliš krátké. Prodloužit na `sleep(10ms)`, aby
OS mohl přeplánovat vlákno a CPU se ochladil:

```rust
std::thread::sleep(std::time::Duration::from_millis(10)); // bylo 2ms
```

Při typickém projektu (500 chunků) to prodlouží indexaci o ~4 sekundy,
ale výrazně sníží tepelnou zátěž.

### 5c — Debounce re-indexace po změně souborů

Místo okamžité re-indexace při každém file watcheru eventu přidat
debounce 30 sekund:

```rust
const REINDEX_DEBOUNCE_SECS: u64 = 30;
if ws.last_sandbox_change.elapsed().as_secs() > REINDEX_DEBOUNCE_SECS
    && !ws.semantic_index.lock().unwrap().is_indexing() {
    // spustit re-indexaci
}
```

### 5d — Spouštět indexaci až po prodlevě od startu

Místo okamžitého spuštění při `new()` odložit start indexace o 5 sekund
po otevření workspace — uživatel má čas začít pracovat, CPU se neblokuje
hned při startu.

---

## Oprava 6 — Prodloužit nebo event-driven Git polling

**Soubor:** `src/app/ui/background.rs:342`

**Problém:**
`git status` a `git rev-parse` jsou spouštěny každých 10 sekund. V rozsáhlých
repozitářích jde o pravidelné I/O špičky.

**Varianta A — Prodloužení intervalu (jednodušší):**
```rust
if ws.git_last_refresh.elapsed().as_secs() > 30 { // bylo 10
```

**Varianta B — Event-driven přístup (doporučeno):**
Přidat `.git/index` do file watcheru. Při jeho změně (= git commit, checkout,
stage) spustit refresh okamžitě. Periodický polling pak může být prodloužen
na 60 sekund jako záloha.

Tím se git status aktualizuje okamžitě po skutečné git operaci, ale nevyvolává
zbytečné procesy každých 10 sekund.

---

## Oprava 7 — Eliminace duplicitního file watcheru pro sandbox

**Soubor:** `src/app/ui/workspace/state.rs:282–283`

**Problém:**
```rust
let mut project_watcher = ProjectWatcher::new(&root_path);
project_watcher.add_path(&sandbox.root);  // ← sandbox je subdir root_path
```

Pokud je `sandbox.root` subdirectory `root_path`, rekurzivní watcher na
`root_path` již sandbox pokrývá. Explicitní `add_path` sandbox způsobuje
duplicitní eventy.

**Navrhovaná oprava:**
Před voláním `add_path` ověřit, že `sandbox.root` není prefix `root_path`:
```rust
if !sandbox.root.starts_with(&root_path) {
    project_watcher.add_path(&sandbox.root);
}
```

Pokud je sandbox vždy uvnitř projektu (`.polycredo/sandbox`), lze `add_path`
odstranit úplně.

---

## Souhrn a očekávaný dopad

| Oprava | Soubor | CPU úspora (odhad) |
|--------|--------|-------------------|
| 1 — egui_term throttle repaint | `vendor/egui_term/src/backend/mod.rs` | 60–70 % |
| 2 — egui_term resize guard | `vendor/egui_term/src/view.rs` | 10–20 % |
| 3 — Lazy init terminálů | `src/app/ui/workspace/mod.rs` | 20–40 % |
| 3b — Výchozí skryté panely | `src/app/types.rs` | doplněk k #3 |
| 4 — Podmíněný repaint | `src/app/ui/workspace/mod.rs` | 5–10 % |
| 5 — Debounced indexer | `src/app/ui/workspace/state.rs` | 15–30 % (při startu) |
| 6 — Git polling interval | `src/app/ui/background.rs` | 2–5 % |
| 7 — Watcher deduplikace | `src/app/ui/workspace/state.rs` | 1–3 % |

**Při implementaci oprav 1 + 3 + 3b** (nejmenší zásahy, největší dopad) se
očekává pokles idle CPU z ~10 % na **< 2 %**.

Po implementaci všech oprav by idle CPU měl být **< 0.5 %** — srovnatelné
s ostatními egui/rust editory.

---

## Doporučené pořadí implementace

1. **Oprava 1** — `vendor/egui_term/src/backend/mod.rs:196`
   Jedna řádka, obrovský dopad. Implementovat jako první.

2. **Oprava 3 + 3b** — `workspace/mod.rs` + `types.rs`
   Dvě řádky v inicializaci + změna výchozích hodnot.

3. **Oprava 5b + 5c** — prodloužení sleep + debounce re-indexace
   Minimální změny, výrazně snižují tepelnou zátěž při práci AI agenta.

4. **Oprava 6** — prodloužit git interval na 30s
   Jedna řádka.

5. **Oprava 2** — resize guard v egui_term
   Vyžaduje přidání stavu do `TerminalBackend`, střední obtížnost.

6. **Oprava 4** — podmíněný repaint
   Vyžaduje koordinaci stavu napříč moduly.

7. **Oprava 5a + 5d** — UI toggle + odložený start indexace
   Větší zásah, nižší priorita.

8. **Oprava 7** — watcher deduplikace
   Nejmenší dopad, implementovat na závěr.
3