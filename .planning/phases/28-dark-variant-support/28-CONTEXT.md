# Phase 28: Dark Variant Support - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning
**Source:** STATE.md decisions + ROADMAP.md

<domain>
## Phase Boundary

Add 2nd dark theme variant (DarkVariant enum) similar to LightVariant pattern from Phase 27.

</domain>

<decisions>
## Implementation Decisions

### Dark Variant Name
- Dark variant will be named `DarkVariant::Midnight` (per STATE.md decision)

### Architecture
- Use same pattern as LightVariant (enum with to_egui_visuals() match)
- Replace simple `dark_theme: bool` with `dark_variant: DarkVariant`

### Claude's Discretion
- Exact RGB colors for Midnight variant (dark blue/gray tones)
- Whether to keep backward compatibility with existing boolean dark_theme setting

</decisions>

<specifics>
## Specific Ideas

- Midnight: dark blue-gray tones, distinct from current dark theme
- Similar UI picker pattern as LightVariant
- i18n support for variant name
</specifics>

<deferred>
## Deferred Ideas

None — Phase 28 covers adding the 2nd dark variant

</deferred>

---

*Phase: 28-dark-variant-support*
*Context gathered: 2026-03-10*
