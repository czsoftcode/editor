# Phase 33: odstranit veskerou zminku a funkce polycredo cli ze systemu - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit interni integrovany AI subsystem navazany na `ai_core` + `ui/terminal/ai_chat` a ponechat jen `ai_bar` jako jednoduchy launcher prikazu do terminalu.
Po zmene aplikace nebude provozovat vlastni AI runtime/chat execution; pouze odesle vybrany prikaz do aktivniho terminal tabu.

</domain>

<decisions>
## Implementation Decisions

### Co zustava a co se maze
- `src/app/ui/terminal/right/ai_bar.rs` zustava.
- `src/app/ai_core/*` bude odstraneno.
- `src/app/ui/terminal/ai_chat/*` bude odstraneno.
- Vysledne chovani: pouze `ai_bar -> send_command` bez interniho AI runtime.

### Chovani bez fallbacku
- Po odstraneni neimplementovat UX fallbacky, disabled stavy ani toast upozorneni pro stare akce.
- Stare AI chat entrypointy a routovani budou tvrde odstraneny (bez hlasek pro koncoveho uzivatele).

### Rozsah mazani zminen
- Zminky `ai_core`/`ai_chat` odstranit i v historickych/planning artefaktech (nejen v aktivnim kodu).
- Cilem je totalni cleanup textovych i kodovych referenci napric repozitarem.

### Claude's Discretion
- Poradi mazani a navaznych compile-fix kroku.
- Minimalni sadu zbytnych glue-uprav nutnych k tomu, aby projekt po odstraneni kompiloval.
- Finalni regression gate sestavu pro potvrzeni, ze zustava pouze `ai_bar` launcher tok.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/terminal/right/ai_bar.rs`: samostatna UI lista, ktera jen vybira agenta a vola `terminal.send_command(&cmd)`.
- `src/app/ui/terminal/instance/*`: existujici terminal backend/instance vrstva, na kterou se ma vysledny flow oprit.

### Established Patterns
- Aktualni AI flow je dnes provazany pres `ui/terminal/ai_chat/*` + `ai_core/*` + `workspace/background` stavy (`show_ai_chat`, `tool_executor`).
- Projektovy gate zustava `cargo check` + `./check.sh`.

### Integration Points
- Odstraneni `ai_chat` zasahne `src/app/ui/workspace/*`, `src/app/ui/panels.rs`, `src/app/ui/background.rs`, `src/app/mod.rs` a souvisejici testy.
- Po odstraneni musi zustat funkcni pouze cesta z `ai_bar` do aktivniho terminal tabu.

</code_context>

<specifics>
## Specific Ideas

- "Neintegrovat AI do programu; nechat jen listu na terminalu."
- "ai_bar zustat, ai_core a ai_chat pryc."
- "Zadne hlaseni chyb/fallbacku pro stare akce."

</specifics>

<deferred>
## Deferred Ideas

- None — zadane rozhodnuti je totalni odstraneni integrovaneho AI subsystemu.

</deferred>

---

*Phase: 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu*
*Context gathered: 2026-03-11*
