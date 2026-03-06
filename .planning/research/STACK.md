# Technology Stack: AI Chat Rewrite

**Project:** PolyCredo Editor v1.2.0
**Researched:** 2026-03-06
**Focus:** Stack additions for native AI Chat with Ollama provider, streaming, approval UI, editor context

## Current Stack (DO NOT change)

Already validated and shipping: `eframe/egui 0.31`, `syntect 5`, `egui_term 0.1`, `fluent-bundle 0.15`, `notify 7`, `rfd 0.15`, `pulldown-cmark 0.12`, `egui_commonmark 0.20`, `tokio 1` (rt-multi-thread), `serde/serde_json 1`, `anyhow 1`.

---

## Recommended Stack Additions

### HTTP Client: reqwest

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `reqwest` | `0.12` | HTTP client for Ollama API | Async-native, tokio-compatible (tokio uz v projektu), streaming pres `bytes_stream()`, de-facto standard v Rust ekosystemu. Projekt uz ma tokio runtime -- ureq (sync-only) by vyzadoval spawn_blocking a nema streaming support. |

**Features to enable:** `json`, `stream`, `rustls-tls`

**Confidence:** HIGH -- reqwest 0.12 je stabilni, aktivne udrzovany, tokio-kompatibilni. Projekt uz pouziva tokio s `rt-multi-thread`.

**Why NOT ureq:** Projekt uz ma tokio runtime. ureq je synchronni, nepodporuje streaming, vyzadoval by `spawn_blocking` wrapper. reqwest je prirozena volba kdyz tokio uz existuje.

**Why NOT hyper primo:** Prilis low-level. reqwest je wrapper nad hyper s ergonomickym API pro JSON, streaming, headers.

### Streaming: No extra crate needed

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| (reqwest built-in) | -- | NDJSON stream parsing | Ollama API vraci newline-delimited JSON (NDJSON). `response.bytes_stream()` + manualni line splitting staci. Neni treba `reqwest-streams` -- ten pridava overhead pro CSV/Protobuf ktere nepotrebujeme. |

**Streaming pattern pro Ollama:**

```rust
// reqwest bytes_stream() + rucni NDJSON parsing
let response = client.post(url).json(&request).send().await?;
let mut stream = response.bytes_stream();
let mut buffer = String::new();

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    buffer.push_str(&String::from_utf8_lossy(&chunk));

    // NDJSON: kazdy radek je kompletni JSON objekt
    while let Some(newline_pos) = buffer.find('\n') {
        let line = buffer[..newline_pos].trim().to_string();
        buffer = buffer[newline_pos + 1..].to_string();
        if line.is_empty() { continue; }

        let chunk: OllamaStreamChunk = serde_json::from_str(&line)?;
        // Poslat pres channel do UI threadu
        let _ = tx.send(StreamEvent::Token(chunk.message.content));
    }
}
```

**Confidence:** HIGH -- Ollama dokumentace potvrzuje NDJSON format. reqwest `bytes_stream()` je stabilni feature.

### Async-to-UI Bridge: tokio channels

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `tokio::sync::mpsc` | (uz v projektu) | Stream tokenu z async tasku do egui UI | Unbounded channel. Async task posila tokeny, UI thread je drainuje kazdy frame. Lepsi nez `std::sync::mpsc` protoze funguje v async i sync kontextu. |
| `egui::Context::request_repaint()` | (uz v projektu) | Probuzeni UI pri novem tokenu | Volat po kazdem `tx.send()` aby egui okamzite prekreslil. Bez toho by streaming vypadal trhane (egui defaultne prekresli jen pri interakci). |

**Pattern:**

```rust
// V async tasku:
let _ = tx.send(StreamEvent::Token(text));
ctx.request_repaint(); // Probudit egui

// V UI kodu (kazdy frame):
while let Ok(event) = rx.try_recv() {
    match event {
        StreamEvent::Token(t) => accumulated_text.push_str(&t),
        StreamEvent::Done => is_loading = false,
        StreamEvent::Error(e) => show_error(e),
    }
}
```

**Confidence:** HIGH -- Toto je standardni egui pattern pro async operace. Potvrzeno v egui diskuzich a prikladech (parasyte/egui-tokio-example).

### Chat UI Rendering: egui_commonmark (uz v projektu)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `egui_commonmark` | `0.20` (uz v Cargo.toml) | Markdown rendering v chat odpovedi | Uz v projektu s `better_syntax_highlighting` feature. Pouzit pro formatovani AI odpovedi -- code blocky, nadpisy, seznamy. |
| `egui::ScrollArea` | (soucasti egui) | Scrollovatelna historie chatu | `stick_to_bottom(true)` pro auto-scroll pri streamingu. Variable-height polozky (zpravy maji ruznou delku). |

