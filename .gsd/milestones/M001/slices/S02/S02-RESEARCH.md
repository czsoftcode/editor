# Phase 1: Základ — Research

**Researched:** 2026-03-04
**Domain:** egui Visuals theming + syntect parametrizace (Rust + eframe/egui)
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| THEME-01 | Settings obsahuje `LightVariant` enum (WarmIvory, CoolGray, Sepia) s `#[serde(default)]` | Settings struct ověřen — chybí pouze LightVariant pole; vzor `#[serde(default)]` + `impl Default` je standardní Rust/Serde pattern |
| THEME-02 | Settings obsahuje metody `to_egui_visuals()` a `syntect_theme_name()` odvozené z aktivního tématu | egui Visuals API ověřeno; `Visuals::light()`/`Visuals::dark()` jsou vstupní body; syntect ThemeSet obsahuje "Solarized (light)" jako built-in téma |
| THEME-03 | Téma je aplikováno při startu přes `cc.egui_ctx.set_visuals()` (bez úvodního bliknutí) | `EditorApp::new()` je správné místo; `Settings::apply()` již existuje, ale je voláno jen v `update()` — nutno přidat i do `new()` |
| THEME-04 | Změna tématu se propaguje přes existující `settings_version` mechanismus | Mechanismus plně ověřen ze zdrojového kódu: `AppShared::settings_version` (AtomicU64), `applied_settings_version` per-viewport, volání `settings.apply(ctx)` při rozdílu verzí |
| EDIT-01 | Highlighter přijímá název syntect tématu jako parametr (bez hardcoded "base16-ocean.dark") | `src/highlighter.rs:77` a `:119` — hardcoded string potvrzen; `highlight()` signatura musí přijmout `theme_name: &str` |
| EDIT-02 | Light mode používá "Solarized (light)" syntect téma | syntect `ThemeSet::load_defaults()` obsahuje "Solarized (light)" jako vestavěné téma — ověřeno z docs.rs |
| EDIT-03 | Dark mode používá "base16-ocean.dark" (stávající chování zachováno) | Stávající hardcoded hodnota — stačí ji parametrizovat, default zůstane stejný |
| EDIT-04 | Highlighter vyčistí cache pouze při skutečné změně tématu (ne každý frame) | Cache `HashMap<u64, Arc<LayoutJob>>` existuje; hash klíč musí zahrnout `theme_name`; clear jen při `set_theme()` volání |
| SETT-04 | Načtení starého settings.toml (bez `light_variant`) neskončí pádem | `#[serde(default)]` na poli + `impl Default for LightVariant` — standardní Serde vzor; existující Settings používá tento vzor konzistentně |
| UI-01 | Menu bar, dialogy a všechny egui widgety respektují aktivní téma (bez hardcoded tmavých barev) | `ctx.set_visuals()` přepíná globálně; egui widgety automaticky čerpají barvy z Visuals — žádné hardcoded barvy v menu/dialogu nebyly nalezeny v kódu Phase 1 scope |
| UI-02 | Záložky editoru — indikátor neuloženého stavu (●) je čitelný v obou módech | Indikátor používá textový znak; čitelnost závisí na `ui.visuals().text_color()` — egui ji odvozuje automaticky z Visuals |
| UI-03 | Status bar text je čitelný v obou módech | Stejný princip jako UI-02 — egui automaticky odvozuje text barvy z aktivních Visuals |
</phase_requirements>

---

## Summary

Phase 1 je foundation pro celý dark/light milestone. Cílem je funkční přepínač dark/light v Settings panelu, čitelný syntax highlighting v obou módech, a eliminace startup flash. Veškerá potřebná infrastruktura již existuje — `settings_version` mechanismus, `Settings::apply()`, `Highlighter` se syntect cache — a vyžaduje pouze cílené rozšíření.

Klíčové změny jsou tři: (1) rozšíření `Settings` struct o `LightVariant` enum s `#[serde(default)]`, implementace `to_egui_visuals()` a `syntect_theme_name()` metod; (2) parametrizace `Highlighter::highlight()` tak, aby přijímal `theme_name: &str` a zahrnoval jej do cache hash klíče; (3) aplikování tématu v `EditorApp::new()` přes `cc.egui_ctx.set_visuals()` aby první frame byl vykreslován správným tématem.

