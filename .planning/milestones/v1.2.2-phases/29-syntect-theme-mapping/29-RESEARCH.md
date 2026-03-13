# Phase 29 Research: Syntect Theme Mapping

Datum: 2026-03-10  
Scope: mapování syntect témat pro 4 light varianty + 2 dark varianty, fallback bezpečnost, test gate pro SYNTAX-01 a SYNTAX-02.

## Co Je Nutné Vědět Pro Kvalitní Plán

- Primární integrační bod je `src/settings.rs::syntect_theme_name()`; změna má být izolovaná sem, bez refaktoru rendereru.
- Runtime path už je správně zapojený: `settings.syntect_theme_name()` se používá v editor renderu i při apply settings (`src/app/mod.rs`, `src/app/ui/editor/ui.rs`).
- `src/highlighter.rs` už má runtime fallback (`unwrap_or_else` na `base16-ocean.dark`), ale Phase 29 explicitně vyžaduje i ověřitelný fallback kontrakt na úrovni mapování a testů.
- Kontext fáze vyžaduje unikátní mapování všech 6 variant, nikoliv jen dark/light split.
- Největší riziko je tiché rozbití při překlepu názvu tématu; plán musí obsahovat validaci against `ThemeSet::load_defaults().themes`.

## Standard Stack

Preskriptivně použij:

- `syntect::highlighting::ThemeSet::load_defaults()` jako jediný zdroj truth pro dostupná built-in témata.
- `Settings` (`src/settings.rs`) jako jediný zdroj truth pro mapování variant -> theme name.
- Unit testy v `src/settings.rs` pro mapování, unikátnost a fallback kontrakt.
- Bez nových závislostí, bez custom tmTheme souborů.

Built-in témata, která jsou v syntect default dumpu a dávají smysl pro tuto fázi:

- Light kandidáti: `Solarized (light)`, `InspiredGitHub`, `base16-ocean.light`
- Dark kandidáti: `base16-ocean.dark`, `Solarized (dark)`, `base16-eighties.dark`, `base16-mocha.dark`

Doporučená cílová mapa pro plán (splní unikátnost + charakter z contextu):

- `LightVariant::WarmIvory` -> `Solarized (light)` (baseline neutrálně teplý)
- `LightVariant::CoolGray` -> `InspiredGitHub` (chladnější/čistý light)
- `LightVariant::Sepia` -> `base16-ocean.light` (tlumený light odlišný od WarmIvory)
- `LightVariant::WarmTan` -> `Solarized (light)` **není povoleno** (duplicitní) -> použij odlišný light kandidát po ověření; pokud zůstane jen 3 light built-in, plán musí explicitně definovat akceptovanou výjimku nebo změnu požadavku
- `DarkVariant::Default` -> `base16-ocean.dark`
- `DarkVariant::Midnight` -> `base16-mocha.dark` (chladnější/hlubší dark charakter)

Poznámka pro plán: pokud produkt trvá na striktní unikátnosti všech 4 light variant a současně pouze built-in tématech, je třeba v plánovací fázi potvrdit, že dostupná light sada skutečně obsahuje 4 vhodné a odlišné položky. Jinak je to rozhodovací checkpoint.

## Architecture Patterns

### Pattern A: Deterministické mapování v jedné funkci

- `syntect_theme_name()` implementuj jako úplný `match` přes `(dark_mode, dark_variant, light_variant)`.
- Žádné heuristiky typu `if dark else light`; musí být explicitní větev pro každou variantu.
- Přidej malý interní helper `resolved_syntect_theme_name(theme_set)` nebo `validate_syntect_theme_name(name, theme_set)` pro fallback logiku.

### Pattern B: Defenzivní fallback s viditelným signálem

- Když mapované téma není dostupné v `ThemeSet::load_defaults().themes`, vrať `base16-ocean.dark`.
- Současně proveď warning log (`eprintln!` nebo existující log mechanismus), aby nešlo o tiché selhání.
- Fallback path musí být testovatelný bez GUI.

### Pattern C: Test gate u zdroje pravdy

- Testy mapování drž v `src/settings.rs` (blízko funkce), ne v UI vrstvě.
- Přidej test „coverage matrix“ pro všech 6 variant.
- Přidej test „uniqueness gate“ a test „fallback gate“.

## Don't Hand-Roll

- Nezaváděj vlastní loader tmTheme souborů.
- Nezaváděj runtime probing z UI threadu při každém renderu.
- Neduplikuj mapování v `highlighter.rs` ani v UI; mapování má být jen v `Settings`.
- Nerefactoruj cache/logiku highlightingu; scope fáze je mapování + validace.

## Common Pitfalls

1. Duplicitní mapování light variant poruší kontrakt SYNTAX-01.
Mitigace: unit test s `HashSet<&'static str>` pro všech 4 light variant.

2. Překlep v názvu tématu se projeví jen fallbackem v `highlighter.rs` a problém je skrytý.
Mitigace: explicitní validace theme name proti `ThemeSet` + warning + unit test.

3. Chybné určení dark mode (`dark_theme` vs `dark_variant`) rozbije dark mapping.
Mitigace: helper `is_dark_mode()` testovaný zvlášť pro kombinace flag + variant.

