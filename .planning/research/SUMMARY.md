# Project Research Summary

**Project:** PolyCredo Editor - Additional Themes
**Domain:** Rust/egui Desktop Editor Theme System Extension
**Researched:** 2026-03-10
**Confidence:** HIGH

---

## Executive Summary

PolyCredo Editor je Rust/egui desktop textový editor s existujícím theme systémem (3 light varianty, 1 dark). Výzkum ukazuje, že přidání 4. light theme a volitelného 2. dark theme je **nízkoriziková, dobře izolovaná změna** — žádné nové závislosti nejsou potřeba, existující architektura plně podporuje rozšíření přes `LightVariant` / `DarkVariant` enumy.

**Doporučený přístup:**
1. **Fáze 1 (nutná):** Přidat 4. light variantu ("Stone" nebo "Cream") — 0.5-1 MD, nízké riziko
2. **Fáze 2 (volitelná):** Opravit syntect theme mapping — 0.5 MD
3. **Fáze 3 (volitelná):** Přidat DarkVariant enum pro druhé dark téma — 1-1.5 MD

**Klíčová rizika:**
- Syntect syntax highlighting ignoruje `LightVariant` — vrací "Solarized (light)" pro všechny
- Hardcoded barvy v UI komponentách mohou porušit nová témata
- Chybějící i18n klíče způsobí zobrazení názvu enum místo lokalizace

---

## Key Findings

### Recommended Stack

Žádné nové závislosti nejsou potřeba. Existující stack plně pokrývá požadavky:

| Technology | Purpose | Rationale |
|------------|---------|-----------|
| eframe/egui 0.31 | Framework + UI | Podporuje `Visuals` struct s plnou customizací barev |
| egui_extras 0.31 | Komponenty | UI picker pro varianty |
| syntect | Syntax highlighting | Téma mapováno přes `syntect_theme_name()` |

**Změny v kódu:**
- `src/settings.rs` — přidat variantu do enum + barvy v `to_egui_visuals()`
- `src/app/ui/workspace/modal_dialogs/settings.rs` — UI picker rozšíření
- 5× i18n soubory — nové překladové klíče

### Expected Features

**Must have (table stakes):**
- Nová `LightVariant` v enum — rozšíření existujícího systému
- Barvy v `to_egui_visuals()` — `panel_fill`, `window_fill`, `faint_bg_color`
- i18n label — lokalizovaný název v Settings UI
- UI swatch — náhled v Settings kartě
- Serializace — serde roundtrip pro persistence

**Should have (differentiators):**
- Per-variant syntect mapping — odlišné barvy syntax highlighting pro CoolGray/Sepia
- Per-variant accent colors — odlišné barvy pro selection, hyperlinky
- Dark variant selector — druhé dark téma (Deep Navy / High Contrast)

**Defer (v2+):**
- Vlastní theme editor — mimo rozsah v1.3.0
- Animované přechody témat — egui nepodporuje
- Theme export/import — zbytečná komplexita

### Architecture Approach

Architektura je **Visuals-first**: všechny komponenty přijímají `&egui::Visuals`, ne `LightVariant`. Přidání nových variant nevyžaduje změny v render logice — pouze definici barev v `to_egui_visuals()`.

```
Settings (Settings struct)
    │
    ▼ to_egui_visuals()
egui::Visuals (read-only)
    │
    ├─► Terminal Theme (blends from panel_fill)
    ├─► File Tree (reads visuals)
    ├─► Git Status (reads visuals)
    └─► Settings UI (label + swatch + i18n)
```

**Klíčové principy:**
1. Visuals-first: render komponenty nevidí enum, pouze hotové barvy
2. Heuristic detection: `warm_ivory_bg()` detekuje teplé/studené tóny automaticky
3. i18n decoupling: názvy témat jsou klíče, ne hardcoded stringy

### Critical Pitfalls

1. **Syntect Theme Not Mapped to Light Variants** — `syntect_theme_name()` vrací "Solarized (light)" pro všechny light varianty. Nutno mapovat CoolGray → base16-ocean.light, Sepia → vhodné téma.

2. **Hardcoded Colors Override Theme** — UI komponenty používají `Color32::from_rgb()` místo `ui.visuals()`. Prevence: audit všech přímých barev.