Žádné nové závislosti nejsou nutné. egui Visuals API automaticky propaguje téma do všech widgetů, menu a dialogů — UI-01, UI-02, UI-03 jsou proto zdarma po správném nastavení `ctx.set_visuals()`. Terminál a git barvy jsou záměrně mimo scope Phase 1 (patří do Phase 2).

**Primary recommendation:** Implementovat v pořadí Settings struct → Highlighter parametrizace → Settings::apply() rozšíření → startup apply → Settings modal UI.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| egui | 0.31 | GUI framework, Visuals API | Již v projektu; `Visuals::light()`/`dark()` + `ctx.set_visuals()` je přesně co potřebujeme |
| syntect | 5.x | Syntax highlighting | Již v projektu; `ThemeSet::load_defaults()` obsahuje "Solarized (light)" |
| serde | 1.x | Serializace Settings | Již v projektu; `#[serde(default)]` zajistí zpětnou kompatibilitu |
| toml | 0.x | Settings persistence | Již v projektu; `settings.toml` je aktivní formát |

### Alternativy neuvažovány

Phase 1 nevyžaduje žádné nové závislosti. Celá implementace je rozšíření existujícího kódu.

**Installation:** Žádné nové balíčky.

---

## Architecture Patterns

### Recommended Project Structure

Žádné nové soubory/adresáře. Změny jsou v existujících souborech:

```
src/
├── settings.rs              -- +LightVariant enum, +light_variant pole, +to_egui_visuals(), +syntect_theme_name()
├── highlighter.rs           -- parametrizovat highlight(), background_color(); přidat set_theme()
├── app/
│   ├── mod.rs               -- přidat settings.apply(ctx) do EditorApp::new()
│   └── ui/workspace/
│       └── modal_dialogs/
│           └── settings.rs  -- přidat dark/light radio buttony (již existují), beze změny pro Phase 1
```

### Pattern 1: LightVariant enum s Serde default

**Co:** Nový enum v `Settings` struct, odvozující egui Visuals a syntect téma.

**Kdy použít:** Vždy — jedná se o centrální datový model Phase 1.

```rust
// src/settings.rs
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Default, Debug)]
pub enum LightVariant {
    #[default]
    WarmIvory,
    CoolGray,
    Sepia,
}

// V Settings struct:
#[serde(default)]
pub light_variant: LightVariant,

// Metody:
impl Settings {
    pub fn syntect_theme_name(&self) -> &'static str {
        if self.dark_theme {
            "base16-ocean.dark"
        } else {
            "Solarized (light)"
        }
    }

    pub fn to_egui_visuals(&self) -> eframe::egui::Visuals {
        if self.dark_theme {
            return eframe::egui::Visuals::dark();
        }
        let mut v = eframe::egui::Visuals::light();
        // Phase 1: WarmIvory jako výchozí; konkrétní RGB pro varianty přijdou v Phase 3
        // Zatím: základní Visuals::light() bez per-variant barev
        v
    }
}
```

**Poznámka k Phase 1:** Per-variant `panel_fill` RGB hodnoty (LITE-01/02/03) jsou v scope Phase 3. Phase 1 implementuje `to_egui_visuals()` jako základ — `Visuals::light()` pro light mode, `Visuals::dark()` pro dark mode. LightVariant enum se připraví pro budoucí rozšíření.

### Pattern 2: Settings::apply() rozšíření

**Co:** Stávající `apply()` metoda rozšířena o `to_egui_visuals()` místo přímého `Visuals::dark()/light()`.

```rust
// src/settings.rs — existující metoda, malá změna
pub fn apply(&self, ctx: &eframe::egui::Context) {
    // ZMĚNA: místo if/else -> to_egui_visuals()
    ctx.set_visuals(self.to_egui_visuals());

    // BEZE ZMĚNY: font
    ctx.style_mut(|style| {
        style.text_styles.insert(
            eframe::egui::TextStyle::Monospace,
            eframe::egui::FontId::new(
                self.editor_font_size,
                eframe::egui::FontFamily::Monospace,
            ),
        );
    });
}
```

### Pattern 3: Highlighter parametrizace

**Co:** `highlight()` přijme `theme_name: &str`, hash klíč ho zahrne, cache se invaliduje na změnu tématu.

