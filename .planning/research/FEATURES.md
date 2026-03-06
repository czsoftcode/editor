# Feature Landscape

**Domain:** AI Chat integration pro nativni kod editor (Rust/egui)
**Researched:** 2026-03-06
**Milestone:** v1.2.0 AI Chat Rewrite

## Table Stakes

Features, ktere uzivatele ocekavaji od AI chatu v editoru. Jejich absence = produkt pusobi nedokonale.

| Feature | Why Expected | Complexity | Depends On | Notes |
|---------|--------------|------------|------------|-------|
| Streaming odpovedi | Uzivatele ocekavaji real-time tok textu, ne cekani na celou odpoved | Med | Ollama provider, async runtime | Ollama `/api/chat` vraci NDJSON stream; uz existuje `tokio` v dependencies |
| Kontext otevreneho souboru | Kazdy AI editor posila aktivni soubor jako kontext | Low | Existing `AiContextPayload` | **Uz implementovano** v `AiManager::generate_context()` — staci pouzit |
| Git status kontext | AI musi vedet, co je zmeneno, na jake vetvi pracuje | Low | Existing git integration | **Uz implementovano** — `AiContextPayload.git_branch`, `git_status` |
| Build errors kontext | AI musi vedet o chybach, aby mohl navrhnout opravy | Low | Existing build runner | **Uz implementovano** — `AiContextPayload.build_errors` |
| Markdown rendering odpovedi | Kod bloky, nadpisy, bold/italic v odpovedich | Low | Existing `egui_commonmark` + `pulldown-cmark` | **Uz implementovano** — `AiChatWidget::ui_conversation()`, `markdown_cache` |
| Conversational historie | Multi-turn konverzace, ne single-shot Q&A | Med | Provider trait, message store | Castecne existuje (`ai_conversation`), ale potrebuje refaktor na `AiConversation` struct |
| File read/write tools | AI musi umet cist a zapisovat soubory | Med | Tool execution engine, approval UI | **Uz implementovano** jako WASM tool — portovat na nativni |
| Command execution tool | AI musi umet spoustet prikazy (cargo check, tests) | Med | Tool execution engine, approval UI | **Uz implementovano** jako WASM — portovat na nativni |
| Approval UI pro nebezpecne akce | Uzivatel musi schvalit zapisy a exec pred provedenim | Med | Existing `PendingPluginApproval` pattern | **Castecne existuje** — `render_approval_ui`, `PluginApprovalResponse` |
| Cancel/Stop tlacitko | Moznost prerusit generovani | Low | `ai_cancellation_token` | **Uz implementovano** — `AtomicBool` + Escape key |
| Dark/Light mode podpora | Chat musi respektovat zvolene tema editoru | Med | Existing theme system | Soucasny chat pouziva hardcoded dark barvy (`Color32::from_rgb(20, 20, 25)`); refaktor na `ui.visuals()` |
| Provider selector | Vyber AI providera (Ollama model) | Low | Provider trait | Castecne existuje (combo box pro agenty), refaktor na nativni providery |
| Input s historii | Sipkami nahoru/dolu prochazel predchozi prompty | Low | Existing `ai_history` | **Uz implementovano** — `AiChatWidget::ui_input()` |

## Differentiators

Features, ktere odlisuji PolyCredo od bezneho AI chatu. Ne ocekavane, ale cenene.

| Feature | Value Proposition | Complexity | Depends On | Notes |
|---------|-------------------|------------|------------|-------|
| Expertise Role system | Junior/Senior/Master meni chovani AI (opatrnost, hloubka) | Low | Existing `AiExpertiseRole` | **Uz implementovano** — portovat system mandatu do nativniho provideru |
| Reasoning Depth | Fast/Balanced/Deep ovlivnuje kolik souboru AI precte | Low | Existing `AiReasoningDepth` | **Uz implementovano** — presunout do system promptu |
| Semantic search tool | AI hleda v projektu pomoci BERT embeddingu, ne jen grep | Low | Existing `SemanticIndex`, candle | **Uz implementovano** — portovat z WASM na nativni tool handler |
| Replace tool (surgical edits) | AI edituje presne bloky kodu misto prepisovani celych souboru | Low | Existing tool definition | **Uz implementovano** v WASM — portovat na nativni |
| Agent memory (store/retrieve_fact) | AI si pamatuje uzivatelske preference mezi sezenimi | Low | Existing `AgentMemory` | **Uz implementovano** — portovat |
| Inspector panel | Debug view: surovy JSON payload, monologue steps, token usage | Low | Existing `render_inspector` | **Uz implementovano** — zachovat |
| Ask-user tool | AI se muze zeptat uzivatele na upresneni misto hadani | Low | Existing `PendingAskUser` | **Uz implementovano** — zachovat |
| Announce completion | AI signalizuje dokonceni ukolu se summary a seznamem zmenenych souboru | Low | Existing `PluginCompleted` | **Uz implementovano** — zachovat |
| Auto-approve ("Always allow") | Uzivatel muze permanentne schvalit typ akce | Low | Existing `ApproveAlways` variant | **Uz implementovano** — zachovat |
| Floating / Viewport / Docked modes | AI chat jako docked panel, floating okno, nebo samostatny viewport | Low | Existing `StandardTerminalWindow` | **Uz implementovano** — zachovat |
| Scratchpad (session memory) | AI si v ramci ukolu poznamenava prubezne vystupy | Low | Existing scratch tools | **Uz implementovano** — portovat |
| Multi-provider abstrakce | Trait pro Ollama dnes, Claude/Gemini/OpenAI zitra | High | Provider trait design | NOVE — klicovy architekturni prvek milniku |
| Ollama auto-detection | Automaticka detekce bezicho Ollama serveru na localhost:11434 | Low | HTTP health check | NOVE — jednoduchy GET `/api/tags` |
| Model picker | Seznam dostupnych modelu z Ollama API | Med | Ollama provider | NOVE — GET `/api/tags` vraci seznam |

