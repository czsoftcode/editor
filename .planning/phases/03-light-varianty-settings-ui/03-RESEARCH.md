# Phase 03 Research: Light varianty + Settings UI

Datum: 2026-03-04
Scope: egui/eframe theme varianty, live preview, persistence, patterny kompatibilni s aktualni codebase.

## Standard Stack

### 1) Theme model v egui + app-specific varianty
Preskriptivne:
- Nechat globalni theme prepinani na egui urovni (`dark`/`light`) a app-specific variantu drzet v aplikacnim modelu (`LightVariant`).
- `to_egui_visuals()` stavet z `egui::Visuals::light()`/`dark()` a prepisovat jen nutna pole (`panel_fill`, `faint_bg_color`, prip. `window_fill`).
- Neprepinat jednotlivym widgetum barvy rucne; jednotna paleta musi jit pres `Visuals`.

Duvod:
- egui ma nativne `Theme::Dark` / `Theme::Light` (+ preference `System`), ne custom sadu "light podvariant".
- `Visuals::light()` uz definuje konzistentni baseline (vcetne `panel_fill`, `faint_bg_color`, widget states).

Confidence: **High**

### 2) Live preview (bez restartu) pres stavajici runtime propagaci
Preskriptivne:
- V modalu drzet draft (`ws.settings_draft`), ale pri zmene theme controls okamzite pushnout runtime settings do `shared.settings` + inkrementovat `settings_version`.
- Persist (`save`) delat az na `Save`; `Cancel` musi vratit puvodni snapshot.
- Pouzit existujici propagacni smycku v `app/mod.rs` (root + deferred viewporty ceka na zmenu `settings_version` a aplikuje `settings.apply(ctx)`).

Duvod:
- To je presne kompatibilni s aktualni architekturou (`AppShared.settings`, `settings_version`, deferred viewport callbacky).

Confidence: **High**

### 3) Persistence
Preskriptivne:
- Canonical persistence pro tuto codebase nechat v `settings.toml` (`Settings::load/save`), nepresouvat to do eframe app storage.
- Nadale pouzit `serde` defaulty pro backward kompatibilitu (`#[serde(default)]` pro `light_variant`).
- Zachovat existujici migraci `settings.json -> settings.toml`.
- Pro phase acceptance mapovat SETT-03 na realny storage kontrakt projektu: zapis/cteni `settings.toml` + overena migrace legacy `settings.json`.

Duvod:
- Projekt uz ma explicitni domenu settings mimo eframe interni storage.
- eframe `get_value`/`set_value` serializuje do RON a je vhodne spis pro app-state persistence, ne pro uzivatelsky konfig file tohoto projektu.

Confidence: **High**

## Architecture Patterns

### Pattern A: "Single source of truth" + versioned broadcast
Pouzit stavajici pattern bez refaktoru:
1. Runtime source of truth je `AppShared.settings: Arc<Settings>`.
2. Kazda runtime zmena theme zvysi `settings_version`.
3. Root + secondary viewporty aplikují settings jen pri zmene verze.

To uz v projektu existuje a je to kompatibilni s egui multi-viewport modelem (deferred viewporty + `Arc<Mutex<...>>`).

Confidence: **High**

### Pattern B: Draft + original snapshot (pro skutecny Cancel revert)
Pro live preview je potreba doplnit snapshot:
1. Pri otevreni settings modalu ulozit `original_settings` (kopie aktualnich settings).
2. Zmeny v draftu okamzite previewnout do runtime (bez save na disk).
3. `Cancel` -> vratit `original_settings` do runtime + zavrit modal.
4. `Save` -> ulozit draft na disk; runtime uz je aplikovany.

Tento pattern je minimalni patch nad existujicim `settings_draft` flow.

Confidence: **High**

### Pattern C: "Only-on-change" apply
Aby live preview nebyl drahy:
- Triggerovat preview jen kdyz control vrati `.changed()`.
- Pred inkrementem verze porovnat theme fingerprint (napr. `(dark_theme, light_variant)`).

Duvod:
- Omezite zbytecne `settings_version` bump, repaint chain a resety cache v navaznych komponentech.

Confidence: **High**

## Don't Hand-Roll

