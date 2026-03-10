# Architecture Research: Additional Themes Integration

**Domain:** Rust/egui Desktop Editor Theme System
**Researched:** 2026-03-10
**Confidence:** HIGH

## Existing Theme Architecture

The editor uses a centralized theme system based on `eframe::egui::Visuals`. The architecture follows a **single source of truth** pattern where theme configuration lives in `Settings` and propagates to all UI components.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Settings Layer                                │
├─────────────────────────────────────────────────────────────────────┤
│  src/settings.rs                                                    │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  LightVariant enum (WarmIvory, CoolGray, Sepia)           │  │
│  │  dark_theme: bool                                           │  │
│  │  to_egui_visuals() → egui::Visuals                         │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼ propagates Visuals
┌─────────────────────────────────────────────────────────────────────┐
│                    Consumption Layer (read-only)                     │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐   │
│  │ Terminal Theme  │  │  File Tree     │  │  Git Status     │   │
│  │ terminal/      │  │ file_tree/     │  │  git_status.rs  │   │
│  │ instance/      │  │ render.rs      │  │                  │   │
│  │ theme.rs       │  │                │  │                  │   │
│  └────────┬────────┘  └────────┬────────┘  └────────┬─────────┘   │
│           │                    │                     │              │
│           ▼                    ▼                     ▼              │
│  All use: ui.visuals() or Settings::to_egui_visuals()            │
│  No direct enum access — pure Visuals input                       │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Settings UI Layer                               │
├─────────────────────────────────────────────────────────────────────┤
│  src/app/ui/workspace/modal_dialogs/settings.rs                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  light_variant_label_key() → i18n key                      │  │
│  │  light_variant_swatch() → Color32 preview                  │  │
│  │  show_light_variant_card() → clickable picker UI           │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Integration Points

### 1. Core Theme Definition

**File:** `src/settings.rs`

| Component | Current State | Change Required |
|-----------|---------------|-----------------|
| `LightVariant` enum | 3 variants | Add new variant(s) |
| `to_egui_visuals()` | 3 match arms | Add color definitions |
| Serde serialization | Works | Automatic (enum derive) |

**Example structure for new variant (in to_egui_visuals):**
```rust
LightVariant::WarmIvory => {
    visuals.panel_fill = egui::Color32::from_rgb(255, 252, 240);
    visuals.window_fill = egui::Color32::from_rgb(250, 246, 235);
    visuals.faint_bg_color = egui::Color32::from_rgb(247, 241, 226);
}
```

### 2. Settings UI (Theme Picker)

**File:** `src/app/ui/workspace/modal_dialogs/settings.rs`

| Function | Change Required |
|----------|-----------------|
| `light_variant_label_key()` | Add i18n key mapping |
| `light_variant_swatch()` | Add color swatch preview |
| Theme selector loop (line 375-379) | Add variant to iteration |

### 3. Terminal Theme Colors

**File:** `src/app/ui/terminal/instance/theme.rs`

| Component | Current State | Change Required |
|-----------|---------------|-----------------|
| `terminal_palette()` | Uses `visuals.dark_mode` | No change (automatic) |
| `warm_ivory_bg()` | Heuristic via r-b threshold | May need adjustment |
| `tone_light_palette()` | Blends colors based on visuals | No change (automatic) |

**Architecture note:** Terminal themes work automatically because they take `&Visuals` as input, not `LightVariant`. The `warm_ivory_bg()` heuristic detects WarmIvory by `r() > b() + 10`.

### 4. Git Status Colors

**File:** `src/app/ui/git_status.rs`

| Component | Change Required |
|-----------|-----------------|
| `git_color_for_visuals()` | No change (uses Visuals) |
| Tests | Add new variant to test coverage |

### 5. File Tree Render

**File:** `src/app/ui/file_tree/render.rs`

| Component | Change Required |
|-----------|-----------------|
| `resolve_file_tree_git_color()` | No change (uses Visuals) |
| Tests | Add new variant to test coverage |

