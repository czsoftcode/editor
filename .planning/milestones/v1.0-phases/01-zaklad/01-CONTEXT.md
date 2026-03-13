# Phase 1: Základ - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Implementace základního dark/light mode pro PolyCredo Editor. Pokrývá egui Visuals, syntax highlighting, a UI panely. Výsledkem je funkční přepínač tématu bez flash při startu a čitelný syntax highlighting v obou módech.

**Co je v scope:**
- egui Visuals přepínání (dark/light)
- Syntax highlighting s Solarized Light
- Zpětná kompatibilita settings.json
- UI widgety bez hardcoded barev

**Co je mimo scope:**
- Light varianty (Phase 3)
- Terminálové téma (Phase 2)
- Git barvy ve file tree (Phase 2)
- OS auto-detect

</domain>

<decisions>
## Implementation Decisions

### Theme Model
- LightVariant enum: WarmIvory (default), CoolGray, Sepia
- #[serde(default)] pro zpětnou kompatibilitu settings.json
- Metody syntect_theme_name() a to_egui_visuals() v Settings

### Syntax Highlighting
- Dark mode: "base16-ocean.dark" (stávající)
- Light mode: "Solarized (light)"
- Highlighter přijímá téma jako parametr

### Startup Behavior
- Téma aplikováno v new() ne update() — žádný flash
- settings_version mechanismus pro propagaci změn

### Persistence
- light_variant persistován v settings.json
- Staré settings.json bez light_variant nespadne (serde default)

</decisions>

<specifics>
## Specific Ideas

Z PROJECT.md:
- "žádný tmavý záblesk při startu"
- "žlutá barva nesplývá s bílým pozadím"
- Terminál zůstává tmavý (Phase 2)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- src/settings.rs: Existující Settings struct — rozšířit o light_variant
- src/highlighter.rs: Existující syntect highlighter
- egui Visuals: ctx.set_visuals() pro přepínání

### Established Patterns
- Settings používá serde pro TOML serializaci
- settings_version pro change detection

### Integration Points
- Settings::apply() volá ctx.set_visuals()
- Highlighter potřebuje téma z Settings

</code_context>

<deferred>
## Deferred Ideas

- Phase 2: Terminál téma (egui_term)
- Phase 2: Git barvy ve file tree
- Phase 3: Light varianty (WarmIvory, CoolGray, Sepia detaily)
- OS auto-detect dark/light preference

</deferred>

---

*Phase: 01-zaklad*
*Context gathered: 2026-03-04*
