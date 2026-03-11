---
phase: 29-syntect-theme-mapping
plan: 29-01+29-02
status: passed
verified_at: 2026-03-11
verifier: codex
requirements_checked:
  - SYNTAX-01
  - SYNTAX-02
---

## VERIFICATION PASSED

Automaticky ověřitelné části cíle fáze 29 jsou splněné včetně dříve reportovaného gapu „terminálové pozadí je bílé/svítí“. Zbývá ruční vizuální UAT pro potvrzení formulace „appropriate theme per color variant“.

## Requirement ID Coverage (PLAN -> REQUIREMENTS)

| Requirement ID | Zdroj v PLAN frontmatter | V REQUIREMENTS.md | Stav |
|---|---|---|---|
| SYNTAX-01 | `29-01-PLAN.md:10-12` | `REQUIREMENTS.md:22` | accounted |
| SYNTAX-02 | `29-01-PLAN.md:10-12`, `29-02-PLAN.md:12-13` | `REQUIREMENTS.md:23` | accounted |

Každé ID z frontmatter plánů 29-01 a 29-02 je dohledané v `.planning/REQUIREMENTS.md`.

## Must-Haves Audit

| Must-have | Důkaz v kódu / testech | Výsledek |
|---|---|---|
| 4 light varianty mapují na odlišná syntect témata | Explicitní matice v `mapped_syntect_theme_name()` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):293-305), unikátnost hlídaná testem `syntax01_light_mapping_unique` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):532-555) | splněno |
| 2 dark varianty mapují na odlišná dark témata | `Default -> Solarized (dark)`, `Midnight -> base16-ocean.dark` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):295-298), kryto `syntax02_dark_variants_are_distinct` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):582-598) | splněno |
| Missing theme používá bezpečný fallback + warning | `resolve_syntect_theme_name_or_fallback()` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):277-285), kryto `syntect_theme_fallback_contract` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):558-562) | splněno |
| Mapování je deterministické a centralizované v Settings | `syntect_theme_name()` ([src/settings.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/settings.rs):347-349), editor render jen čte theme (`theme_name = settings.syntect_theme_name()`) ([ui.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/editor/ui.rs):128), highlighter napojen při apply settings ([mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/mod.rs):738-740) | splněno |
| Terminal background je navázaný na aktivní variantu a dark není světlý/oslňující | Dark paleta vychází z `visuals.panel_fill` přes `tone_dark_palette` a míchání do tmava ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):118-126), vstup přes `terminal_palette(visuals)` ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):158-163) | splněno (automaticky) |
| Background mapping je centralizovaný v terminal theme vrstvě bez ad-hoc UI přepisů | Theme se nastavuje přes `terminal_theme_for_visuals_with_focus(ui.visuals(), focused)` ([mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/mod.rs):214), rozhodovací logika v `theme.rs` | splněno |
| Existuje regresní gate pro kontrast a variant mapping terminal backgroundu | Testy `terminal_theme_dark_background_stays_dark`, `terminal_theme_dark_foreground_has_readable_contrast_for_all_dark_variants`, `terminal_theme_light_variant_backgrounds_are_distinct_across_all_three` ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):283-378) | splněno |

## Execution Evidence (2026-03-11)

Spuštěno:
- `cargo test -q syntax01_light_mapping_matrix_complete` -> pass
- `cargo test -q syntax01_light_mapping_unique` -> pass
- `cargo test -q syntax02_dark_variants_are_distinct` -> pass
- `cargo test -q syntect_theme_fallback_contract` -> pass
- `cargo test -q terminal_theme_dark_background_stays_dark` -> pass
- `cargo test -q terminal_theme_dark_foreground_has_readable_contrast_for_all_dark_variants` -> pass
- `cargo test -q terminal_theme_light_variant_backgrounds_are_distinct_across_all_three` -> pass
- `cargo check` -> pass (1 non-blocking warning na unused variable)
- `./check.sh` -> fail pouze na `cargo fmt` drift v jiných souborech (`src/app/ui/git_status.rs`, `src/app/ui/workspace/modal_dialogs/settings.rs`), mimo scope SYNTAX-01/SYNTAX-02

## Gap Re-Check: Terminal Background Mismatch

Dříve reportovaný gap („terminálové pozadí je bílé a svítí“) je po 29-02 pokryt implementačně i testy:
- dark background má explicitní limit přes luminanci `< 0.2` v testu ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):283-294),
- dark varianty mají odlišné background tóny ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):296-307),
- foreground/background kontrast v dark režimu je guardovaný (`>= 4.5`) ([theme.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/terminal/instance/theme.rs):310-321).

## Human Sign-Off

- 2026-03-11: uživatel potvrdil manuální vizuální UAT jako `approved`.
- Subjektivní část wordingu „appropriate theme per color variant“ je tímto uzavřená.