### 6. i18n Strings

**File:** `src/i18n.rs`

| Language | Change Required |
|----------|-----------------|
| cs (Czech) | Add theme name translation |
| en (English) | Add theme name translation |
| de (German) | Add theme name translation |
| ru (Russian) | Add theme name translation |
| sk (Slovak) | Add theme name translation |

**Key pattern:** `"settings-light-variant-{name}"` format (snake_case)

## Files That Require Modification

### Primary (must modify)

| File | Lines | Changes |
|------|-------|---------|
| `src/settings.rs` | 63-68, 308-324 | Enum + to_egui_visuals match |
| `src/app/ui/workspace/modal_dialogs/settings.rs` | 16-30, 375-379 | UI picker |

### Secondary (tests)

| File | Purpose |
|------|---------|
| `src/settings.rs` | Test new variant roundtrip |
| `src/app/ui/terminal/instance/theme.rs` | Test distinct terminal colors |
| `src/app/ui/git_status.rs` | Test distinct git status tones |
| `src/app/ui/file_tree/render.rs` | Test distinct file tree tones |

### Optional (automatic, but verify)

| File | Expected Behavior |
|------|-------------------|
| `src/app/ui/terminal/instance/theme.rs` | Works automatically via Visuals |
| `src/app/ui/git_status.rs` | Works automatically via Visuals |
| `src/app/ui/file_tree/render.rs` | Works automatically via Visuals |

## Key Design Principles

### 1. Visuals-First Architecture
All rendering components receive `&egui::Visuals`, not `LightVariant`. This means:
- Adding new variants doesn't require touching render logic
- The `warm_ivory_bg()` heuristic detects WarmIvory by color analysis
- Terminal/file_tree/git colors blend dynamically based on panel_fill

### 2. Heuristic Detection
The `warm_ivory_bg()` function (line 38-45 in terminal/theme.rs) uses:
```rust
if panel_fill.r() as i32 - panel_fill.b() as i32 > 10 {
    "#f5f2e8" // warm tone
} else {
    "#f3f5f7" // cool base
}
```
**Implication:** New light variants that are warm-toned (>10 r-b) will blend with warm ivory; cool-toned variants will use the cool base.

### 3. i18n Decoupling
Theme names use i18n keys, not hardcoded strings. This allows translation without code changes.

## Anti-Patterns to Avoid

### 1. Direct LightVariant in Render Layer
**Wrong:** `match light_variant { ... }` in terminal/theme.rs
**Correct:** Use `visuals.panel_fill` to derive colors

### 2. Hardcoding Theme Names
**Wrong:** `if variant == "Sepia"`
**Correct:** Use enum matching with i18n keys for display

### 3. Missing Test Coverage
**Risk:** New variants may look identical to existing ones
**Prevention:** Tests verify `HashSet` of backgrounds has distinct values

## Build Order

For implementing new themes:

1. **Phase 1:** Add `LightVariant` enum + `to_egui_visuals()` in settings.rs
2. **Phase 2:** Add i18n translations (all 5 languages)
3. **Phase 3:** Add UI picker in settings.rs (label + swatch + iteration)
4. **Phase 4:** Add/update tests in settings.rs (roundtrip)
5. **Phase 5:** Verify terminal/file_tree/git colors work automatically
6. **Phase 6:** Update tests in terminal/git/file_tree (distinctness)

## Sources

- Implementation files analyzed:
  - `src/settings.rs` (LightVariant, to_egui_visuals)
  - `src/app/ui/workspace/modal_dialogs/settings.rs` (UI picker)
  - `src/app/ui/terminal/instance/theme.rs` (terminal colors)
  - `src/app/ui/git_status.rs` (git status colors)
  - `src/app/ui/file_tree/render.rs` (file tree colors)
- `.planning/PROJECT.md` (milestone context)

---

*Architecture research for: PolyCredo Editor v1.3.0 Additional Themes*
*Researched: 2026-03-10*
