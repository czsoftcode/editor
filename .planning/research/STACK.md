# Technology Stack: Additional Themes

**Project:** PolyCredo Editor
**Researched:** 2026-03-10
**Focus:** Stack changes needed for 4th light theme + optional 2nd dark theme

---

## Current Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| eframe/egui | 0.31 | Desktop framework + UI toolkit |
| egui_extras | 0.31 | Additional egui components |
| syntect | (via egui_commonmark) | Syntax highlighting theme |

---

## Existing Theme System

### Data Model
- `dark_theme: bool` — boolean for dark/light mode
- `light_variant: LightVariant` — enum for light theme variants

### LightVariant enum (src/settings.rs:61-68)
```rust
pub enum LightVariant {
    WarmIvory,  // default
    CoolGray,
    Sepia,
}
```

### Theme Conversion (src/settings.rs:302-328)
```rust
pub fn to_egui_visuals(&self) -> eframe::egui::Visuals {
    if self.dark_theme {
        eframe::egui::Visuals::dark()
    } else {
        let mut visuals = eframe::egui::Visuals::light();
        match self.light_variant {
            LightVariant::WarmIvory => { /* colors */ }
            LightVariant::CoolGray => { /* colors */ }
            LightVariant::Sepia => { /* colors */ }
        }
        visuals
    }
}
```

### Integration Points
1. **Settings persistence** — `settings.toml` via serde
2. **UI picker** — `src/app/ui/workspace/modal_dialogs/settings.rs:371-386`
3. **Syntect highlighting** — `syntect_theme_name()` returns `"Solarized (light)"`
4. **Terminal colors** — `src/app/ui/terminal/instance/theme.rs` blends against `panel_fill`
5. **i18n** — 5 language files with variant labels

---

## Changes Required

### 1. 4th Light Theme (mandatory)

#### A. Add variant to enum (src/settings.rs)
```rust
pub enum LightVariant {
    WarmIvory,
    CoolGray,
    Sepia,
    // NEW: between Sepia (240,230,210) and Brown
    // Suggested name: "Stone" or "Sand"
    Stone,
}
```

#### B. Add colors in to_egui_visuals()
Colors should be:
- Between Sepia `rgb(240,230,210)` and Brown
- Light enough to not feel "dark"
- Distinct from existing variants

Example colors for new variant:
```rust
LightVariant::Stone => {
    visuals.panel_fill = eframe::egui::Color32::from_rgb(235, 228, 218);
    visuals.window_fill = eframe::egui::Color32::from_rgb(229, 221, 209);
    visuals.faint_bg_color = eframe::egui::Color32::from_rgb(218, 208, 193);
}
```

#### C. Update UI picker (src/app/ui/workspace/modal_dialogs/settings.rs:375-379)
Add new card to variant loop:
```rust
for variant in [
    LightVariant::WarmIvory,
    LightVariant::CoolGray,
    LightVariant::Sepia,
    LightVariant::Stone,  // NEW
] { /* ... */ }
```

#### D. Update helper functions (settings.rs)
- `light_variant_label_key()` — add new match arm
- `light_variant_swatch()` — add new swatch color

#### E. i18n translations (5 files)
Add to each locale file (cs, en, de, ru, sk):
```
settings-light-variant-stone = [translation]
```

#### F. Tests (src/settings.rs)
Add test coverage:
- Round-trip serialization
- Distinct panel_fill from other variants
- Terminal background distinctness

#### G. Terminal theme (no changes expected)
The `warm_ivory_bg()` heuristic (line 38-45 in theme.rs) uses r-b threshold:
```rust
fn warm_ivory_bg(panel_fill: egui::Color32) -> &'static str {
    if panel_fill.r() as i32 - panel_fill.b() as i32 > 10 {
        "#f5f2e8"  // warm
    } else {
        "#f3f5f7"  // cold
    }
}
```
New variant will blend automatically. May need test update.

---

### 2. Optional 2nd Dark Theme

**Complexity:** Higher than light variants

#### Option A: DarkVariant enum (recommended)
```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub enum DarkVariant {
    #[default]
    Default,  // current base16-ocean.dark
    Midnight, // NEW: deeper blue-black
}
```

**Changes needed:**
1. Add `dark_variant: DarkVariant` to Settings struct
2. Update `to_egui_visuals()` — currently just returns `Visuals::dark()`
3. Custom colors for Midnight variant (override dark mode defaults)
4. Update syntect theme mapping (different dark theme)
5. UI picker for dark variant (similar to light)
6. i18n keys
7. Tests

#### Option B: Dark theme as simple preset
Keep `dark_theme: bool`, just swap syntect theme name for different dark option. Limited customization, simpler but less flexible.

---

## Source Files to Modify

| File | Changes |
|------|---------|
| `src/settings.rs` | Enum + to_egui_visuals + helpers + tests |
| `src/app/ui/workspace/modal_dialogs/settings.rs` | UI picker loop + helpers |
| `locales/cs/ui.ftl` | Translation key |
| `locales/en/ui.ftl` | Translation key |
| `locales/de/ui.ftl` | Translation key |
| `locales/ru/ui.ftl` | Translation key |
| `locales/sk/ui.ftl` | Translation key |
| `src/app/ui/terminal/instance/theme.rs` | Possibly test update |

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Light variant changes | HIGH | Standard pattern, existing infrastructure supports |
| Dark variant option | MEDIUM | No DarkVariant exists yet, more design work |
| Terminal integration | HIGH | Heuristic-based, should work automatically |
| i18n scope | HIGH | Known pattern from existing variants |

---

## No Stack Additions Required

- **No new dependencies** — existing egui API sufficient
- **No version bumps** — 0.31 supports required customization
- **No breaking changes** — additive only

---

## Sources

- `src/settings.rs` — LightVariant enum, to_egui_visuals(), tests
- `src/app/ui/workspace/modal_dialogs/settings.rs` — Theme picker UI
- `src/app/ui/terminal/instance/theme.rs` — Terminal color blending
- `locales/*/ui.ftl` — i18n keys for variants
- egui 0.31 documentation — Visuals API verification
- `.planning/PROJECT.md` — Current theme system context

---

*Stack research for: additional themes milestone*
*Researched: 2026-03-10*