1. Nehandrollovat dark/light/system prepinac.
Pouzijte egui API (`global_theme_preference_switch/buttons`) tam, kde staci global dark/light. Pro 3 light varianty jen doplnte app-level picker.
Confidence: **High**

2. Nehandrollovat kompletni styl od nuly.
Vychazet z `Visuals::light()` a prepsat jen par barev. Ruce psany kompletni `Visuals` casto rozbije kontrast/stavy widgetu.
Confidence: **High**

3. Nehandrollovat serializaci ani vlastni config parser.
Drzet `serde` + TOML (`Settings::load/save`), defaulty a migraci. Nepsat custom parser pro varianty.
Confidence: **High**

4. Nehandrollovat extra event bus pro synchronizaci oken.
Uz existuje robustni broadcast pres `settings_version` a aplikaci pri update passu.
Confidence: **High**

5. Negativni tvrzeni (overeno): egui nema out-of-the-box custom "tri light varianty" model.
`Theme` enum je jen `Dark`/`Light`; `ThemePreference` je `Dark`/`Light`/`System`.
Confidence: **High**

## Common Pitfalls

1. Live preview bez revert snapshotu.
Symptom: `Cancel` jen zavre modal, ale vizual zustane zmenen.
Mitigace: explicitni `original_settings` snapshot a restore pri Cancel.
Confidence: **High**

2. Bump `settings_version` kazdy frame.
Symptom: zbytecne repainty + potencialni vedlejsi efekty v highlighter/terminal propagaci.
Mitigace: preview poustet pouze na `.changed()` + fingerprint guard.
Confidence: **High**

3. Konflikt vice otevrenych Settings modalu v ruznych oknech.
Symptom: posledni modal prepise preview/revert prvniho.
Mitigace: bud globalne povolit jen jeden settings modal, nebo zavest preview token (owner viewport id) pro deterministicky commit/revert.
Confidence: **Medium**

4. Prilis maly kontrast panelu u light variant.
Symptom: file tree/editor/panel splývaji.
Mitigace: ladit `faint_bg_color` proti `panel_fill`; vychazet z faktu, ze default `faint_bg_color` je velmi jemny.
Confidence: **High**

5. Zapomenute i18n klice pro variant picker.
Symptom: fallback texty nebo chybejici labely v casti lokalizaci.
Mitigace: doplnit vsechny `locales/*/ui.ftl` soubory pri zavedení pickeru.
Confidence: **High**

## Code Examples

### Example 1: `Settings::to_egui_visuals()` pro 3 light varianty
```rust
use eframe::egui::{Color32, Visuals};

impl Settings {
    pub fn to_egui_visuals(&self) -> Visuals {
        if self.dark_theme {
            return Visuals::dark();
        }

        let mut v = Visuals::light();
        match self.light_variant {
            LightVariant::WarmIvory => {
                v.panel_fill = Color32::from_rgb(255, 252, 240);
                v.window_fill = Color32::from_rgb(252, 248, 236);
                v.faint_bg_color = Color32::from_rgb(245, 240, 226);
            }
            LightVariant::CoolGray => {
                v.panel_fill = Color32::from_rgb(242, 242, 242);
                v.window_fill = Color32::from_rgb(238, 238, 238);
                v.faint_bg_color = Color32::from_rgb(228, 228, 228);
            }
            LightVariant::Sepia => {
                v.panel_fill = Color32::from_rgb(240, 230, 210);
                v.window_fill = Color32::from_rgb(235, 224, 202);
                v.faint_bg_color = Color32::from_rgb(224, 212, 188);
            }
        }
        v
    }
}
```

