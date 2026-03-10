# Phase 15: Streaming Chat UI - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Uzivatel muze vest konverzaci s AI pres novy nativni chat s plnym streamingem a vizualnim formatovanim. Chat pouziva nativni OllamaProvider (Phase 13) misto WASM pluginu. Zahrnuje: hybrid CLI layout, streaming rendering, dark/light mode, markdown, historie, model picker, Ollama settings v Settings dialogu. Tool calling a i18n patri do Phase 16 a 17.

</domain>

<decisions>
## Implementation Decisions

### Umisteni chatu
- Floating okno (StandardTerminalWindow) — NE docked do right panelu
- Right panel zustava pro terminalove taby (Claude CLI terminaly)
- Moznost otevrit jako separate viewport/window (jiny monitor)
- Klavesova zkratka pro otevreni (dedicka zkratka, napr. Ctrl+Shift+I)
- Resizable okno (StandardTerminalWindow uz to podporuje)

### Vizualni styl zprav
- Barevne bloky — user zprava ma jedno pozadi, AI odpoved jine
- Metadata u kazde zpravy: casove razitko (hh:mm), role label (You / nazev modelu), token count u AI odpovedi, copy tlacitko
- Code blocky se syntax highlightingem (pouzit syntect, uz v codebase) + copy tlacitko na code blocku
- Theme-aware barvy odvozene z ui.visuals() — automaticky funguje v dark i light mode
- Zadne hardcoded barvy (nahradit existujici Color32::from_rgb(...) v render.rs)

### Streaming chovani
- Inkrementalni markdown rendering behem streamu — bufferovat text, re-renderovat egui_commonmark kazdych N tokenu
- Stop tlacitko v prompt baru: behem streamingu se Send tlacitko zmeni na Stop. Plus Escape jako klavesova zkratka (uz implementovano pres cancellation_token)
- Po preruseni streamu: zachovat castecnou odpoved v konverzaci + stitek '[preruseno]'
- Auto-scroll s moznosti zastaveni: automaticky scrollovat dolu behem generovani, pokud uzivatel manualne scrollne nahoru, autoscroll se zastavi. Tlacitko 'scroll to bottom' pro obnoveni

### Ollama Settings
- Nova sekce 'AI' v Settings dialogu (modal_dialogs/settings.rs)
- Polozky: Ollama Base URL, Ollama API Key, Expertise role, Reasoning depth, vychozi model + vsechna nastaveni prevzata z plugin settings
- Zmeny se aplikuji po ulozeni (Save), pri zmene URL se okamzite spusti novy check na novou adresu
- Automaticka migrace z plugin settings pri startu editoru — precist Ollama hodnoty z plugin settings do nove AI sekce. Stare hodnoty ponechat (WASM plugin jeste neni odebran az do Phase 17)

### Claude's Discretion
- Presny interval re-renderu markdown behem streamu (kazdych kolik tokenu)
- Konkretni theme barvy pro user vs AI bloky (jaka visuals() pole pouzit)
- Layout details info baru a footer baru
- Presna klavesova zkratka pro otevreni chatu
- Migracni logika — jak detekovat ze plugin settings jeste nebyly migrovany

</decisions>

<specifics>
## Specific Ideas

- PolyCredo CLI chat se nedokuje do right panelu — right panel je vyhrazeny pro terminalove taby
- Aktualni chat UI v ai_chat/ uz ma zakladni CLI layout (prompt dole, historie nahore) — rozsirit, ne prepisovat od nuly
- egui_commonmark uz se pouziva pro markdown rendering (markdown_cache v AiState)
- cancellation_token (Arc<AtomicBool>) uz existuje a Escape handler je implementovany
- Prevzit VSECHNA nastaveni Ollama z plugin settings, ne jen URL a API key

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ai_chat/render.rs`: Existujici chat layout (render_body, render_chat_content, render_footer) — zaklad pro rozsireni
- `ai_chat/logic.rs`: send_query_to_agent logika — prepojit na OllamaProvider.stream_chat()
- `ai_chat/mod.rs`: AiChatAction enum, handle_action, StandardTerminalWindow integrace
- `ai_bar.rs`: Ollama status ikona + model ComboBox — uz funkcni
- `AiState` (ai/state.rs): ChatState, OllamaState, AiSettings — kompletni stav po Phase 14
- `AiProvider` trait + `OllamaProvider` (ai/provider.rs, ai/ollama.rs): stream_chat() vraci mpsc::Receiver<StreamEvent>
- `egui_commonmark::CommonMarkCache` v AiState — markdown rendering cache
- `syntect` (highlighter.rs): Syntax highlighting — pouzit pro code blocky v chatu
- `AiChatWidget` (ui/widgets/ai/): ui_conversation, ui_monologue, ui_input, ui_settings metody

### Established Patterns
- mpsc::Receiver pro async vysledky (build_error_rx, git_status_rx, ollama_check_rx)
- StandardTerminalWindow pro floating okna (head/body/footer pattern)
- ui.visuals() pro theme-aware barvy (dark/light mode)
- egui::Frame s fill() pro barevne pozadi bloku
- cancellation_token (Arc<AtomicBool>) pro preruseni async operaci
- Toast (AppAction::ShowToast) pro user feedback

### Integration Points
- `ai_chat/logic.rs`: Prepojit send_query z WASM pluginu na OllamaProvider.stream_chat()
- `modal_dialogs/settings.rs`: Pridat novou AI sekci
- `settings.rs`: Pridat AI-specificka pole do Settings struct
- `background.rs`: Polling StreamEvent z mpsc::Receiver behem frame update
- `workspace/state/mod.rs`: show_ai_chat flag pro otevreni/zavreni

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 15-streaming-chat-ui*
*Context gathered: 2026-03-06*