```rust
// src/highlighter.rs

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    // PŘIDAT: aktuální téma pro detekci změny
    current_theme: std::sync::Mutex<String>,
    cache: std::sync::Mutex<HashMap<u64, Arc<egui::text::LayoutJob>>>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: std::sync::Mutex::new("base16-ocean.dark".to_string()),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Volat při změně tématu — invaliduje cache pouze při skutečné změně (EDIT-04)
    pub fn set_theme(&self, theme_name: &str) {
        let mut current = self.current_theme.lock().expect("lock");
        if *current != theme_name {
            *current = theme_name.to_string();
            self.cache.lock().expect("lock").clear();
        }
    }

    pub fn highlight(
        &self,
        text: &str,
        extension: &str,
        filename: &str,
        font_size: f32,
        theme_name: &str,  // NOVÝ PARAMETR (EDIT-01)
    ) -> Arc<egui::text::LayoutJob> {
        // Cache key musí zahrnout theme_name (EDIT-04)
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        extension.hash(&mut hasher);
        filename.hash(&mut hasher);
        ((font_size * 100.0) as u32).hash(&mut hasher);
        theme_name.hash(&mut hasher);  // PŘIDAT
        let key = hasher.finish();

        // ... zbytek beze změny, jen:
        let theme = &self.theme_set.themes[theme_name];  // místo hardcoded
        // ...
    }

    pub fn background_color(&self, theme_name: &str) -> egui::Color32 {
        let theme = &self.theme_set.themes[theme_name];  // místo hardcoded
        // ...
    }
}
```

### Pattern 4: Startup apply — bez flash (THEME-03)

**Co:** `Settings::apply()` volat v `EditorApp::new()` přes `cc.egui_ctx`, ne jen v `update()`.

```rust
// src/app/mod.rs — v EditorApp::new(), před return Self { ... }
// Po načtení settings:
let settings = std::sync::Arc::new(crate::settings::Settings::load());

// PŘIDAT ihned po načtení settings:
settings.apply(&cc.egui_ctx);

// ... zbytek inicializace
```

**Proč:** Bez tohoto volání egui zobrazí první frame s default Visuals (tmavé), než `update()` stihne zavolat `apply()`. Výsledek je viditelný flash.

**Ověření:** `EditorApp::new()` aktuálně nenastavuje Visuals — potvrzeno ze zdrojového kódu (řádek 88–320 `src/app/mod.rs`).

### Anti-Patterns to Avoid

- **Hardcoded `"base16-ocean.dark"` v highlight():** Zaměnit za parametr. Dvě místa: řádky 77 a 119 v `src/highlighter.rs`.
- **`Color32` barvy uložené v Settings:** Ukládat enum (`LightVariant`), odvozovat barvy v kódu.
- **`ctx.set_visuals()` každý frame:** Výhradně přes `settings_version` mechanismus — `apply()` jen při změně verze.
- **Čistit celou highlighter cache každý frame:** Volat `set_theme()` jen při skutečné změně tématu.
- **Volat `ThemeSet::load_defaults()` při každém highlight:** `ThemeSet` je v `Highlighter` struct a vytváří se jednou v `new()`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Přepínání dark/light egui widgetů | Custom widget coloring | `ctx.set_visuals(Visuals::light()/dark())` | egui Visuals propaguje barvy do všech widgetů automaticky |
| Light syntax highlighting | Custom highlighting engine | `syntect` s "Solarized (light)" téma | ThemeSet::load_defaults() obsahuje built-in light témata |
| Zpětná kompatibilita Settings | Ruční migrace | `#[serde(default)]` + `impl Default` | Serde to řeší automaticky při deserializaci |
| Per-frame theme detection | Polling dark_theme každý frame | `settings_version` AtomicU64 mechanismus | Již implementován, odolný vůči race conditions |

**Key insight:** egui Visuals je globální styl — jedno volání `ctx.set_visuals()` přepne menu bar, dialogy, panely, widgety, scrollbary najednou. Žádný custom theming kód není potřeba.

---

## Common Pitfalls

### Pitfall 1: Startup flash (dark frame před light)
**Co se stane:** První frame egui použije default dark Visuals, pak `update()` zavolá `apply()` — uživatel vidí záblesk tmavého okna.
**Proč:** `EditorApp::new()` aktuálně nevolá `settings.apply(&cc.egui_ctx)`.
**Jak se vyhnout:** Přidat `settings.apply(&cc.egui_ctx)` do `EditorApp::new()` ihned po `Settings::load()`.
**Varovné signály:** Viditelný flash při spuštění s light mode.

