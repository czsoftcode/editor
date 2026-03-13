# Phase 14: State Refactor - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Konsolidace ~25 ai_* a ollama_* poli z WorkspaceState do AiState sub-structu (ws.ai.*). Cileho stavu je cista separace AI stavu od workspace stavu, priprava codebase pro napojeni provideru na UI v Phase 15. Zadna nova funkcionalita — ciste refaktorovaci faze.

</domain>

<decisions>
## Implementation Decisions

### Grouping strategie
- Presunout do AiState: vsechna ai_* pole (prompt, history, conversation, system_prompt, expertise, reasoning_depth, tokens, inspector, loading, response, focus_requested, last_payload, font_scale, selected_provider, show_settings, cancellation_token) + vsechna ollama_* pole (status, models, selected_model, check_rx, last_check, base_url, api_key) + markdown_cache
- Zustavaji ve WorkspaceState: claude_tabs, claude_active_tab, next_claude_tab_id, claude_float (terminalove pole), pending_ask_user, pending_plugin_approval (plugin/WASM pole), ai_tool_available, ai_tool_check_rx, ai_tool_last_check (detekce externich CLI nastroju), selected_agent_id, show_ai_chat, ai_viewport_open

### Pojmenovani a pristup
- Pole ve WorkspaceState: `pub ai: AiState` — pristup pres ws.ai.*
- Struct nazev: `AiState`
- Pri presunu zkratit nazvy odstranenim prefixu ai_: ai_prompt -> prompt, ai_loading -> loading, ai_conversation -> conversation, ai_expertise -> expertise atd.
- ollama_* pole zachovat prefix ollama_ uvnitr OllamaState (ollama_status -> status, ollama_models -> models atd.)

### Sub-struct granularita
- 3 vnorene sub-structy uvnitr AiState:
  - `ChatState`: prompt, history, history_index, monologue, conversation, system_prompt, response, loading, focus_requested, last_payload, in_tokens, out_tokens
  - `OllamaState`: status (OllamaConnectionStatus), models, selected_model, check_rx, last_check, base_url, api_key
  - `AiSettings`: expertise, reasoning_depth, font_scale, language, selected_provider, show_settings
- Top-level pole v AiState: inspector_open, cancellation_token, markdown_cache
- Pristup: ws.ai.chat.prompt, ws.ai.ollama.status, ws.ai.settings.expertise

### Migracni pristup
- Inkrementalni migrace ve 3-4 commitech:
  1. Extract ChatState z WorkspaceState
  2. Extract OllamaState z WorkspaceState
  3. Extract AiSettings z WorkspaceState
  4. Wire AiState do WorkspaceState (top-level pole + imports)
- Kazdy krok presouva pole A zaroven prejmenovava (ai_prompt -> prompt) — zadny mezikrok se starymi nazvy
- Kazdy commit musi kompilovat bez warningu

### Umisteni kodu
- AiState, ChatState, OllamaState, AiSettings definovany v novem souboru src/app/ai/state.rs
- OllamaConnectionStatus enum presunout z workspace/state/mod.rs do src/app/ai/state.rs
- Re-export pres src/app/ai/mod.rs

### Claude's Discretion
- Presny field ordering uvnitr sub-structu
- Default impl strategie pro AiState a sub-structy
- Poradi inkrementalnich kroku (ktery sub-struct prvni)
- Handling borrow checker issues pri pristupu k vnorenym sub-structum

</decisions>

<specifics>
## Specific Ideas

- ai_tool_* pole zustavaji ve WorkspaceState — tykaji se detekce externich CLI nastroju (Claude CLI apod.), ne noveho PolyCredo CLI
- markdown_cache presunout do AiState i kdyz se pouziva v editoru — editor si muze ziskat vlastni instanci
- Vsechna pole ktera zustavaji ve WS jsou bud terminalova (claude_tabs), plugin/WASM (pending_ask_user) nebo UI flags (show_ai_chat, ai_viewport_open)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- src/app/ai/types.rs: Existujici AI typy (AiExpertiseRole, AiReasoningDepth, OllamaStatus) — nektere se presunou nebo re-exportuji
- src/app/ai/mod.rs: AiManager — zustava nezmeneny, ale bude importovat z noveho state.rs
- src/app/ai/provider.rs + ollama.rs: Provider trait a implementace z Phase 13 — nepresouva se, ale OllamaState bude referencovat typy odtud

### Established Patterns
- mpsc::Receiver pro async vysledky: ollama_check_rx se presouva do OllamaState — pattern zustavame konzistentni
- Sub-struct pattern: Jiz existuje v codebase (ProjectProfiles, WizardState) — AiState nasleduje stejny vzor
- Derive macros: #[derive(Default)] pro structy se rozumnymi default hodnotami

### Integration Points
- WorkspaceState (workspace/state/mod.rs): Hlavni misto zmeny — pole se odstranuji, pridava se `pub ai: AiState`
- init.rs (workspace/state/init.rs): Inicializace se zmeni — AiState::default() nebo explicitni init
- Vsechny soubory referencujici ws.ai_* pole: Hromadny rename na ws.ai.chat.*, ws.ai.ollama.*, ws.ai.settings.*
- background.rs: Polling logika pro ollama_check_rx se presmeruje na ws.ai.ollama.check_rx

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 14-state-refactor*
*Context gathered: 2026-03-06*