**Neni treba:** Zadna nova UI knihovna. egui ma vse potrebne -- `ScrollArea`, `TextEdit`, `RichText`, `CollapsingHeader` pro approval UI.

**Confidence:** HIGH -- egui_commonmark uz v projektu funguje.

---

## Provider Abstraction: Pure Rust trait

Zadna nova knihovna. Ciste Rust trait + async_trait.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `async-trait` | `0.1` | Async metody v trait definicich | Rust edition 2024 ma RPITIT ale neni plne kompatibilni s `dyn` dispatch (Box<dyn AiProvider>). `async-trait` je de-facto standard pro dynamicky dispatch s async metodami. |

**Navrh trait hierarchie:**

```rust
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn default_model(&self) -> &str;

    async fn send_streaming(
        &self,
        request: ChatRequest,
        tx: tokio::sync::mpsc::UnboundedSender<StreamEvent>,
        cancel: Arc<AtomicBool>,
    ) -> Result<(), ProviderError>;

    fn available_models(&self) -> Vec<String> { vec![] }
    fn supports_tools(&self) -> bool { false }
    fn supports_reasoning(&self) -> bool { false }
}
```

**Why trait-based:** Soucasny kod pouziva WASM plugin s `extism` -- kazdy provider je separatni WASM modul. To pridava rezii (WASM runtime, serializace, omezeni na sync HTTP). Nativni trait umozni:
- Primo streaming (WASM neumi streamy)
- Primo pristup k host functions (zadna serializace)
- Snadne testovani (mock implementace)
- Rozsirovatelnost pro Claude/Gemini bez WASM buildu

**Confidence:** HIGH -- Standardni Rust pattern.

---

## Ollama API Integration Details

**Endpoint:** `POST /api/chat`
**Format:** NDJSON streaming (default), single JSON s `"stream": false`
**Port:** 11434 (default, konfigurovatelny pres `OLLAMA_HOST`)

**Klicove parametry:**
- `model`: nazev modelu (povinny)
- `messages`: pole `{role, content, tool_calls?, tool_call_id?}`
- `tools`: pole tool definic (type: "function")
- `stream`: bool (default true)
- `options`: `{temperature, num_ctx, ...}`

**Streaming response chunk:**
```json
{"model":"llama3.1","created_at":"...","message":{"role":"assistant","content":"token"},"done":false}
```

**Final chunk (done=true) obsahuje:** `prompt_eval_count`, `eval_count` (token usage)

**Tool calling:** Podporovan, vcetne streaming tool calls (od Ollama 0.5+). Response s tool_calls ma `message.tool_calls` pole.

**Confidence:** HIGH -- Potvrzeno z oficialni Ollama dokumentace.

---

## What NOT to Add

| Library | Why NOT |
|---------|---------|
| `ureq` | Sync-only, projekt uz ma tokio. Zadne streaming. |
| `reqwest-streams` | Overkill -- NDJSON parsing je trivialni (~10 radku kodu). |
| `eventsource-client` / `sse` crate | Ollama nepouziva SSE (Server-Sent Events). Pouziva NDJSON. |
| `ollama-rs` | Vysokourovnovy wrapper -- pridava abstrakci nad jednoduche API. Vlastni trait abstrakce je flexibilnejsi a umozni snadne pridani Claude/Gemini. |
| `egui-async` | Zbytecna abstrakce. tokio channels + request_repaint() staci. |
| `tui` / `ratatui` | Editor uz ma vlastni terminal rendering. CLI-style chat je egui widget, ne skutecny TUI. |
| Nova markdown knihovna | `egui_commonmark 0.20` uz v projektu, pokryva code blocky, seznamy, nadpisy. |

---

## What to REMOVE (eventually)

| Library | Why Remove | When |
|---------|-----------|------|
| `extism 1.5` | WASM plugin runtime -- nahrazeno nativnimi providery | Po migraci vsech provideru |
| `candle-core/nn/transformers 0.9` | ML inference -- pouzivano pro semantic embeddingy | Overit zda se embeddingy presmeruji na Ollama API (POST /api/embed) |
| `hf-hub 0.4` | HuggingFace model download -- spojeno s candle | Overit ve fazi planovani |
| `tokenizers 0.21` | Tokenizace pro candle | Overit ve fazi planovani |

---

## Installation

```toml
# Novy v Cargo.toml [dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"] }
async-trait = "0.1"

# POZNAMKA: futures-util je potreba pro StreamExt (bytes_stream iteration)
futures-util = "0.3"

# Uz existuje, bez zmeny:
# tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "io-util"] }
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# anyhow = "1"
```