4. Přidání testů bez jasné requirement traceability.
Mitigace: názvy testů prefixuj `syntax01_`/`syntax02_` a mapuj je na requirement ID.

## Code Examples

### Example 1: Preskriptivní mapování v `Settings`

```rust
impl Settings {
    fn is_dark_mode(&self) -> bool {
        self.dark_theme || self.dark_variant != DarkVariant::Default
    }

    pub fn syntect_theme_name(&self) -> &'static str {
        if self.is_dark_mode() {
            match self.dark_variant {
                DarkVariant::Default => "base16-ocean.dark",
                DarkVariant::Midnight => "base16-mocha.dark",
            }
        } else {
            match self.light_variant {
                LightVariant::WarmIvory => "Solarized (light)",
                LightVariant::CoolGray => "InspiredGitHub",
                LightVariant::Sepia => "base16-ocean.light",
                LightVariant::WarmTan => "Solarized (light)", // placeholder: v plánu nahradit unikátním kandidátem
            }
        }
    }
}
```

### Example 2: Validace mapovaného názvu + fallback

```rust
use syntect::highlighting::ThemeSet;

fn resolve_syntect_theme_or_fallback(mapped: &'static str) -> &'static str {
    let themes = ThemeSet::load_defaults();
    if themes.themes.contains_key(mapped) {
        mapped
    } else {
        eprintln!("warning: missing syntect theme '{mapped}', using fallback base16-ocean.dark");
        "base16-ocean.dark"
    }
}
```

### Example 3: Unit gate pro unikátnost light variant

```rust
#[test]
fn syntax01_light_variants_use_unique_theme_mapping() {
    use std::collections::HashSet;

    let mut names = HashSet::new();
    for v in [
        LightVariant::WarmIvory,
        LightVariant::CoolGray,
        LightVariant::Sepia,
        LightVariant::WarmTan,
    ] {
        let s = Settings {
            dark_theme: false,
            light_variant: v,
            ..Default::default()
        };
        assert!(names.insert(s.syntect_theme_name()));
    }
}
```

## Validation Architecture

### Nyquist Validation Strategy

- Fáze je complete pouze když jsou splněny `SYNTAX-01` a `SYNTAX-02` ve 3 vrstvách: unit, integration-smoke, manual UAT.
- Unit vrstva je povinná gate v CI (`cargo test` část pro `settings.rs`).
- Integration-smoke vrstva ověří, že mapování teče do runtime render path (`settings -> editor ui -> highlighter`) bez paniky.
- Manual UAT vrstva ověří vizuální charakter variant podle context decision (WarmIvory baseline, WarmTan teplejší, Midnight chladnější dark).

### Validation Layers

- Unit (`src/settings.rs`):
- `syntax01_light_mapping_matrix_complete` (4/4 light variant explicitně mapované)
- `syntax01_light_mapping_unique` (žádné duplicity)
- `syntax01_fallback_for_missing_theme_is_safe` (invalid name -> fallback + bez paniky)
- `syntax02_dark_mapping_matrix_complete` (Default + Midnight)
- `syntax02_dark_variants_are_distinct` (doporučeno: unikátní dark theme names)

- Integration smoke (existující flow):
- Inicializace workspace + volání `settings.syntect_theme_name()` v `src/app/ui/editor/ui.rs` proběhne bez erroru pro všech 6 variant.
- `highlighter.set_theme(theme_name)` + `highlight(..., theme_name)` nesmí panicnout ani při fallback scénáři.

- Manual UAT:
- Přepnout všech 6 variant v Settings UI, otevřít stejný zdrojový soubor, potvrdit konzistentní změnu syntax barev.
- Ověřit, že `Midnight` je vizuálně chladnější než `Default` při zachování čitelnosti.
- Ověřit, že light varianty nejsou vizuálně zaměnitelné.

### Requirement Matrix

| Requirement | Unit Gate | Integration Smoke | UAT | Exit Criteria |
|---|---|---|---|---|
| `SYNTAX-01` | úplnost + unikátnost mapování light variant + fallback test | flow settings->highlighter funguje pro všechny light varianty | vizuálně odlišné 4 light varianty | všechny 3 vrstvy green |
| `SYNTAX-02` | úplnost mapování dark variant + (doporučeno) distinct test | flow settings->highlighter funguje pro `Default` i `Midnight` | `Default` vs `Midnight` je konzistentně odlišné | všechny 3 vrstvy green |

## Doporučené Plánovací Rozdělení (minimální patch)

1. `src/settings.rs`: přepsat `syntect_theme_name()` na explicitní mapovací matici.
2. `src/settings.rs`: doplnit validovaný resolver/fallback helper (s warning logem).
3. `src/settings.rs` testy: přidat gate testy pro SYNTAX-01/02.
4. Verifikace: `cargo check`, `./check.sh`, cílené `cargo test settings::tests::syntax`.

## Otevřený Checkpoint Pro PLAN

- Potvrdit produktové rozhodnutí pro `LightVariant::WarmTan`, pokud má být mapování 4/4 unikátní a zároveň „pouze built-in“: je třeba schválit konkrétní 4. light téma dostupné v `ThemeSet::load_defaults()` (nebo explicitně povolit controlled výjimku z unikátnosti).
