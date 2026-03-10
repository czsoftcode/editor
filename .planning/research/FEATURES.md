# Feature Landscape: Additional Themes

**Domain:** Rust/egui Desktop Editor Theme Extensions  
**Researched:** 2026-03-10  
**Mode:** Feature Research for v1.3.0

---

## Context Summary

PolyCredo Editor má existující theme systém:
- **Dark mode:** 1 varianta (výchozí, `base16-ocean.dark` syntect)
- **Light mode:** 3 varianty — WarmIvory, CoolGray, Sepia
- **Implementace:** `LightVariant` enum → `to_egui_visuals()` → `ctx.set_visuals()`
- **Persist:** `settings.toml` s `light_variant` polem
- **UI:** Settings modal s live kartami (swatch + label)
- **i18n:** 5 jazyků, klíče v `ui.ftl`

**Cíl v1.3.0:**
- Přidat 4. light theme (mezi Sepia a brown, "ne moc tmavá")
- (Volitelně) 2. dark theme

---

## Table Stakes

Základní features nutné pro jakékoli nové téma. Bez nich téma nebude fungovat.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Nová varianta v enum** | Rozšíření `LightVariant` / `DarkVariant` | Low | Přidat položku do existujícího enum |
| **Barvy v `to_egui_visuals()`** | Definice `panel_fill`, `window_fill`, `faint_bg_color` | Low | Podle vzoru existujících variant |
| **i18n label** | Lokalizovaný název v Settings UI | Low | Přidat do všech 5 `.ftl` souborů |
| **UI swatch barva** | Náhled v Settings kartě | Low | `light_variant_swatch()` funkce |
| **Serializace/deserializace** | Persistence přes serde | Low | Automaticky pokryto serde derive |
| **Live preview** | Okamžitá změna bez restartu | Low | Existující infrastructure stačí |

---

## Differentiators

Volitelné features, které nejsou nutné, ale mohou zlepšit UX.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Vlastní syntect téma** | Odlišné barvy pro syntax highlighting mezi light variantami | Medium | Aktuálně sdílejí "Solarized (light)" — možno přidat mapping |
| **Per-variant accent color** | Odlišné barvy pro selection, hyperlinky | Medium | Vyžaduje rozšíření `to_egui_visuals()` |
| **Dark variant selector** | Druhé dark téma (např. více/méně kontrastní) | Medium | Vyžaduje nový `DarkVariant` enum a UI |
| **Theme-aware ikony** | Ikony které se přizpůsobí tématu | Low | Egui standardní ikony jsou theme-aware |

---

## Anti-Features

Explicitně NEBUDOVAT.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Vlastní theme editor** | Mimo rozsah v1.3.0 (explicitně v PROJECT.md out of scope) | Použít předdefinované varianty |
| **Animované přechody** | Egui nepodporuje, není v roadmap | Instant switch je standard |
| **Theme export/import** | Zbytečná komplexita pro 4-5 témat | settings.toml stačí |
| **OS auto-detect** | Záměrně vynecháno v v1.0.2 | Ruční výběr zůstává |

---

## Feature Dependencies

```
LightVariant (enum rozšíření)
    ↓
to_egui_visuals() (match větev)
    ↓
light_variant_swatch() (UI)
    ↓
i18n klíče (5 jazyků)
    ↓
Live preview v Settings (existující)
```

**Pro DarkVariant (volitelné):**
```
DarkVariant (nový enum)
    ↓
to_egui_visuals() (dark branch rozšíření)
    ↓
dark_variant_swatch() (nová UI funkce)
    ↓
dark_variant_label_key() (nová i18n funkce)
    ↓
Settings UI rozšíření (radiobutton/selector)
```

---

## MVP Recommendation

### Priorita 1: 4. Light Theme

**Nutné kroky:**
1. Přidat `Cream` (nebo `Sand`) do `LightVariant` enum v `settings.rs`
2. Definovat barvy v `to_egui_visuals()`:
   - `panel_fill`: ~RGB(238, 225, 205) — mezi Sepia(240,230,210) a brown
   - `window_fill`: ~RGB(232, 218, 195)
   - `faint_bg_color`: ~RGB(220, 205, 180)
3. Přidat `light_variant_swatch()` match větev
4. Přidat i18n klíče do 5 locales (en, cs, de, ru, sk)
5. Ověřit `syntect_theme_name()` — aktuálně vrací "Solarized (light)" pro všechny

**Barva navrhována:** "Cream" nebo "Sand" — mezi Sepia a classic brown, teplá ale světlá.

### Priorita 2: Dark Theme Varianta (volitelné)

**Pokud bude chtěno:**
1. Přidat `DarkVariant` enum do `settings.rs`
2. Rozšířit `Settings` struct o `dark_variant: DarkVariant`
3. Definovat barvy v `to_egui_visuals()` pro dark branch
4. Přidat UI selector v Settings (pod dark_theme toggle)
5. Persist v settings.toml

**Dark variant options:**
- **Deep Navy:** Výrazně tmavší než výchozí base16-ocean
- **High Contrast:** Ostřejší barvy pro lepší čitelnost
- **Warm Dark:** Hnědé/oranžové akcenty místo modrých

---

## Implementation Complexity

| Komponenta | Závislosti | Odhad |
|------------|------------|-------|
| Nová light varianta | settings.rs, i18n, UI | 0.5-1 MD |
| Dark variant selector | settings.rs + nový enum + UI | 1-1.5 MD |
| Per-variant syntect | highlighter.rs mapping | 0.5 MD |

**MD = Manday** (8 hodin práce)

---

## Testing Requirements

| Test | Pokrytí |
|------|---------|
| `LightVariant::Cream` roundtrip serialize/deserialize | settings.rs testy |
| Live preview v Settings | Manuální UAT |
| Kontrast barev (WCAG AA) | Vizuální kontrola |
| Terminal theme适配 | Ověřit egui_term barvy |
| File tree git colors | Ověřit vizuální konzistenci |

---

## Sources

- **Existující kód:** `src/settings.rs` (LightVariant enum, to_egui_visuals)
- **UI:** `src/app/ui/workspace/modal_dialogs/settings.rs` (light_variant_swatch, karty)
- **i18n:** `locales/*/ui.ftl` (settings-light-variant-* klíče)
- **egui theming:** Context7/oficiální dokumentace — Visuals struct s panel_fill, window_fill, faint_bg_color
