# Phase 2: Terminal + Git barvy - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Fáze 2 řeší čitelnost terminálů a git stavů ve file tree v light mode. Scope je:
- Claude panel (terminál) a Build terminál: světlé pozadí, čitelný text, theme-aware scrollbar
- Git status barvy ve file tree: čitelné M/A/??/D na světlém pozadí

Mimo scope této fáze:
- nové capabilities mimo téma terminál/file-tree (např. nové workflow funkce terminálu)
- light varianty palet v Settings (Phase 3)

</domain>

<decisions>
## Implementation Decisions

### Terminál — vizuální styl v light mode
- Pozadí terminálu má být mírně šedé (ne čistě bílé).
- Hlavní text má být tmavě neutrální (vysoký kontrast bez tvrdé černé).
- ANSI barvy mají být light-safe paleta: zachovat význam, doladit tóny pro čitelnost na světlém pozadí.
- Kurzor a výběr textu mají být středně výrazné (dobře viditelné, ne agresivní).

### Přepínání tématu za běhu
- Priorita je nepřerušit běžící procesy v terminálu.
- Změna tématu má být viditelná ihned po přepnutí.
- Historii výstupu se má systém pokusit sjednotit (recolor/re-render), pokud to backend dovolí.
- Pokud změna nepůjde aplikovat perfektně, použít tichý fallback (bez rušivého upozornění), pokud terminál zůstane použitelný.

### Git barvy ve file tree (light mode)
- Použít samostatnou light paletu (ne jen mechanické ztmavení dark barev).
- `??` (untracked) má být modrá/azurová, aby byla čitelná a odlišná.
- Celková intenzita git barev má být střední (viditelné, ale bez přebití názvů souborů).
- Při konfliktu indikací má prioritu git status.

### Scrollbar terminálu v light mode
- Scrollbar zůstává trvale viditelný.
- Track/thumb mají mít střední kontrast vůči světlému pozadí.
- Šířka scrollbaru se nemění (zachovat existující layout).
- Hover/drag stav thumbu má používat jemné zesílení.

### Claude's Discretion
- Přesné RGB/HEX hodnoty light-safe palety terminálu a git stavů.
- Konkrétní algoritmus pro sjednocení historického výstupu po přepnutí tématu.
- Detailní ladění alpha/opacity u výběru textu a overlay stavů.

</decisions>

<specifics>
## Specific Ideas

- „Ihned po přepnutí“ + „bez přerušení běhu“ je preferovaný UX kompromis pro runtime změnu tématu.
- U git statusů je kritický bod čitelnost `??` (untracked) na světlém pozadí.
- Scrollbar má být theme-aware, ale stále stabilně viditelný.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/terminal/instance/mod.rs`: společná implementace `Terminal::ui(...)` pro Claude i Build terminál.
- `src/app/ui/terminal/instance/render.rs`: vlastní kreslení scrollbaru terminálu (`draw_scrollbar`) — aktuálně hardcoded tmavé barvy.
- `src/app/ui/background.rs`: `parse_git_status(...)` mapuje git stavy na barvy (`M/A/??/D`).
- `src/app/ui/file_tree/render.rs`: aplikuje barvy git statusů a dnes v light mode barvy pouze násobí faktorem 0.55.

### Established Patterns
- Theme propagace běží přes `Settings` + `settings_version` mechanismus (aplikováno v `src/app/mod.rs`).
- Claude panel i Build panel používají stejný `Terminal` wrapper → změna v `instance` se propíše do obou.
- UI obecně používá `ui.visuals()` jako zdroj theme-aware hodnot; hardcoded barvy jsou zdroj regressí.

### Integration Points
- `src/app/ui/terminal/right/mod.rs` a `src/app/ui/terminal/bottom/mod.rs`: volání `terminal.ui(...)`.
- `src/app/ui/terminal/instance/render.rs`: napojení scrollbar stylu na aktivní téma.
- `src/app/ui/background.rs` + `src/app/ui/file_tree/render.rs`: produkce + rendering git status barev.
- `src/config.rs`: `TERMINAL_SCROLLBAR_WIDTH` (šířka má zůstat stabilní).

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-terminal-git-barvy*
*Context gathered: 2026-03-04*
