# Debug Session: 31-ollama-in-ai-terminal

Datum: 2026-03-11
Cíl: najít root cause, proč se v AI terminalu stále objevuje Ollama model, přestože scope požaduje pouze AI terminal tok bez prvků/navázání na Ollama.

## Shrnutí nálezu

Root cause je **scope/requirements drift mezi "locked" produkční hranicí a realizačními plány Phase 31**:
- Dokumentace a plány pro provedení (hlavně `31-02-PLAN.md` a `.planning/STATE.md`) explicitně drží "model picker continuity".
- Implementace tomu odpovídá: AI terminal UI i runtime jsou stále pevně navázané na `OllamaState`/`OllamaProvider`.

To znamená, že přítomnost Ollama modelu v AI terminalu není izolovaná chyba renderu, ale očekávaný výsledek toho, co bylo ve fázi 31 plánováno a dokončeno.

## Důkazy

1. Plán přímo vyžaduje zachování model pickeru
- `.planning/phases/31-ai-terminal-runtime-migration/31-02-PLAN.md:24`
  - "Model picker zůstává napojený na runtime stav..."
- `.planning/phases/31-ai-terminal-runtime-migration/31-02-PLAN.md:107-118`
  - Samostatný task `TERM-03 model picker + slash/GSD continuity`.

2. STATE explicitně drží model picker jako cílovou feature
- `.planning/STATE.md:24`
  - Cíl milestone zahrnuje "AI terminal chat + streaming + model picker + slash/GSD".
- `.planning/STATE.md:66`
  - Uvedeno, že "Model picker v ai_bar" je vědomé rozhodnutí ve Phase 31.

3. UI AI baru stále renderuje model combobox
- `src/app/ui/terminal/right/ai_bar.rs:57-95`
  - Label modelu + combobox nad `ws.available_ai_models()` a `ws.active_ai_model()`.

4. Workspace API je zadrátované přímo na Ollama state
- `src/app/ui/workspace/state/mod.rs:232-254`
  - `ai_provider_is_connected`, `ai_provider_connection_parts`, `active_ai_model`, `available_ai_models`, `set_active_ai_model` používají `self.ai.ollama.*`.

5. Background loop aktivně synchronizuje a polluje Ollamu
- `src/app/ui/background.rs:180-262`
  - Sync `ollama_base_url`/API key z Settings, periodické `spawn_ollama_check`, správa `ws.ai.ollama.models`, fetch model info.

6. Chat send path používá přímo `OllamaProvider`
- `src/app/ui/terminal/ai_chat/logic.rs:156-177`
  - Konstrukce provideru přes `OllamaProvider::new(...)` a stream přes tento provider.

7. AI core runtime provider je alias na Ollama
- `src/app/ai_core/mod.rs:3-7,13`
  - `runtime_provider` re-exportuje `ollama`; veřejně exportuje `spawn_ollama_check`.

## Závěr (root cause)

Primární příčina: **nesoulad scope a plánů (requirements drift)**. I když UAT truth požaduje bez-Ollama AI terminal, plánovací artefakty Phase 31 stále definovaly model picker/Ollama vazby jako požadované. Implementace je s těmito artefakty konzistentní, proto se Ollama model v AI terminalu dál zobrazuje.

Sekundární technická příčina: **silné architektonické couplingy AI terminalu na `OllamaState` a `OllamaProvider` v UI + workspace state + background polling + chat send path**.

## Fix direction (bez implementace)

1. Nejdřív sjednotit source-of-truth: aktualizovat phase/state požadavky tak, aby explicitně zakazovaly model/provider UI a runtime vazby na Ollamu v AI terminalu.
2. Následně provést oddělení AI terminalu od `ws.ai.ollama.*` API a odstranit model picker wiring z `ai_bar`.
3. Přesměrovat/abstrahovat chat runtime provider tak, aby terminal-only tok nebyl hardcoded na `OllamaProvider`.
