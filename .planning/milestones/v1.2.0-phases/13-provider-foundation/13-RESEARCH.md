# Phase 13: Provider Foundation - Research

**Researched:** 2026-03-06
**Domain:** Rust HTTP streaming, Ollama API, trait-based provider abstraction
**Confidence:** HIGH

## Summary

Phase 13 implementuje AI provider abstrakci pro PolyCredo Editor. Jde o vytvoreni `AiProvider` traitu a prvni implementace `OllamaProvider`, ktery komunikuje s Ollama serverem pres jeho nativni REST API. Klicove je NDJSON streaming pres `ureq` + `std::thread` s vysledky predavanymi pres `mpsc::Receiver<StreamEvent>` -- pattern jiz overeny v codebase (build_error_rx, lsp_hover_rx, ai_tool_check_rx).

Codebase jiz obsahuje `src/app/ai/` modul s `AiManager`, `AiMessage`, `AiConversation` a dalsimi typy. Existujici Ollama WASM plugin (`src/plugins/ollama/`) slouzi jako reference pro API format, ale novy nativni provider je zcela oddeleny. Ureq 2.x je jiz v lockfile jako transitivni zavislost -- je potreba pridat jako primou zavislost do Cargo.toml.

**Primary recommendation:** Pridat `ureq` do Cargo.toml, vytvorit `AiProvider` trait + `OllamaProvider` v `src/app/ai/`, streaming pres `BufReader::lines()` na `std::thread`, vysledky pres `mpsc::channel`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Periodicky polling kazdych 10s na pozadi (GET /api/tags) s okamzitym checkem pri spusteni
- Status ikona v AI baru (zelena/cervena/seda)
- URL Ollama serveru konfigurovatelna pres existujici nastaveni agentu v Settings
- Refresh seznamu modelu spolu s detekci -- jeden request GET /api/tags
- Prazdny stav: ComboBox zobrazi "No models available" + tooltip
- Vychozi model: zapamatovat posledni zvoleny model v settings, fallback na prvni v seznamu
- ComboBox zobrazuje jen nazev modelu (bez velikosti/rodiny)
- Chyby prezentovany inline v chatu jako cerveny blok
- Pri preruseni streamu: zachovat castecnou odpoved + pripojit chybovy blok
- Tlacitko "Retry" pod chybovym blokem
- Zadny automaticky retry
- Trait stredniho rozsahu: name(), available_models(), send_chat(), stream_chat(), is_available(), config(), capabilities()
- capabilities() vraci struct (supports_streaming, supports_tools)
- stream_chat() vrati mpsc::Receiver<StreamEvent>
- StreamEvent enum: Token, Done, Error, ToolCall (ToolCall pripravena ale neimplementovana do Phase 16)
- Rozsirit existujici src/app/ai/ modul

### Claude's Discretion
- Konkretni StreamEvent enum design a pole
- Vnitrni buffering strategie pro NDJSON parsing
- Timeout hodnoty pro ureq requesty
- Presny design capabilities() struct

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROV-01 | AiProvider trait s metodami send_chat(), stream_chat(), name(), available_models() | Trait design v Architecture Patterns, plus is_available(), config(), capabilities() z CONTEXT.md |
| PROV-02 | OllamaProvider implementuje AiProvider s NDJSON streaming pres ureq + std::thread | ureq 2.x `into_reader()` + `BufReader::lines()` pattern, Ollama /api/chat NDJSON format |
| PROV-03 | Auto-detect Ollama serveru na localhost:11434 (GET /api/tags) | Polling pattern z existujiciho `spawn_ai_tool_check`, /api/tags response format |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ureq | 2.12 | Blocking HTTP client | Jiz v lockfile jako transitivni dep, matches codebase "no tokio for HTTP" rozhodnuti |
| serde + serde_json | 1.x | JSON serialization/deserialization | Jiz v Cargo.toml, pouzivano vsude v projektu |
| std::sync::mpsc | stdlib | Channel pro streaming eventy | Existujici codebase pattern (build_error_rx, lsp_hover_rx) |
| std::thread | stdlib | Background HTTP requesty | Existujici codebase pattern, ne tokio |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::io::BufReader | stdlib | Line-by-line NDJSON cteni | Streaming /api/chat response |
| std::time::Instant | stdlib | Polling timer | Casovani 10s intervalu detekce |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ureq | reqwest | reqwest vyzaduje tokio runtime, codebase pouziva std::thread |
| mpsc::channel | crossbeam-channel | Nepotrebne, std mpsc staci pro one-producer-one-consumer |
| BufReader::lines() | custom NDJSON parser | lines() je dostatecne -- kazdy NDJSON radek je jeden JSON objekt |

