# Phase 13: Provider Foundation - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

AI provider abstrakce funguje a komunikuje s Ollama serverem. AiProvider trait s metodami pro chat, streaming, dostupnost a modely. OllamaProvider implementace s NDJSON streaming přes ureq + std::thread. Auto-detect běžícího Ollama serveru a zobrazení dostupných modelů. UI a konverzační logika patří do Phase 15, tool calling do Phase 16.

</domain>

<decisions>
## Implementation Decisions

### Detekce serveru
- Periodický polling každých 10s na pozadí (GET /api/tags)
- Při prvním spuštění okamžitý check
- Status ikona v AI baru (zelená/červená/šedá) — vždy viditelná když je panel otevřený
- URL Ollama serveru konfigurovatelná přes existující nastavení agentů v Settings (výchozí localhost:11434, ale podporuje remote)

### Model list
- Refresh seznamu modelů spolu s detekcí — jeden request GET /api/tags vrátí obojí
- Prázdný stav: ComboBox zobrazí "No models available" + tooltip s návodem (ollama pull), disabled stav
- Výchozí model: zapamatovat poslední zvolený model v settings, fallback na první v seznamu
- ComboBox zobrazuje jen název modelu (bez velikosti/rodiny)

### Chybové stavy
- Chyby prezentovány inline v chatu jako červený blok — uživatel vidí kontext (co poslal, co selhalo)
- Při přerušení streamu uprostřed odpovědi: zachovat částečnou odpověď + připojit chybový blok "Stream interrupted"
- Tlačítko "Retry" pod chybovým blokem — pošle stejný prompt znovu
- Žádný automatický retry — provider jen reportuje chybu, periodický polling detekce se postará o zjištění dostupnosti

### Trait rozsah
- Střední rozsah: name(), available_models(), send_chat(), stream_chat(), is_available(), config(), capabilities()
- capabilities() vrací struct (supports_streaming, supports_tools) — připraveno pro rozlišení Claude/Gemini/Ollama
- stream_chat() vrátí mpsc::Receiver<StreamEvent> — odpovídá existujícímu mpsc patternu (build_error_rx, git_status_rx)
- StreamEvent enum obsahuje varianty: Token, Done, Error, ToolCall (ToolCall připravena ale neimplementována do Phase 16)
- Tool calling varianta v traitu připravena od začátku, OllamaProvider ji implementuje až v Phase 16

### Umístění kódu
- Rozšířit existující src/app/ai/ modul — trait a OllamaProvider žijí vedle AiManager a typů

### Claude's Discretion
- Konkrétní StreamEvent enum design a pole
- Vnitřní buffering strategie pro NDJSON parsing
- Timeout hodnoty pro ureq requesty
- Přesný design capabilities() struct

</decisions>

<specifics>
## Specific Ideas

- URL Ollama serveru se čte z existujícího nastavení agentů (Settings plugin sekce) — uživatel může nastavit jak lokální tak cloudovou instanci
- Polling a model list v jednom requestu (GET /api/tags) — efektivní, minimální síťová zátěž
- mpsc pattern konzistentní s celým codebase (build_error_rx, git_status_rx, lsp_hover_rx)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AiManager` (src/app/ai/mod.rs): Existující context generation — nový provider jej využije
- `AiMessage`, `AiConversation`, `AiContextPayload` (src/app/ai/types.rs): Existující typy pro AI komunikaci
- `AiToolDeclaration` (src/app/ai/types.rs): Existující tool deklarace — využitelné pro tool calling v Phase 16
- ai_bar (src/app/ui/terminal/right/ai_bar.rs): Existující AI bar s ComboBox — rozšířit o status ikonu a nový model picker
- Toast pattern (AppAction::ShowToast): Pro případné sekundární notifikace

### Established Patterns
- mpsc::Receiver pro async výsledky: build_error_rx, git_status_rx, lsp_hover_rx — provider streaming bude stejný pattern
- Periodický background check: ai_tool_check_rx, git_last_refresh — polling detekce serveru bude stejný pattern
- ureq + std::thread pro HTTP: Rozhodnuto v PROJECT.md, odpovídá threading modelu codebase (ne tokio pro HTTP)
- eprintln! pro error logging: Existující pattern, žádný structured logging

### Integration Points
- WorkspaceState.ai_* pole: ~30 polí pro AI stav — Phase 14 je konsoliduje do AiChatState
- AI bar render (right/ai_bar.rs): Přidat status ikonu a napojit na nový provider
- WASM Ollama plugin (src/plugins/ollama/): Koexistuje s novým nativním providerem — odstranění v Phase 17
- Settings: Existující agent konfigurace pro URL Ollama serveru

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 13-provider-foundation*
*Context gathered: 2026-03-06*