### Pitfall 2: Cache collision po změně tématu
**Co se stane:** Highlighter vrátí dark-colored LayoutJob pro light mode, protože hash klíč neobsahuje `theme_name`.
**Proč:** Stávající cache hash nezahrnuje téma.
**Jak se vyhnout:** Přidat `theme_name.hash(&mut hasher)` do cache key výpočtu v `highlight()`.
**Varovné signály:** Po přepnutí na light mode kód zůstane tmavě obarvený.

### Pitfall 3: ThemeSet index panic
**Co se stane:** `&self.theme_set.themes["Solarized (light)"]` způsobí panic pokud název tématu neexistuje.
**Proč:** `themes` je `HashMap<String, Theme>` — přímé indexování panikuje při chybějícím klíči.
**Jak se vyhnout:** Použít `themes.get(theme_name).unwrap_or_else(|| &themes["base16-ocean.dark"])` jako fallback.
**Varovné signály:** Panic při prvním použití light mode.

### Pitfall 4: LightVariant bez `#[serde(default)]`
**Co se stane:** Načtení existujícího `settings.toml` (bez `light_variant` klíče) selže s Serde chybou → `Settings::default()` → ztrácíme jiná nastavení uživatele.
**Proč:** Serde vyžaduje všechna pole pokud není `#[serde(default)]` přítomno.
**Jak se vyhnout:** Přidat `#[serde(default)]` na `light_variant` pole + `#[derive(Default)]` nebo `impl Default for LightVariant` s `WarmIvory` jako výchozí hodnotou.
**Varovné signály:** Po přidání pole uživatelé přijdou o nastavení fontu, jazyka atd.

### Pitfall 5: `set_visuals()` voláno každý frame
**Co se stane:** Zbytečný výkon — `set_visuals()` je pomalejší než typický widget update.
**Proč:** Triviální chyba při nepoužití `settings_version` mechanismu.
**Jak se vyhnout:** Existující mechanismus (`applied_settings_version != v`) — volat `apply()` jen při rozdílu.
**Varovné signály:** Profiler ukazuje `set_visuals` jako hot path.

---

## Code Examples

### Plná SignSettings rozšíření

```rust
// src/settings.rs — kompletní nová část

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum LightVariant {
    #[default]
    WarmIvory,
    CoolGray,
    Sepia,
}

// V Settings struct přidat:
/// Light mode color variant (only relevant when dark_theme = false).
#[serde(default)]
pub light_variant: LightVariant,

// Metody:
impl Settings {
    pub fn syntect_theme_name(&self) -> &'static str {
        if self.dark_theme {
            "base16-ocean.dark"
        } else {
            "Solarized (light)"
        }
    }

    pub fn to_egui_visuals(&self) -> eframe::egui::Visuals {
        if self.dark_theme {
            eframe::egui::Visuals::dark()
        } else {
            // Phase 1: základní light; per-variant barvy přijdou v Phase 3
            eframe::egui::Visuals::light()
        }
    }
}
```

### Highlighter parametrizace (klíčové změny)

```rust
// src/highlighter.rs — pouze změněné části

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: std::sync::Mutex<String>,
    cache: std::sync::Mutex<HashMap<u64, Arc<egui::text::LayoutJob>>>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: std::sync::Mutex::new("base16-ocean.dark".to_string()),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Invaliduje cache pouze při změně tématu (EDIT-04).
    pub fn set_theme(&self, theme_name: &str) {
        let mut current = self.current_theme.lock().expect("Highlighter current_theme lock");
        if current.as_str() != theme_name {
            *current = theme_name.to_string();
            self.cache.lock().expect("Highlighter cache lock").clear();
        }
    }

    pub fn highlight(
        &self,
        text: &str,
        extension: &str,
        filename: &str,
        font_size: f32,
        theme_name: &str,  // NOVÝ: EDIT-01
    ) -> Arc<egui::text::LayoutJob> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        extension.hash(&mut hasher);
        filename.hash(&mut hasher);
        ((font_size * 100.0) as u32).hash(&mut hasher);
        theme_name.hash(&mut hasher);  // NOVÝ: zahrnout téma do klíče (EDIT-04)
        let key = hasher.finish();

        // ... cache check stejný ...

        let theme = self.theme_set.themes.get(theme_name)
            .unwrap_or_else(|| &self.theme_set.themes["base16-ocean.dark"]);  // fallback
        let mut h = HighlightLines::new(syntax, theme);
        // ... zbytek beze změny
    }

    pub fn background_color(&self, theme_name: &str) -> egui::Color32 {
        let theme = self.theme_set.themes.get(theme_name)
            .unwrap_or_else(|| &self.theme_set.themes["base16-ocean.dark"]);
        if let Some(bg) = theme.settings.background {
            egui::Color32::from_rgb(bg.r, bg.g, bg.b)
        } else {
            egui::Color32::from_rgb(43, 48, 59)
        }
    }
}
```

