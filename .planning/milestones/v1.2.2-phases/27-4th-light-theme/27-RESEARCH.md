# Phase 27 Research: 4th Light Theme (Regression Fix Extension)

**Datum:** 2026-03-11
**Fokus:** minimální a bezpečný fix regrese „WarmTan není vidět v Settings pickeru, nelze na něj přepnout“.

## Co Je Potřeba Vědět Pro Kvalitní Plán

- Scope má zůstat striktně na THEME-01 až THEME-04 (viditelnost, přepnutí, persistence, lokalizace).
- V aktuálním workspace je `WarmTan` už přítomen v:
  - `LightVariant` enumu (`src/settings.rs`)
  - label/swatch mapování (`src/app/ui/workspace/modal_dialogs/settings.rs`)
  - picker iteraci (`src/app/ui/workspace/modal_dialogs/settings.rs`)
  - i18n klíčích ve všech 5 locale souborech.
- Přesto uživatel hlásí regresi. To znamená, že plán musí počítat i s nesouladem mezi "aktuální větví" a "builtem/runtime konfigurací".

## Pravděpodobné Příčiny Regrese

1. Běží jiný build/commit než ten, kde je `WarmTan` přidaný.
2. UI picker používá jiný render path (feature flag, jiný modal, jiná platformní větev).
3. Runtime stav je v dark režimu a light picker není aktivní (uživatel vnímá jako „není vidět“).
4. Stará konfigurace nebo serializovaná hodnota způsobí fallback, který skryje očekávané chování.
5. Chybí anti-regresní test přímo na počet light variant v settings pickeru.

## Standard Stack

- Rust + `eframe/egui` pro UI.
- `serde` + `toml` pro persistenci `Settings`.
- Stávající testy v `src/settings.rs` + UI modulové testy tam, kde už jsou zavedené.

## Architecture Patterns

- Jediný zdroj pravdy pro varianty témat má být v `LightVariant`.
- UI picker nesmí mít "skrytou" logiku mimo explicitně auditovatelný seznam variant.
- Theme preview se musí propsat přes `settings_version` bump bez restartu.
- Lokalizační klíč musí mít deterministické mapování `LightVariant -> i18n key`.

## Don't Hand-Roll

- Nevytvářet nový perzistenční formát ani vlastní parser konfigurace.
- Nepřidávat nový UI komponent framework ani refaktor modal systému.
- Nezavádět velký refaktor settings architektury kvůli jedné regresi.

## Common Pitfalls

- Oprava pouze v enumu bez opravy picker iterace.
- Oprava pouze v pickeru bez testu, který hlídá počet variant.
- Verifikace jen přes `cargo check` bez runtime ověření v Settings UI.
- Záměna „theme existuje v kódu“ za „theme je skutečně uživatelsky volitelné“. 

## Minimální Plan Extension (Doporučení)

1. **Reprodukce a izolace regrese**
- Potvrdit konkrétní prostředí (branch/commit, OS, jak byl build spuštěn).
- Ověřit, že v otevřeném Settings modalu je aktivní `Light` režim.
- Zachytit důkaz: screenshot / přesná observace počtu karet ve pickeru.

2. **Cílený patch pouze pro viditelnost a výběr**
- Pokud je v konkrétní větvi seznam variant neúplný, doplnit `WarmTan` do iterace pickeru.
- Pokud je problém v mapování, opravit `light_variant_label_key` a `light_variant_swatch`.
- Bez rozšiřování scope na nové varianty nebo redesign pickeru.

3. **Regresní guard testy (nutné pro stabilitu)**
- Přidat test, který ověří, že picker pracuje se 4 light variantami.
- Přidat test, že `WarmTan` má label key + swatch + barvy visuals.
- Udržet testy malé a přímo mapované na THEME-01..04.

4. **Verifikace po fixu**
- `cargo check`
- `./check.sh` (včetně `cargo fmt` gate)
- Manuální UI ověření: viditelnost, přepnutí, okamžitý preview efekt, persist po restartu.

## Validation Architecture

### Cíl validace
Potvrdit, že regresní fix vrací uživatelsky pozorovatelné chování pro THEME-01, THEME-02, THEME-03, THEME-04 bez vedlejších regresí.

### Validation layers

1. **Statická validace (build gate)**
- `cargo check` musí projít bez errorů.
- `./check.sh` musí projít (včetně formátování); pokud selže na preexistujících změnách, explicitně oddělit, co je mimo scope fáze 27.

2. **Strukturální validace (code-level assertions)**
- Ověřit přítomnost `LightVariant::WarmTan` v enumu.
- Ověřit mapování:
  - `LightVariant::WarmTan -> settings-light-variant-warm-tan`
  - `LightVariant::WarmTan -> swatch RGB(215,200,185)`
  - `LightVariant::WarmTan -> visuals panel/window/faint` dle rozhodnutí ve `27-CONTEXT.md`.
- Ověřit i18n klíč `settings-light-variant-warm-tan` v `cs/en/de/ru/sk`.

3. **Behaviorální validace (UI/runtime)**
- V Settings > Editor > Light variant jsou vidět 4 karty včetně WarmTan.
- Klik na WarmTan okamžitě změní preview tématu.
- Uložení settings a restart zachová `light_variant = warm_tan`.

4. **Anti-regression validace**
- Přidat/udržet test, který failne, pokud počet light variant v pickeru klesne pod 4.
- Přidat/udržet test pro roundtrip persistence `WarmTan` v `settings.toml`.

### Evidence artifacts

- Test output z `cargo check` a `./check.sh`.
- Krátký checklist výsledků k THEME-01..04.
- Odkazy na změněné soubory (jen minimální patch).

### Exit criteria

- Všechny 4 požadavky THEME-01..04 jsou znovu prokazatelně splněny.
- Reprodukce původní regrese po patchi selže (tj. problém už nelze vyvolat).
- Žádný rozšířený scope mimo regresní fix.

## Dopad Na Plánování

- Největší riziko není implementace barvy, ale drift mezi deklarovaným stavem fáze a reálným buildem.
- Plan má explicitně obsahovat krok "verify runtime branch/build provenance" před editací kódu.
- Fix má být veden jako „minimal correction + regression guard“, ne jako redesign témat.

## Otevřené Body Před Final Planem

- V jakém konkrétním buildu/commitu byla regrese pozorována?
- Je reprodukce konzistentní i po čistém buildu?
- Má regresi jen Settings picker, nebo i persistence/swatch/preview?

