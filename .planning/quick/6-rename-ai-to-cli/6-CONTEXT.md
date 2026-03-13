# Quick Task 6: Přejmenovat src/app/ai → src/app/cli - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Task Boundary

Přejmenovat adresář src/app/ai na src/app/cli a aktualizovat všechny související mod/use cesty v Rust kódu.

</domain>

<decisions>
## Implementation Decisions

### Rozsah přejmenování
- Přejmenovat POUZE src/app/ai → src/app/cli
- Ostatní ai_ adresáře (widgets/ai, terminal/ai_chat, ai_bar, ai_panel) zůstávají beze změny

### Přejmenování typů
- NEPŘEJMENOVÁVAT Rust typy s Ai prefixem (AiManager, AiState, AiMessage, AiExpertiseRole, AiReasoningDepth…)
- Měnit pouze mod deklarace a use cesty (app::ai → app::cli)

### Claude's Discretion
- Soubory v .polycredo/sandbox_old/ ignorovat (zastaralá kopie)

</decisions>

<specifics>
## Specific Ideas

Dotčené soubory (hlavní kód, bez sandbox_old):
- src/app/mod.rs: `pub mod ai` → `pub mod cli`
- src/app/ai/mod.rs → src/app/cli/mod.rs (celý adresář)
- ~13 souborů s `use crate::app::ai::` importy
- ~61 výskytů `app::ai` celkem

</specifics>
