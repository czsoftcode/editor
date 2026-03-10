---
phase: 13-provider-foundation
plan: 02
status: complete
started: 2026-03-06
completed: 2026-03-06
---

# Summary: 13-02 UI napojení OllamaProvider

## What Was Built

Polling detekce Ollama serveru, status ikona a model ComboBox v AI baru (dočasné umístění — přesune se do PolyCredo CLI po jeho implementaci).

## Key Files

### Modified
- `src/config.rs` — OLLAMA_CHECK_INTERVAL_SECS (10s), OLLAMA_DEFAULT_URL
- `src/app/ui/workspace/state/mod.rs` — OllamaConnectionStatus enum, ollama_* fields
- `src/app/ui/workspace/state/init.rs` — inicializace ollama polí, immediate first check
- `src/app/ui/workspace/state/actions.rs` — persistence ollama_selected_model
- `src/app/types.rs` — ollama_selected_model v PersistentState
- `src/app/ui/background.rs` — polling logika s ai_loading guardem
- `src/app/ui/terminal/right/ai_bar.rs` — status circle + model ComboBox (dočasné umístění)

## Decisions

- UI prvky (status ikona, model picker) dočasně v ai_bar.rs — přesunou se do PolyCredo CLI (AI Chat) po jeho implementaci
- Polling skip při ai_loading == true (neinterferuje se streamem)
- Immediate first check via last_check inicializovaný v minulosti

## Deviations

- Task 3 (vizuální checkpoint) schválen uživatelem jako "test" — finální umístění bude v PolyCredo CLI

## Verification

- 68 testů prošlo, cargo check OK
- Kompiluje bez chyb (2 expected warnings pro dosud nepoužité re-exporty)

## Self-Check: PASSED
