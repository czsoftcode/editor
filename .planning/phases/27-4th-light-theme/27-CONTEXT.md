# Phase 27: 4th Light Theme - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Přidat 4. světlé téma (mezi Sepia a hnědou, ne moc tmavé, ne kovové). Toto téma bude součástí LightVariant enum a bude mít vlastní syntect theme mapping.

</domain>

<decisions>
## Implementation Decisions

### Název varianty
- Nová varianta se jmenuje **WarmTan**

### Barvy (RGB)
- panel_fill: (215, 200, 185)
- window_fill: (205, 190, 175)
- faint_bg_color: (195, 180, 165)

### Syntect Theme Mapping
Každá light varianta má vlastní syntect theme:
- WarmIvory → "Solarized Light"
- CoolGray → "base16-ocean.light"
- Sepia → "Solarized Light"
- WarmTan → "GitHub Light"

### Swatch Design
- Standardní swatch - čtvereček barvy s názvem pod ním (jako ostatní varianty)

### i18n klíče
- cs: "Teplý Tan"
- en: "Warm Tan"  
- de: "Warne Tan"
- sk: "Teplý Tan"
- ru: "Тёплый тан"

### Výchozí varianta
- WarmIvory zůstává výchozí pro nové uživatele (neměnit)

</decisions>

<specifics>
## Specific Ideas

- WarmTan má být tmavší než Sepia a světlejší než hnědá
- Barvy mají být teplé, ne kovové (žádné šedé podtóny)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- LightVariant enum v src/settings.rs - rozšířit o WarmTan
- light_variant_label_key() - přidat klíč pro i18n
- light_variant_swatch() - přidat swatch pro UI picker
- to_egui_visuals() - přidat match větev pro WarmTan
- syntect_theme_name() - změnit z jednoho theme na per-variant mapping

### Established Patterns
- Každá light varianta má 3 barvy: panel_fill, window_fill, faint_bg_color
- UI picker iteruje přes všechny varianty v settings.rs
- i18n klíče jsou ve formátu "settings-light-variant-{name}"

### Integration Points
- src/settings.rs: enum, to_egui_visuals(), syntect_theme_name()
- src/app/ui/workspace/modal_dialogs/settings.rs: picker UI, label, swatch
- src/i18n.rs: lokalizace pro 5 jazyků

</code_context>

<deferred>
## Deferred Ideas

Žádné - diskuse zůstala v rámci scope fáze.

</deferred>

---

*Phase: 27-4th-light-theme*
*Context gathered: 2026-03-10*