3. **Terminal/FileTree Desync on Theme Change** — komponenty cacheují Visuals při konstrukci. Prevence: používat `ui.visuals()` uvnitř paint/show callbacků.

4. **Missing serde Deserialize for New Enum Variants** — přidání nové varianty může rozbít deserializaci existujícího settings.toml. Prevence: mít `#[default]` na existující variantě.

5. **Missing UI Strings for New Variants** — chybějící i18n klíče zobrazí enum název. Prevence: aktualizovat `light_variant_label_key()` ve všech 5 jazycích.

---

## Implications for Roadmap

### Phase 1: 4th Light Theme
**Rationale:** Nejjednodušší změna, nízké riziko, plně podporovaná existující architekturou. Odhad: 0.5-1 MD.

**Delivers:**
- Nová `LightVariant::Stone` (mezi Sepia a brown)
- Barvy v `to_egui_visuals()`: panel_fill ~RGB(235,228,218)
- UI picker rozšíření v Settings
- i18n klíče do všech 5 locales

**Addresses:**
- FEATURES: nová light varianta (MVP)
- STACK: žádné nové závislosti

**Avoids:**
- PITFALL #4: serde deserializace — otestovat roundtrip
- PITFALL #5: i18n klíče — přidat do všech souborů

### Phase 2: Syntect Theme Mapping Fix
**Rationale:** Každá light varianta má jiné optimální barvy pro syntax highlighting. Nutno opravit před release.

**Delivers:**
- Rozšířená `syntect_theme_name()`:
  - WarmIvory → "Solarized (light)"
  - CoolGray → "base16-ocean.light"
  - Sepia → "Solarized (light)"
  - Stone → "InspiredGitHub" nebo ekvivalent

**Avoids:**
- PITFALL #1: syntect mismatch — hlavní riziko v1.3.0

### Phase 3: Dark Variant Support (Optional)
**Rationale:** Vyšší komplexita, vyžaduje nový `DarkVariant` enum a UI selector. Odhad: 1-1.5 MD.

**Delivers:**
- `DarkVariant::Default` + `DarkVariant::Midnight`
- Vlastní barvy v dark branch `to_egui_visuals()`
- UI selector (radiobutton/picker) pod dark_theme toggle
- Odlišné syntect mapping pro dark varianty

**Addresses:**
- FEATURES: dark variant selector (differentiator)
- STACK: DarkVariant enum v settings.rs

**Avoids:**
- PITFALL #2: audit hardcoded barev před přidáním dark varianty

### Research Flags

**Needs research:**
- **Phase 3 (Dark Variant):** Které syntect téma je nejlepší pro "Midnight" dark variant? Vyžaduje testování více témat.

**Standard patterns (skip research):**
- **Phase 1:** Light variant — standardní egui patterns, plně zdokumentované v kódu
- **Phase 2:** Syntect mapping — jednoduchá změna, žádné API research

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Žádné nové závislosti, egui 0.31 plně pokrývá |
| Features | HIGH | Jasný MVP rozsah, varianty jsou additive |
| Architecture | HIGH | Visuals-first pattern plně pochopen, kód analyzován |
| Pitfalls | HIGH | Známé problémy z existujícího kódu + tech debt |

**Overall confidence:** HIGH

### Gaps to Address

- **Syntect theme choice:** Které téma je optimální pro novou Stone light variantu? Ověřit vizuálně při implementaci.
- **Dark variant design:** Jaké barvy pro Midnight variant? Není hotový design — nechat na Phase 3 planning.
- **WCAG kontrast:** Formální testování kontrastu není součástí research — provést před release.

---

## Sources

### Primary (HIGH confidence)
- `src/settings.rs` — LightVariant enum, to_egui_visuals(), existing implementation
- `src/app/ui/workspace/modal_dialogs/settings.rs` — UI picker, light_variant_swatch
- `src/app/ui/terminal/instance/theme.rs` — terminal color blending heuristics

### Secondary (MEDIUM confidence)
- egui GitHub issue #4490 — custom theme support
- egui GitHub PR #4744 — set_dark_style/set_light_style API

### Tertiary (LOW confidence)
- WebAIM WCAG guidelines — pro kontrast testování (potřebuje validaci)

---

*Research completed: 2026-03-10*
*Ready for roadmap: yes*