### Example 2: Live preview trigger v Settings UI (jen pri zmene)
```rust
fn theme_fingerprint(s: &crate::settings::Settings) -> (bool, crate::settings::LightVariant) {
    (s.dark_theme, s.light_variant.clone())
}

fn apply_theme_preview(shared: &std::sync::Arc<std::sync::Mutex<crate::app::types::AppShared>>, draft: &crate::settings::Settings) {
    let mut sh = shared.lock().expect("lock");
    if theme_fingerprint(&sh.settings) != theme_fingerprint(draft) {
        sh.settings = std::sync::Arc::new(draft.clone());
        sh.settings_version
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

// v settings modalu:
let mut changed = false;
changed |= ui.radio_value(&mut draft.dark_theme, true, i18n.get("settings-theme-dark")).changed();
changed |= ui.radio_value(&mut draft.dark_theme, false, i18n.get("settings-theme-light")).changed();

if !draft.dark_theme {
    changed |= ui
        .selectable_value(&mut draft.light_variant, crate::settings::LightVariant::WarmIvory, i18n.get("settings-light-variant-warm"))
        .changed();
    changed |= ui
        .selectable_value(&mut draft.light_variant, crate::settings::LightVariant::CoolGray, i18n.get("settings-light-variant-cool"))
        .changed();
    changed |= ui
        .selectable_value(&mut draft.light_variant, crate::settings::LightVariant::Sepia, i18n.get("settings-light-variant-sepia"))
        .changed();
}

if changed {
    apply_theme_preview(shared, draft);
}
```

### Example 3: Cancel revert pri live preview
```rust
// WorkspaceState doplni:
// pub settings_original: Option<crate::settings::Settings>,

if ws.settings_draft.is_none() {
    let current = (*shared.lock().expect("lock").settings).clone();
    ws.settings_original = Some(current.clone());
    ws.settings_draft = Some(current);
}

match act {
    SettingsModalAction::Save => {
        if let Some(draft) = ws.settings_draft.take() {
            draft.save();
            ws.settings_original = None;
            // runtime uz je aplikovany pres preview
        }
    }
    SettingsModalAction::Cancel => {
        if let Some(original) = ws.settings_original.take() {
            let mut sh = shared.lock().expect("lock");
            sh.settings = std::sync::Arc::new(original);
            sh.settings_version
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        ws.settings_draft = None;
    }
}
```

## Validation Architecture

### Nyquist Validation Strategy (preskriptivne)

- Kazdy pozadavek `LITE-01..04` a `SETT-01..03` musi mit tri vrstvy validace: `unit`, `integration`, `manual UAT`.
- Zmena se nepovazuje za complete, pokud chybi kterakoliv vrstva bez explicitniho waiveru.
- `unit` vrstva overuje deterministickou logiku (mapovani variant -> visuals, guardy zmen).
- `integration` vrstva overuje propojeni hranic (`settings` model -> settings modal -> `AppShared.settings/settings_version` -> aplikace do root/deferred viewportu + persistence).
- `manual UAT` vrstva overuje vizualni citelnost, UX semantiku a multi-window chovani, ktere nelze spolehlive pokryt ciste automatizovane.

Confidence: **High**

### Test Layers (co validovat kde)

- `Unit tests`: `src/settings.rs` (+ male pure helpery v settings UI, pokud je treba je vyclenit) pro presne RGB mapovani, `faint_bg_color`, theme fingerprint a no-op guard.
- `Integration tests`: cross-module tok `Settings modal -> shared runtime -> settings_version propagate -> apply(ctx)` + save/load/migration cesta.
- `Manual UAT`: realne GUI overeni v root okne i secondary viewportu, vcetne `Save`/`Cancel` semantiky.

Confidence: **High**

### Requirement Matrix (LITE/SETT)