### Settings::apply() s to_egui_visuals()

```rust
// src/settings.rs
pub fn apply(&self, ctx: &eframe::egui::Context) {
    ctx.set_visuals(self.to_egui_visuals());  // ZMĚNA
    ctx.style_mut(|style| {
        style.text_styles.insert(
            eframe::egui::TextStyle::Monospace,
            eframe::egui::FontId::new(self.editor_font_size, eframe::egui::FontFamily::Monospace),
        );
    });
}
```

### Startup apply v EditorApp::new()

```rust
// src/app/mod.rs — přidat ihned po Settings::load()
let settings = std::sync::Arc::new(crate::settings::Settings::load());

// PŘIDAT: apply theme before first frame (THEME-03, eliminates startup flash)
settings.apply(&cc.egui_ctx);
```

### Settings modal — dark/light radio (existující, kontrola)

```rust
// src/app/ui/workspace/modal_dialogs/settings.rs — BEZE ZMĚNY pro Phase 1
// Již existuje:
ui.radio_value(&mut draft.dark_theme, true, i18n.get("settings-theme-dark"));
ui.radio_value(&mut draft.dark_theme, false, i18n.get("settings-theme-light"));
// LightVariant picker přijde v Phase 3 (SETT-01)
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded `"base16-ocean.dark"` v highlight() | Parametrizovaný `theme_name: &str` | Phase 1 | EDIT-01, EDIT-02, EDIT-03 |
| `Visuals::dark()/light()` přímo v apply() | `self.to_egui_visuals()` | Phase 1 | Základ pro THEME-02 |
| Visuals nastaveny jen v update() | Nastaveny v new() i update() | Phase 1 | Eliminace startup flash (THEME-03) |

**Deprecated/outdated:**
- `Highlighter::background_color()` bez parametru: musí přijmout `theme_name: &str`
- Přímý if/else v `Settings::apply()` pro dark/light: zaměnit za `to_egui_visuals()`

---

## Open Questions

1. **Highlighter volání sites**
   - Co víme: `highlight()` je voláno z `editor.rs` (záložky); `background_color()` tamtéž
   - Co je nejasné: Přesný počet call sites — nutno auditovat při implementaci
   - Doporučení: Grep `highlighter.highlight(` a `background_color(` před implementací

2. **`Default` impl pro `Settings` a nové pole**
   - Co víme: `Settings` má `impl Default` manuálně (ne `#[derive(Default)]`)
   - Co je nejasné: Je nutno přidat `light_variant: LightVariant::default()` do `impl Default for Settings`
   - Doporučení: Přidat explicitně do `Default` impl bloku

---

## Validation Architecture

> `nyquist_validation: true` v config.json — sekce povinná.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (vestavěný) |
| Config file | `Cargo.toml` (žádná extra konfigurace) |
| Quick run command | `cargo test --lib 2>&1 \| tail -20` |
| Full suite command | `cargo test 2>&1` |

Projekt již obsahuje `#[cfg(test)]` modul v `src/highlighter.rs` (řádky 128–161) s performance testem — infrastruktura existuje.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| THEME-01 | `LightVariant` enum se serializuje/deserializuje + default při chybějícím poli | unit | `cargo test --lib settings::tests` | ❌ Wave 0 |
| THEME-02 | `to_egui_visuals()` vrátí `dark_mode: true` pro dark, `false` pro light | unit | `cargo test --lib settings::tests` | ❌ Wave 0 |
| THEME-02 | `syntect_theme_name()` vrátí "base16-ocean.dark" pro dark a "Solarized (light)" pro light | unit | `cargo test --lib settings::tests` | ❌ Wave 0 |
| THEME-03 | Startup bez flash — vizuální ověření | manual | Spustit `cargo run`, pozorovat start s light mode | N/A |
| THEME-04 | `settings_version` se inkrementuje po Save v settings modalu | unit | `cargo test --lib settings::tests` | ❌ Wave 0 |
| EDIT-01 | `highlight()` přijme `theme_name` parametr a použije ho | unit | `cargo test --lib highlighter::tests` | ✅ (rozšířit) |
| EDIT-02 | Light mode vrátí LayoutJob s světlejšími barvami (jiný hash než dark) | unit | `cargo test --lib highlighter::tests` | ✅ (rozšířit) |
| EDIT-03 | Dark mode vrátí stejný výsledek jako před refactoringem | unit | `cargo test --lib highlighter::tests::test_dark_unchanged` | ❌ Wave 0 |
| EDIT-04 | `set_theme()` invaliduje cache; opakované volání stejného tématu cache nezahazuje | unit | `cargo test --lib highlighter::tests::test_cache_invalidation` | ❌ Wave 0 |
| SETT-04 | TOML bez `light_variant` pole se načte bez paniku s `WarmIvory` jako default | unit | `cargo test --lib settings::tests::test_sett04_backward_compat` | ❌ Wave 0 |
| UI-01 | Menu bar, dialogy bez hardcoded barev — kompilace bez regresí | smoke | `cargo build 2>&1` | N/A |
| UI-02 | Záložkový indikátor ● čitelný — vizuální kontrola | manual | Spustit s light mode, otevřít soubor, upravit | N/A |
| UI-03 | Status bar text čitelný — vizuální kontrola | manual | Spustit s light mode | N/A |

### Sampling Rate

- **Per task commit:** `cargo test --lib 2>&1 | tail -20`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** `cargo test 2>&1` plně zelený před `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/settings.rs` — přidat `#[cfg(test)] mod tests` s testy:
  - `test_theme01_light_variant_serde_default` — deserializace TOML bez pole
  - `test_theme02_to_egui_visuals_dark_mode_flag` — dark_mode bool
  - `test_theme02_syntect_theme_name` — správné string hodnoty
  - `test_sett04_backward_compat` — TOML bez `light_variant` neskončí pádem
- [ ] `src/highlighter.rs` — rozšířit existující `mod tests`:
  - `test_edit01_theme_name_parameter` — `highlight()` přijme parametr
  - `test_edit03_dark_unchanged` — dark výstup shodný před/po refactoringu
  - `test_edit04_cache_invalidation` — `set_theme()` invaliduje cache, stejné téma ne

*(Existující `test_highlight_performance_10k` je nutno aktualizovat pro nový parametr `theme_name`.)*

---

## Sources

### Primary (HIGH confidence)

- `src/settings.rs` — projekt — kompletní Settings struct, `apply()` metoda, serde vzory
- `src/highlighter.rs` — projekt — hardcoded "base16-ocean.dark" na ř.77 a 119, cache implementace
- `src/app/mod.rs` — projekt — `EditorApp::new()`, `settings_version` mechanismus, `applied_settings_version`
- `src/app/ui/workspace/modal_dialogs/settings.rs` — projekt — existující dark/light radio buttons
- syntect ThemeSet — docs.rs — `load_defaults()`, built-in témata včetně "Solarized (light)"
- egui Visuals — docs.rs — `Visuals::light()`, `Visuals::dark()`, `ctx.set_visuals()`

### Secondary (MEDIUM confidence)

- catppuccin/egui — GitHub — vzor pro theme-aware egui aplikace (více variant Visuals)
- egui GitHub discussions #2050, #1627 — Visuals theming vzory a `set_visuals()` best practices

### Tertiary (LOW confidence)

- Žádné LOW confidence zdroje pro Phase 1 scope.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — vše ověřeno ze zdrojového kódu projektu a docs.rs
- Architecture: HIGH — settings_version mechanismus plně ověřen, volací místa identifikována
- Pitfalls: HIGH — hardcoded strings ověřeny na konkrétních řádcích kódu
- Validation: HIGH — test infrastruktura existuje (cargo test), Wave 0 gaps přesně identifikovány

**Research date:** 2026-03-04
**Valid until:** 2026-04-04 (stabilní stack, žádné fast-moving závislosti)