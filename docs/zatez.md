# Analýza zatížení CPU — PolyCredo Editor

**Datum:** 2026-02-22
**Symptom:** CPU roste z ~23 % a postupně dosáhne ~97 %

---

## Příčiny a jejich závažnost

### 🔴 KRITICKÉ

#### 1. `get_staged_files()` volaná na každém UI framu

**Soubor:** `src/app/ui/workspace/mod.rs:335`
**Volání:** `render_sandbox_staged_bar()` → `ws.sandbox.get_staged_files()`

Funkce `get_staged_files()` provádí při každém vykreslení UI (cca 60× za sekundu):
- `WalkDir` průchod celým sandbox adresářem
- `WalkDir` průchod celým projektem
- `fs::metadata()` na každém souboru
- `fs::read_to_string()` na souborech, které jsou v sandboxu novější

Pro projekt se stovkami souborů jde o tisíce diskových operací za sekundu.

**Řešení:**
- Přidat do `Sandbox` metodu `spawn_get_staged_files() -> mpsc::Receiver<Vec<PathBuf>>`, která spustí výpočet v background vlákně.
- Do `WorkspaceState` přidat pole:
  - `staged_files_cache: Vec<PathBuf>` — naposledy spočítaný výsledek
  - `staged_files_rx: Option<mpsc::Receiver<Vec<PathBuf>>>` — čekající výsledek z vlákna
  - `staged_files_dirty: bool` — příznak, že cache je neplatná
- V `process_background_events()` (background.rs) poll `staged_files_rx` a spouštět nový výpočet, když je `staged_files_dirty == true`.
- Cache označit jako dirty při každé `FsChange` události ze sandbox adresáře.
- V `render_sandbox_staged_bar()` používat `ws.staged_files_cache.clone()` místo přímého volání.

---

#### 2. Vynucený repaint každých 500 ms

**Soubor:** `src/config.rs:7` + `src/app/ui/workspace/mod.rs:62`
```rust
ctx.request_repaint_after(Duration::from_millis(config::REPAINT_INTERVAL_MS));
```

Každý workspace volá `request_repaint_after(500ms)` na **každém framu** → UI se nikdy neuspí.
Terminál a LSP si spravují vlastní repainty — tento interval slouží jen jako fallback pro autosave a watcher.

**Řešení:**
Zvýšit `REPAINT_INTERVAL_MS` z `500` na `2000` (nebo více).

---

#### 3. Polling git procesů — busy-wait smyčka

**Soubor:** `src/app/ui/background.rs:255`
```rust
Ok(None) => std::thread::sleep(Duration::from_millis(25)),
```

`wait_for_child_stdout()` čeká na dokončení `git status` / `git rev-parse` v backgroundu s 25ms sleep smyčkou. Samotné git příkazy jsou v odděleném vlákně, takže to neblokuje UI, ale přidává zátěž při každém 5sekundovém refreshi.

`git status --porcelain=v1 -z --untracked-files=all` na velkém repozitáři může trvat 100–500 ms.

**Řešení:**
- Prodloužit git refresh interval z 5 s na 10–15 s (v background.rs:205).
- Alternativně použít `child.wait()` s timeoutem místo poll smyčky.

---

#### 4. LSP diagnostiky — request_repaint každých 100 ms

**Soubor:** `src/app/lsp/mod.rs`

Při každé diagnostické zprávě z rust-analyzeru (při indexování projekt = stovky zpráv):
```rust
egui_ctx_handler.request_repaint_after(Duration::from_millis(100));
```
Během inicializace LSP = desítky repaintů za sekundu navíc.

**Řešení:**
Throttle na úrovni LSP handleru — zkontrolovat, zda od posledního repaintu neuplynulo méně než 500 ms.

---

### 🟡 SEKUNDÁRNÍ

#### 5. Terminal — 256 PTY eventů zpracováno synchronně na frame

**Soubor:** `src/config.rs:49`, `src/app/ui/terminal.rs`
```rust
pub const TERMINAL_MAX_EVENTS_PER_FRAME: usize = 256;
```
Při `cargo build` terminál přijímá bursts tisíců řádků → 256 zpracování + plné překreslení terminálu každý frame.

**Řešení:** Limit 64 eventů/frame + dirty tracking aby se terminál překresloval jen při změně.

---

#### 6. Project index — full rescan při každé FsChange

**Soubor:** `src/app/ui/workspace/index.rs`

Při každé změně souboru se spouští `full_rescan()` v novém vlákně. Při hromadném zápisu souborů (AI agent) může vzniknout desítky vláken najednou.

**Řešení:** Debounce + jediný sdílený thread pool (nebo `std::sync::OnceLock` na rescan).

---

## Pořadí doporučených oprav

| Pořadí | Soubor | Změna | Odhadovaný dopad |
|--------|--------|-------|-----------------|
| 1 | `sandbox.rs` + `state.rs` + `background.rs` + `workspace/mod.rs` | Cache `get_staged_files()` do background threadu | −30 % CPU |
| 2 | `config.rs` | `REPAINT_INTERVAL_MS: 500 → 2000` | −15 % CPU |
| 3 | `background.rs:205` | git refresh interval: 5 s → 15 s | −5 % CPU |
| 4 | `lsp/mod.rs` | throttle repaint na 500 ms | −8 % CPU (při LSP init) |
| 5 | `terminal.rs` + `config.rs` | limit eventů 256 → 64 | −5 % CPU (při buildu) |

---

## Aktualizace 2026-02-22

### ✅ Potvrzená hlavní příčina

#### 7. Drahý full-scan sandbox staged souborů v render smyčce

**Soubory:** `src/app/sandbox.rs:105`, `src/app/ui/workspace/mod.rs:350`, `src/app/ui/workspace/modal_dialogs/ai.rs:43`

`Sandbox::get_staged_files()` dělá kompletní průchod přes sandbox i projekt (`WalkDir`) a u některých souborů i porovnání obsahu (`read_to_string`).
Tento scan byl volaný z UI toku opakovaně (staged bar + modal), což při častějším repaintu způsobovalo výrazný růst CPU.

### ✅ Implementované opatření

**Soubory:** `src/app/ui/workspace/state.rs`, `src/app/ui/workspace/mod.rs`, `src/config.rs`, `src/app/ui/workspace/modal_dialogs/ai.rs`

- Přidána cache staged souborů do `WorkspaceState`:
  - `sandbox_staged_files`
  - `sandbox_staged_last_refresh`
- Přidán interval obnovy cache:
  - `SANDBOX_STAGED_REFRESH_MS = 3000` (`src/config.rs:57`)
- Render už nevolá full-scan na každý frame:
  - staged bar i staged modal čtou z cache
  - cache se obnovuje periodicky (interval) a okamžitě po `promote` akcích

### Ověření

- `cargo check` proběhl bez chyb.
- Očekávaný efekt: výrazné snížení CPU zátěže zejména v projektech se sandbox změnami.
