# Requirements: PolyCredo Editor v1.2.0

**Defined:** 2026-03-06
**Core Value:** Editor nesmi zahrivat notebook v klidovem stavu — idle CPU zatez musi byt minimalni.

## v1.2.0 Requirements

Requirements pro AI Chat Rewrite. Kazdy mapuje na roadmap faze.

### Provider

- [ ] **PROV-01**: AiProvider trait s metodami send_chat(), stream_chat(), name(), available_models()
- [ ] **PROV-02**: OllamaProvider implementuje AiProvider s NDJSON streaming pres ureq + std::thread
- [x] **PROV-03**: Auto-detect Ollama serveru na localhost:11434 (GET /api/tags)
- [ ] **PROV-04**: Model picker — ComboBox s dostupnymi modely z Ollama API

### Chat UI

- [ ] **CHAT-01**: Hybrid CLI layout — prompt dole jako CLI, odpovedi nahore s vizualnim oddelenim
- [ ] **CHAT-02**: Streaming rendering — prubezne zobrazovani odpovedi token po tokenu
- [ ] **CHAT-03**: Dark/light mode — theme-aware barvy z ui.visuals() misto hardcoded
- [ ] **CHAT-04**: Markdown v odpovedich — code blocks, inline code, bold/italic
- [ ] **CHAT-05**: Konverzacni historie — multi-turn chat s persistenci v session
- [ ] **CHAT-06**: Input s historii promptu (sipky nahoru/dolu)
- [ ] **CHAT-07**: Cancel/Stop tlacitko pro preruseni generovani

### Context & Tools

- [ ] **TOOL-01**: Automaticky editor kontext — otevrene soubory, git stav, build errory
- [ ] **TOOL-02**: File read tool — AI cte soubory s approval
- [ ] **TOOL-03**: File write/replace tool — AI upravuje soubory s approval a diff preview
- [ ] **TOOL-04**: Command execution tool — AI spousti prikazy s approval
- [ ] **TOOL-05**: Approval UI — Approve/Deny/Always workflow pro tool volani
- [ ] **TOOL-06**: Ask-user tool — AI se muze zeptat uzivatele na upresneni

### Cleanup

- [ ] **CLEN-01**: AiChatState sub-struct — konsolidace ~30 ai_* poli z WorkspaceState
- [ ] **CLEN-02**: Odstraneni WASM plugin systemu — extism, PluginManager, ~2000 LOC
- [ ] **CLEN-03**: i18n aktualizace — nove klice pro novy chat, odstraneni starych WASM klicu

## Future Requirements

### Dalsi providery

- **PROV-05**: ClaudeProvider — Anthropic API s streaming
- **PROV-06**: GeminiProvider — Google AI API s streaming
- **PROV-07**: OpenAI-compatible provider — pro custom endpointy

### Pokrocile funkce

- **CHAT-08**: Multimodal vstup (obrazky)
- **TOOL-07**: Inline code suggestions (ghost text)

## Out of Scope

| Feature | Reason |
|---------|--------|
| OpenAI-compatible endpoint pro Ollama | Nativni Ollama API je spolehlive pro streaming tool calling (issue #12557) |
| WASM runtime pro AI | Odstranujeme prave kvuli slozitosti a overhead |
| Auto-execute bez approval | Bezpecnostni riziko — AI nesmi volne mazat/zapisovat |
| RAG / vector DB integrace | Over-engineering, existujici semantic search staci |
| Voice input/output | Mimo scope editoru |
| Plugin marketplace | Over-engineering pro v1.2 |
| Automaticke code apply | Windsurf-style "apply pred schvalenim" je rizikove |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROV-01 | Phase 13 | Pending |
| PROV-02 | Phase 13 | Pending |
| PROV-03 | Phase 13 | Complete |
| PROV-04 | Phase 15 | Pending |
| CHAT-01 | Phase 15 | Pending |
| CHAT-02 | Phase 15 | Pending |
| CHAT-03 | Phase 15 | Pending |
| CHAT-04 | Phase 15 | Pending |
| CHAT-05 | Phase 15 | Pending |
| CHAT-06 | Phase 15 | Pending |
| CHAT-07 | Phase 15 | Pending |
| TOOL-01 | Phase 16 | Pending |
| TOOL-02 | Phase 16 | Pending |
| TOOL-03 | Phase 16 | Pending |
| TOOL-04 | Phase 16 | Pending |
| TOOL-05 | Phase 16 | Pending |
| TOOL-06 | Phase 16 | Pending |
| CLEN-01 | Phase 14 | Pending |
| CLEN-02 | Phase 17 | Pending |
| CLEN-03 | Phase 17 | Pending |

**Coverage:**
- v1.2.0 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0

---
*Requirements defined: 2026-03-06*
*Last updated: 2026-03-06 after roadmap creation*