**Installation:**
```bash
# Pridat do Cargo.toml [dependencies]:
# ureq = { version = "2", features = ["json"] }
# (serde, serde_json jiz pritomny)
```

## Architecture Patterns

### Recommended Project Structure
```
src/app/ai/
  mod.rs             -- existujici AiManager, pridana re-exporty
  types.rs           -- existujici typy (AiMessage, AiConversation, ...)
  tools.rs           -- existujici tool deklarace
  provider.rs        -- NOVY: AiProvider trait + ProviderCapabilities + StreamEvent
  ollama.rs          -- NOVY: OllamaProvider implementace
```

### Pattern 1: AiProvider Trait
**What:** Trait abstrakce pro AI providery s async streaming pres mpsc
**When to use:** Kazda interakce s AI backendem

```rust
// src/app/ai/provider.rs

/// Capabilities reportovane providerem
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
}

/// Konfigurace providera
pub struct ProviderConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub num_ctx: u64,
}

/// Eventy z streaming odpovedi
pub enum StreamEvent {
    /// Novy token textu
    Token(String),
    /// Stream dokoncen, obsahuje metadata
    Done {
        model: String,
        prompt_tokens: u64,
        completion_tokens: u64,
    },
    /// Chyba behem streamu
    Error(String),
    /// Tool call request (Phase 16)
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
}

/// Provider trait -- vsechny metody jsou blocking (volat z background threadu)
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn available_models(&self) -> Result<Vec<String>, String>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn config(&self) -> &ProviderConfig;

    /// Blocking send -- ceka na celou odpoved
    fn send_chat(
        &self,
        messages: &[AiMessage],
        config: &ProviderConfig,
    ) -> Result<AiMessage, String>;

    /// Streaming send -- vraci Receiver s tokeny
    fn stream_chat(
        &self,
        messages: Vec<AiMessage>,
        config: ProviderConfig,
    ) -> std::sync::mpsc::Receiver<StreamEvent>;
}
```

### Pattern 2: Polling Detection (existujici codebase pattern)
**What:** Periodicky background check kazdy interval pres mpsc
**When to use:** Detekce Ollama serveru

```rust
// Pattern z existujiciho spawn_ai_tool_check:
pub fn spawn_ollama_check(base_url: &str) -> mpsc::Receiver<OllamaStatus> {
    let (tx, rx) = mpsc::channel();
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    std::thread::spawn(move || {
        match ureq::get(&url).call() {
            Ok(resp) => {
                let body: serde_json::Value = resp.into_json().unwrap_or_default();
                let models: Vec<String> = body["models"]
                    .as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                        .collect())
                    .unwrap_or_default();
                let _ = tx.send(OllamaStatus::Available(models));
            }
            Err(_) => {
                let _ = tx.send(OllamaStatus::Unavailable);
            }
        }
    });
    rx
}
```

### Pattern 3: NDJSON Streaming
**What:** Cteni Ollama /api/chat streaming response radek po radku
**When to use:** stream_chat() implementace

```rust
// Kazdy radek z Ollama /api/chat (stream:true) je samostatny JSON objekt:
// {"model":"llama3.2","message":{"role":"assistant","content":"tok"},"done":false}
// Posledni radek: {"model":"...","message":{"role":"assistant","content":""},"done":true,"prompt_eval_count":26,"eval_count":290}

fn stream_chat_impl(
    url: &str,
    body: String,
    tx: mpsc::Sender<StreamEvent>,
) {
    let resp = match ureq::post(url)
        .set("Content-Type", "application/json")
        .send_string(&body)
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(StreamEvent::Error(format!("HTTP error: {}", e)));
            return;
        }
    };

    let reader = std::io::BufReader::new(resp.into_reader());
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                let _ = tx.send(StreamEvent::Error(format!("Read error: {}", e)));
                return;
            }
        };
        if line.trim().is_empty() { continue; }

        let chunk: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(StreamEvent::Error(format!("JSON parse error: {}", e)));
                return;
            }
        };

        let done = chunk["done"].as_bool().unwrap_or(false);
        let content = chunk["message"]["content"].as_str().unwrap_or("");

        if !content.is_empty() {
            if tx.send(StreamEvent::Token(content.to_string())).is_err() {
                return; // receiver dropped -- cancelled
            }
        }

        if done {
            let _ = tx.send(StreamEvent::Done {
                model: chunk["model"].as_str().unwrap_or("").to_string(),
                prompt_tokens: chunk["prompt_eval_count"].as_u64().unwrap_or(0),
                completion_tokens: chunk["eval_count"].as_u64().unwrap_or(0),
            });
            return;
        }
    }
}
```