**TLS poznamka:** Pouzit `rustls-tls` misto `default-tls` (nativni openssl). Duvody:
1. Projekt uz bezi na Linuxu kde openssl muze byt problem pri cross-compilation
2. rustls je pure Rust, zadna C dependency
3. Pro lokalni Ollama (localhost) TLS neni potreba, ale pro budouci Claude/Gemini API ano

**Tokio features:** Projekt uz ma `rt-multi-thread`, `macros`, `process`, `io-util`. Moze byt potreba pridat `sync` feature pro `mpsc` channels -- overit pri implementaci.

---

## Integration Points

### 1. Async Runtime
Projekt uz ma `tokio` s `rt-multi-thread`. reqwest ho pouzije automaticky. Zadny novy runtime.

### 2. Stavajici AI Chat kod
Soucasny flow: `logic.rs` spousti `std::thread::spawn` -> vola WASM plugin pres `extism` -> vraci vysledek pres `AppAction::PluginResponse`.

**Novy flow:**
1. `logic.rs` spawne tokio task (misto `std::thread::spawn`)
2. Tokio task vola `provider.send_streaming()` s `mpsc::UnboundedSender`
3. UI thread kazdy frame drainuje `mpsc::UnboundedReceiver`
4. Streaming tokeny se akumuluji v `WorkspaceState.ai_streaming_buffer`

### 3. Approval UI
Tool calls (read_file, write_file, exec) prichazi jako `StreamEvent::ToolCall`. UI zobrazi approval dialog. Uzivatel schvali/odmitne. Vysledek se posle zpet do async tasku pres `oneshot` channel.

```rust
enum StreamEvent {
    Token(String),
    Reasoning(String),
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
        response_tx: oneshot::Sender<ToolResult>,
    },
    Usage { prompt_tokens: u64, completion_tokens: u64 },
    Done,
    Error(String),
}
```

### 4. egui Rendering
- `ScrollArea::vertical().stick_to_bottom(true)` pro chat historii
- `egui_commonmark` pro formatovane odpovedi (code blocky, markdown)
- `RichText` s monospace font pro CLI-style prompt
- `CollapsingHeader` pro reasoning/monologue sekce

### 5. Soucasny WASM plugin kod (co se zachova)
Datove struktury v `src/plugins/ollama/src/lib.rs` (OllamaRequest, Message, Tool, ToolCall, AiContextPayload) jsou dobre navrzene a mohou se primo prevzit do nativniho provideru. Tool handling logika (read_project_file, write_file, exec, atd.) se presune z WASM host functions na primo Rust volani.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| HTTP Client | reqwest 0.12 | ureq 3.x | Sync-only, no streaming, tokio uz v projektu |
| HTTP Client | reqwest 0.12 | hyper primo | Prilis low-level, reqwest je ergonomicky wrapper |
| Ollama SDK | Vlastni trait | ollama-rs | Prilis specificke, neumozni snadne pridani jinych provideru |
| Async trait | async-trait 0.1 | RPITIT (nativni) | Neni plne kompatibilni s dyn dispatch; async-trait je bezpecnejsi volba |
| Streaming bridge | tokio mpsc | crossbeam | Neni async-aware, tokio mpsc je prirozena volba s tokio runtime |
| Chat markdown | egui_commonmark 0.20 | Vlastni parser | Uz v projektu, funguje, zbytecna duplikace |

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| reqwest jako HTTP client | HIGH | De-facto standard, tokio-kompatibilni, overeno v ekosystemu |
| NDJSON streaming pattern | HIGH | Ollama dokumentace potvrzuje format, reqwest bytes_stream() je stabilni |
| async-trait pro provider abstrakci | HIGH | Standard Rust pattern, siroko pouzivany |
| tokio channels pro UI bridge | HIGH | Potvrzeno v egui community (parasyte/egui-tokio-example, emilk/egui#2462) |
| egui_commonmark pro chat rendering | HIGH | Uz v projektu, funguje |
| Odstraneni extism/candle | MEDIUM | Overit zda candle ma jine pouziti krome AI chatu |

## Sources

- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Ollama Streaming Tool Calls](https://ollama.com/blog/streaming-tool)
- [reqwest crate](https://crates.io/crates/reqwest)
- [egui-tokio-example](https://github.com/parasyte/egui-tokio-example)
- [egui Discussion #2462: Reqwest + Egui](https://github.com/emilk/egui/discussions/2462)
- [reqwest Docs.rs - Response streaming](https://docs.rs/reqwest/latest/reqwest/struct.Response.html)
- [LogRocket: How to choose the right Rust HTTP client](https://blog.logrocket.com/best-rust-http-client/)
- [Ollama Streaming docs](https://docs.ollama.com/api/streaming)