## Anti-Features

Features, ktere NECHCEME stavet. Explicitne vynechane.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| OpenAI-compatible endpoint | Ollama ma vlastni nativni API, OpenAI compat ma problemy se streaming tool calling (issue #12557) | Pouzit nativni Ollama `/api/chat` endpoint |
| Vlastni WASM runtime pro AI | Odstranujem WASM prave kvuli slozitosti a overhead | Nativni Rust trait + HTTP klient |
| Auto-execute bez approval | Bezpecnostni riziko — AI nesmi volne mazat/zapisovat | Zachovat existujici approval workflow |
| RAG / vector DB integrace | Over-engineering, existujici semantic search staci | Ponechat stavajici BERT-based SemanticIndex |
| Inline code suggestions (ghost text) | Vyzaduje LSP-level integraci, jiny milestone | Ponechat chat-based interakci |
| Voice input/output | Mimo scope editoru, zbytecna komplexita | Textovy vstup |
| Image/multimodal input | Ollama to castecne podporuje, ale UX v egui je slozite | Text-only prozatim |
| Automaticke code apply | Windsurf-style "apply pred schvalenim" je rizikove | Uzivatel schvaluje kazdou zmenu explicitne (Cursor-style) |
| Plugin marketplace | Over-engineering pro v1.2 | Fixni sada nativnich toolu |

## Feature Dependencies

```
Provider Trait Abstrakce
  --> Ollama Provider (implements trait)
    --> Streaming Engine (NDJSON parser)
      --> Chat UI s real-time aktualizaci
        --> Markdown rendering odpovedi (uz existuje)

Provider Trait Abstrakce
  --> Tool Declaration System (uz existuje, portovat)
    --> Nativni Tool Handler (nahrazuje WASM host)
      --> File Read/Write/Replace Tools
      --> Command Execution Tool
      --> Search Tools (grep + semantic)
      --> Memory Tools (scratch + facts)
        --> Approval UI (uz existuje, zachovat)
        --> Ask-User UI (uz existuje, zachovat)

Kontext Engine (uz existuje)
  --> System Prompt Builder (uz existuje, portovat)
    --> Provider Trait (pouziva kontext + system prompt)

WASM Plugin System
  --> Postupne odstraneni az po dokonceni nativnich nahrad
```

## MVP Recommendation

### Prioritize (Phase 1 — zakladni nativni chat):

1. **Provider trait abstrakce** — `AiProvider` trait s `send_chat()` a `stream_chat()` metodami
2. **Ollama provider** — implementace traitu pro Ollama `/api/chat` s NDJSON streaming
3. **Nativni tool handler** — portovat tool execution z WASM host na nativni Rust funkce
4. **Chat UI napojeni** — prepojit existujici UI na nativni provider misto WASM plugin manageru

### Prioritize (Phase 2 — doladeni):

5. **Dark/Light mode fix** — nahradit hardcoded barvy za `ui.visuals()` queries
6. **Model picker** — GET `/api/tags` pro vyber modelu
7. **Ollama auto-detection** — health check na startu
8. **i18n** — nove klice pro provider-related UI

### Defer:

- **Claude/Gemini/OpenAI providery** — az po Ollama funguje stabilne (v1.3+)
- **WASM system removal** — az po overeni, ze nativni chat plne funguje; v1.2.0 muze mit oba systemy paralelne
- **Multimodal input** — v1.4+ pokud vubec

## Complexity Assessment

| Feature Group | New Code | Reuse | Overall Complexity |
|---------------|----------|-------|-------------------|
| Provider trait + Ollama | ~400 LOC | 0 | Med — NDJSON streaming parser je netrivialni |
| Nativni tool handler | ~300 LOC | ~500 LOC z WASM host | Med — portovani logiky, ne psani od nuly |
| Chat UI prepojeni | ~100 LOC | ~800 LOC existujici UI | Low — meni se jen `logic.rs` a data flow |
| Dark/Light mode fix | ~50 LOC | cely theme system | Low — nahrada literal barev |
| Model picker + auto-detect | ~150 LOC | 0 | Low — jednoduche HTTP requesty |
| WASM removal | -2000+ LOC | n/a | Med — opatrne mazani s testovanim |

**Total noveho kodu:** ~1000 LOC
**Znovupouzity kod:** ~1300 LOC (existujici UI, tools, kontext, approval)
**Smazany kod:** ~2000+ LOC (WASM runtime, plugin host)
**Net effect:** Mensi codebase s jednodussim kodem

## Sources

- [Ollama API docs](https://github.com/ollama/ollama/blob/main/docs/api.md) — MEDIUM confidence
- [Ollama streaming + tool calling blog](https://ollama.com/blog/streaming-tool) — HIGH confidence (official)
- [ollama-rs crate](https://github.com/pepperoni21/ollama-rs) — MEDIUM confidence (considered but NOT recommended — raw reqwest is simpler for this use case)
- [genai crate](https://github.com/jeremychone/rust-genai) — MEDIUM confidence (multi-provider abstraction reference)
- [egui_commonmark](https://github.com/lampsitter/egui_commonmark) — HIGH confidence (already in use)
- [Cursor vs Windsurf approval patterns](https://www.builder.io/blog/windsurf-vs-cursor) — MEDIUM confidence
- Existujici zdrojovy kod projektu: `src/app/ai/`, `src/app/ui/terminal/ai_chat/`, `src/app/registry/plugins/` — HIGH confidence