### Anti-Patterns to Avoid
- **Tokio runtime pro HTTP:** Codebase pouziva std::thread, nemixit async/blocking
- **Response buffering celeho streamu:** Pouzit BufReader::lines(), ne into_string()
- **Blocking UI thread:** Vsechny HTTP requesty MUSI bezet na std::thread, vysledky pres mpsc
- **Polling na UI threadu:** Timer check v update() ale request na background threadu

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client | Custom TCP/TLS handling | ureq 2.x | TLS, redirects, timeouts, encoding |
| JSON streaming | Custom byte parser | BufReader::lines() + serde_json | NDJSON je line-delimited, lines() staci |
| Thread communication | Custom shared state | mpsc::channel | Overeny pattern v codebase, zero dependencies |
| URL construction | String concatenation | format!() s trim_end_matches('/') | Konzistentni, bezpecne |

**Key insight:** Ollama NDJSON format je trivialni pro parsing -- kazdy radek je kompletni JSON objekt. BufReader::lines() je idealni, neni potreba custom NDJSON parser.

## Common Pitfalls

### Pitfall 1: Blocking UI thread
**What goes wrong:** HTTP request nebo streaming cteni na hlavnim egui threadu zamrzne editor
**Why it happens:** Zapomenuti obalit do std::thread::spawn
**How to avoid:** stream_chat() VZDY spawne novy thread, vraci jen mpsc::Receiver
**Warning signs:** Editor nereaguje behem generovani odpovedi

### Pitfall 2: Ureq timeout
**What goes wrong:** Streaming request timeouty po 30s (vychozi ureq timeout)
**Why it happens:** Ureq ma vychozi read timeout, Ollama muze generovat pomalu
**How to avoid:** Nastavit dostatecny timeout: `ureq::AgentBuilder::new().timeout_read(Duration::from_secs(300)).build()`
**Warning signs:** Streamy se prerusuji u delsi odpovedi

### Pitfall 3: Receiver dropped = silent cancel
**What goes wrong:** Kdyz UI dropne Receiver (uzivatel canceluje), sender dostane SendError
**Why it happens:** mpsc semantika -- send() selze kdyz receiver je dropnut
**How to avoid:** Kontrolovat vysledek tx.send() -- pokud Err, ukoncit thread ciste
**Warning signs:** Background thready bez ukonceni

### Pitfall 4: Model name parsing
**What goes wrong:** Ollama vraci "llama3.2:latest" ale uzivatel ocekava "llama3.2"
**Why it happens:** /api/tags vraci plne jmeno s tagem
**How to avoid:** Zobrazit v ComboBoxu jen cast pred ":" nebo cele jmeno -- CONTEXT rika "jen nazev modelu"
**Warning signs:** Duplicitni modely v seznamu (s a bez :latest)

### Pitfall 5: Paralelni polling a streaming
**What goes wrong:** Polling GET /api/tags behem aktivniho streamu zpusobi zbytecnou zatez
**Why it happens:** Timer bezici nezavisle
**How to avoid:** Preskocit polling kdyz probiha aktivni stream
**Warning signs:** Zbytecne sitove requesty

## Code Examples

### Ollama /api/tags Response Format
```json
// GET http://localhost:11434/api/tags
{
  "models": [
    {
      "name": "llama3.2:latest",
      "model": "llama3.2:latest",
      "modified_at": "2024-10-01T12:00:00Z",
      "size": 2019393189,
      "details": {
        "family": "llama",
        "parameter_size": "3.2B",
        "quantization_level": "Q4_K_M"
      }
    }
  ]
}
```

### Ollama /api/chat Streaming Request/Response
```json
// POST http://localhost:11434/api/chat
// Request:
{
  "model": "llama3.2",
  "messages": [
    {"role": "system", "content": "You are helpful."},
    {"role": "user", "content": "Hello"}
  ],
  "stream": true
}

// Response (NDJSON -- kazdy radek je JSON):
// {"model":"llama3.2","message":{"role":"assistant","content":"Hello"},"done":false}
// {"model":"llama3.2","message":{"role":"assistant","content":"!"},"done":false}
// {"model":"llama3.2","message":{"role":"assistant","content":""},"done":true,"prompt_eval_count":26,"eval_count":5}
```

