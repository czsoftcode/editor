# Phase 2: Background Isolation - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Zajistit plynulost UI i při vysoké aktivitě na pozadí. Zaměřit se na:
- Terminál (PTY throughput vs UI latency)
- AI chat (izolace komunikace s agenty)
- Centrální throttling asynchronních událostí

Tato fáze se nezabývá vizuální stránkou (layouty), ale datovými toky a vlákny.

</domain>

<decisions>
## Implementation Decisions

### Terminal Isolation
- PTY události (`PtyWrite`) parsovat v dávkách s limitem času (nejen počtu eventů) — Claude rozhoduje o limitu (např. max 2ms per frame na parsování).
- Prozkoumat možnost přesunu `TerminalBackend` do vlákna (pokud `egui_term` podporuje Send/Sync).

### AI Chat Isolation
- Komunikace s agenty je již v `std::thread`, ale synchronizace stavu přes `AppAction` může být vylepšena (batching).

### LSP Throttling
- Stávající throttling diagnostik (500ms) je v pořádku.

</decisions>

<code_context>
## Existing Code Insights

### Terminal
- `src/app/ui/terminal/instance/mod.rs:124` — smyčka pro čtení z PTY.
- `config::TERMINAL_MAX_EVENTS_PER_FRAME` — aktuální limit je 64 událostí.

### LSP
- `src/app/lsp/mod.rs:83` — throttling diagnostik.

</code_context>

<specifics>
## Specific Ideas

- Success criterion 1: `cat large_file.txt` v terminálu nezpůsobí pokles FPS pod 30.
- Success criterion 2: AI agent pracující na pozadí (přes plugin host) nezpůsobuje "micro-stutters" v editoru.

</specifics>