| Requirement | Unit validation (preskriptivne) | Integration validation (preskriptivne) | Manual UAT (preskriptivne) | Failure signals | Confidence |
|---|---|---|---|---|---|
| `LITE-01` WarmIvory | Assert `to_egui_visuals()` pro `dark_theme=false, light_variant=WarmIvory` vrati `panel_fill=rgb(255,252,240)` a ocekavana `window_fill/faint_bg_color` | Pri zmene varianty z modalu se runtime settings prepne na WarmIvory a `settings_version` bumpne jen pri realne zmene | V light mode zvolit WarmIvory, UI musi byt jemne teple bez ztraty kontrastu textu | Odchylka RGB, bez vizualni zmeny, zmena az po restartu | **High** |
| `LITE-02` CoolGray | Assert `panel_fill=rgb(242,242,242)` a konzistentni light visuals baseline | Preview prepnuti na CoolGray se projevi i v secondary viewportu po propagate passu | V light mode zvolit CoolGray, vzhled ma pusobit neutralne/chladne (GitHub/VS Code light styl) | Tone drift do teple palety, nepropagace do dalsich oken | **High** |
| `LITE-03` Sepia | Assert `panel_fill=rgb(240,230,210)` + ocekavana teplejsi `faint_bg_color` | Pri prepnuti Warm/Cool -> Sepia dojde k jedinemu version bumpu a aplikaci visuals | V light mode zvolit Sepia, tonalita musi byt zretelne teplejsi nez ostatni varianty | Varianta vypada skoro stejne jako jina, zmena se neprojevi live | **High** |
| `LITE-04` faint_bg_color odliseni | Assert pro vsechny 3 varianty: `faint_bg_color != panel_fill` a pairwise odlisne hodnoty mezi variantami | Overit, ze aplikovane `ctx.style().visuals.faint_bg_color` odpovida aktivni variante v runtime | Overit citelne oddeleni panelu (file tree/editor/side panel) bez agresivniho kontrastu | Panely splivaji, `faint_bg_color` zustane default nebo shodna hodnota napric variantami | **High** |
| `SETT-01` picker jen v light mode | Otestovat helper predicate (napr. `show_light_variant_picker(dark_theme)`) pro dark/light vetve | Render settings modalu: v dark mode picker neni mountnuty; v light mode je viditelny se 3 volbami | Prepnout dark/light v modalu a overit spravne show/hide pickeru | Picker viditelny v dark mode nebo chybi v light mode | **High** |
| `SETT-02` live preview bez restartu | Otestovat guard: zmena fingerprintu => 1x bump `settings_version`; bez zmeny => 0 bump | Simulovat zmenu varianty v jednom okne a overit okamzitou aplikaci v root + secondary viewportu | Pri kliknuti na variantu se vzhled meni okamzite; `Cancel` vraci puvodni snapshot | Preview az po Save/restartu, frame-by-frame bump storm, `Cancel` nerevertuje | **High** |
| `SETT-03` persistence | Roundtrip test `save()->load()` zachova `dark_theme/light_variant`; test chybejiciho pole `light_variant` pouzije default | Otestovat legacy migraci: existuje jen `settings.json` -> load zachova data -> nasledny save zapisuje canonical `settings.toml` | Zvolit variantu, `Save`, restart aplikace; po startu zustane stejna varianta i mode | Varianta se po restartu resetne, chyba parsovani, crash na legacy souboru, soubor se nevytvori | **High** |

### Failure Signal Monitoring (Nyquist gate)

- CI red signal: fail kterehokoliv unit/integration testu mapovaneho na `LITE-*`/`SETT-*`.
- Runtime red signal: `settings_version` roste i bez interakce uzivatele (indikace chybejiciho `.changed()`/fingerprint guard).
- UX red signal: `Cancel` nezrevertuje preview nebo secondary viewport zustane na jinem vzhledu nez root.
- Persistence red signal: po restartu je jina varianta nez ulozena, nebo load fallbackne na default kvuli parse chybe.

Confidence: **High**

## Sources

Primarni zdroje (oficialni docs/repo):
- egui `Context` API (`set_theme`, `set_visuals`, `set_visuals_of`, multi-viewport docs): https://github.com/emilk/egui/blob/main/crates/egui/src/context.rs
- egui `Visuals` defaulty (`light/dark`, `panel_fill`, `faint_bg_color`): https://github.com/emilk/egui/blob/main/crates/egui/src/style.rs
- egui theme model (`Theme`, `ThemePreference`): https://github.com/emilk/egui/blob/main/crates/egui/src/memory/theme.rs
- egui widgets helpery (`global_theme_preference_switch/buttons`): https://github.com/emilk/egui/blob/main/crates/egui/src/widgets/mod.rs
- eframe persistence (`Storage`, `get_value`, `set_value`, `APP_KEY`): https://github.com/emilk/egui/blob/main/crates/eframe/src/epi.rs
- eframe/egui multiple viewport example: https://github.com/emilk/egui/blob/main/examples/multiple_viewports/src/main.rs

Relevantni lokalni kontext:
- `src/settings.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/mod.rs`
- `src/highlighter.rs`
- `locales/*/ui.ftl`