### Existujici mpsc Pattern v Codebase
```rust
// Source: src/app/build_runner.rs
pub(crate) fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // ... blocking work ...
        let _ = tx.send(results);
    });
    rx
}

// Consumption in UI update loop (non-blocking try_recv):
if let Some(rx) = &ws.build_error_rx {
    if let Ok(errors) = rx.try_recv() {
        ws.build_errors = errors;
        ws.build_error_rx = None; // one-shot pattern
    }
}
```

### WorkspaceState Integration Points
```rust
// Existujici pole pro AI v WorkspaceState (src/app/ui/workspace/state/mod.rs):
pub ai_selected_provider: String,          // "ollama" / "gemini" -- rozsirit
pub ai_tool_available: HashMap<String, bool>,  // nahradit/doplnit o provider status
pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,  // nahradit
pub ai_tool_last_check: std::time::Instant,    // pouzit pro polling interval
pub ai_response: Option<String>,               // Phase 14 konsoliduje
pub ai_loading: bool,                          // pouzit pro streaming stav
pub ai_cancellation_token: Arc<AtomicBool>,    // pouzit pro cancel streaming
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WASM plugin (extism) pro Ollama | Nativni provider pres ureq | v1.2.0 | Odstranuje WASM overhead, umoznuje streaming |
| stream:false (blocking) | stream:true (NDJSON) | v1.2.0 | Token-po-tokenu rendering, lepsi UX |
| Jediny provider (Ollama WASM) | Trait abstrakce pro N provideru | v1.2.0 | Priprava pro Claude/Gemini/OpenAI |

**Deprecated/outdated:**
- WASM Ollama plugin (src/plugins/ollama/): Koexistuje az do Phase 17, pak odstraneni
- extism crate: Bude odebrano v Phase 17 (CLEN-02)

## Open Questions

1. **ureq major version: 2.x vs 3.x?**
   - What we know: Lockfile obsahuje ureq 2.12.1 (transitivni) i ureq 3.2.0 (transitivni). API se lisi -- 2.x ma `into_reader()`, 3.x ma `body_mut().as_reader()`.
   - What's unclear: Ktera verze je vhodnejsi jako prima zavislost
   - Recommendation: Pouzit ureq 2.x -- stabilni, overene API, `into_reader()` + `BufReader::lines()` pattern. Verze 3.x ma breakujici zmeny v API.

2. **Jak integrovat ProviderConfig se Settings?**
   - What we know: Settings jiz ma `plugins: HashMap<String, PluginSettings>` a `custom_agents`. Ollama plugin pouziva `config::get("API_URL")`.
   - What's unclear: Jestli pouzit existujici PluginSettings nebo novy mechanismus
   - Recommendation: Precist URL z existujiciho `settings.plugins["ollama"].config["API_URL"]` pokud existuje, jinak default localhost:11434. Model ukladat do eframe persistence.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in #[test] + cargo test |
| Config file | Cargo.toml (defaultni) |
| Quick run command | `cargo test --lib ai::` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROV-01 | AiProvider trait definice a metody | unit | `cargo test --lib ai::provider` | No -- Wave 0 |
| PROV-02 | OllamaProvider NDJSON parsing | unit | `cargo test --lib ai::ollama` | No -- Wave 0 |
| PROV-03 | /api/tags response parsing | unit | `cargo test --lib ai::ollama::tests` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib ai::`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/app/ai/provider.rs` -- AiProvider trait + StreamEvent enum + unit testy pro StreamEvent
- [ ] `src/app/ai/ollama.rs` -- OllamaProvider + testy pro NDJSON parsing (mock data, ne live server)
- [ ] Testy pro /api/tags response parsing (deserializace JSON -> Vec<String>)

*(Pozn.: Live integration testy s Ollama serverem nelze automatizovat v CI -- pouzit unit testy s mock JSON daty)*

## Sources

### Primary (HIGH confidence)
- Ollama API docs (GitHub raw) -- /api/tags format, /api/chat streaming NDJSON format
- ureq 2.12.1 docs.rs -- Response::into_reader(), BufReader pattern
- Existujici codebase (src/app/ai/, src/plugins/ollama/, src/app/build_runner.rs) -- mpsc patterns, typy

### Secondary (MEDIUM confidence)
- ureq GitHub repo -- streaming response handling patterns

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- ureq jiz v lockfile, mpsc overeny pattern v codebase
- Architecture: HIGH -- trait design vychazi z CONTEXT.md rozhodnuti, NDJSON format overeny z Ollama docs
- Pitfalls: HIGH -- vychazi z realnych zkusenosti s ureq timeouty a mpsc semantikou

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (stabilni domeny -- Ollama API, ureq 2.x)
