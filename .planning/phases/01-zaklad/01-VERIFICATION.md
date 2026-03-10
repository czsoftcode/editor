---
phase: 01-zaklad
verified: "2026-03-04T21:31:45Z"
status: passed
score: 32/32 checks verified (12 requirements + 20 must-have/key-link checks)
---

# Phase 01: zaklad — Verification

## Observable Truths
| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Po gap-closure `01-03` a `01-04` je Phase 1 v roadmap označená jako completed a obsahuje všechny 4 plány | passed | `.planning/ROADMAP.md:9`, `.planning/ROADMAP.md:26`, `.planning/ROADMAP.md:27`, `.planning/ROADMAP.md:28`, `.planning/ROADMAP.md:29` |
| 2 | Frontmatter requirements z `01-01..01-04` jsou účetně pokryté v `REQUIREMENTS.md` i `SUMMARY` frontmatterech | passed | `.planning/phases/01-zaklad/01-01-PLAN.md:10`, `.planning/phases/01-zaklad/01-02-PLAN.md:11`, `.planning/phases/01-zaklad/01-03-PLAN.md:11`, `.planning/phases/01-zaklad/01-04-PLAN.md:11`, `.planning/phases/01-zaklad/01-01-SUMMARY.md:38`, `.planning/phases/01-zaklad/01-02-SUMMARY.md:28`, `.planning/phases/01-zaklad/01-03-SUMMARY.md:24`, `.planning/phases/01-zaklad/01-04-SUMMARY.md:28` |
| 3 | Settings model drží theme mapování i zpětnou kompatibilitu (`LightVariant`, `#[serde(default)]`, `syntect_theme_name`, `to_egui_visuals`, delegace v `apply`) | passed | `src/settings.rs:76`, `src/settings.rs:101`, `src/settings.rs:198`, `src/settings.rs:208`, `src/settings.rs:259`, `src/settings.rs:342` |
| 4 | Startup apply tématu běží před prvním frame a runtime propagace je navázaná na `settings_version` + `highlighter.set_theme(...)` | passed | `src/app/mod.rs:119`, `src/app/mod.rs:556`, `src/app/mod.rs:560`, `src/app/mod.rs:561`, `src/app/mod.rs:716`, `src/app/mod.rs:721`, `src/app/mod.rs:723` |
| 5 | Highlighter je theme-aware end-to-end (`theme_name` v signatuře, lookup/fallback, editor předává `settings.syntect_theme_name()`) | passed | `src/highlighter.rs:49`, `src/highlighter.rs:97`, `src/highlighter.rs:139`, `src/app/ui/editor/ui.rs:96`, `src/app/ui/editor/render/normal.rs:109`, `src/app/ui/editor/render/markdown.rs:115` |
| 6 | Gap-closure `01-03`: floating terminal frame už není hardcoded tmavý a používá runtime visuals | passed | `src/app/ui/terminal/window.rs:37`, `src/app/ui/terminal/window.rs:52` |
| 7 | Gap-closure `01-04`: status bar text i stavové barvy jsou theme-aware a branchují podle `visuals.dark_mode` | passed | `src/app/ui/editor/ui.rs:214`, `src/app/ui/editor/ui.rs:216`, `src/app/ui/editor/ui.rs:219`, `src/app/ui/editor/ui.rs:228`, `src/app/ui/editor/ui.rs:339`, `src/app/ui/editor/ui.rs:353` |
| 8 | UI-02 (indikátor neuloženého stavu `●`) zůstává v labelu tabu bez hardcoded světlé barvy, tedy se renderuje přes aktivní theme text | passed | `src/app/ui/editor/render/tabs.rs:26`, `src/app/ui/editor/render/tabs.rs:62` |

## Goal Criteria Check (Goal-Backward)
| Roadmap Success Criterion | Status | Evidence |
|---------------------------|--------|----------|
| 1. Přepnutí na light mode mění UI panely/dialogy bez tmavých artefaktů | passed | `.planning/ROADMAP.md:20`, `src/settings.rs:259`, `src/app/ui/terminal/window.rs:37`, `src/app/ui/editor/ui.rs:214` |
| 2. Syntax highlighting v light mode je čitelný (`Solarized (light)`) | passed | `.planning/ROADMAP.md:21`, `src/settings.rs:202`, `src/app/ui/editor/ui.rs:96`, `src/highlighter.rs:97` |
| 3. Při startu s uloženým light mode není dark flash (apply v `new()`) | passed | `.planning/ROADMAP.md:22`, `src/app/mod.rs:88`, `src/app/mod.rs:119` |
| 4. Starý `settings.json` bez `light_variant` nepadá | passed | `.planning/ROADMAP.md:23`, `src/settings.rs:101`, `src/settings.rs:226`, `src/settings.rs:342` |
| 5. Tabs + status bar čitelné v obou módech | passed | `.planning/ROADMAP.md:24`, `src/app/ui/editor/render/tabs.rs:26`, `src/app/ui/editor/ui.rs:216`, `src/app/ui/editor/ui.rs:219` |

## Requirement ID Cross-Reference
| Requirement | PLAN Frontmatter | REQUIREMENTS.md | SUMMARY Frontmatter | Status |
|-------------|------------------|-----------------|---------------------|--------|
| THEME-01 | `01-01-PLAN.md:10` | `.planning/REQUIREMENTS.md:10`, `.planning/REQUIREMENTS.md:76` | `01-01-SUMMARY.md:38` | passed |
| THEME-02 | `01-01-PLAN.md:10` | `.planning/REQUIREMENTS.md:11`, `.planning/REQUIREMENTS.md:77` | `01-01-SUMMARY.md:38` | passed |
| THEME-03 | `01-02-PLAN.md:11` | `.planning/REQUIREMENTS.md:12`, `.planning/REQUIREMENTS.md:78` | `01-02-SUMMARY.md:28` | passed |
| THEME-04 | `01-01-PLAN.md:10` | `.planning/REQUIREMENTS.md:13`, `.planning/REQUIREMENTS.md:79` | `01-01-SUMMARY.md:38` | passed |
| EDIT-01 | `01-02-PLAN.md:11` | `.planning/REQUIREMENTS.md:17`, `.planning/REQUIREMENTS.md:80` | `01-02-SUMMARY.md:28` | passed |
| EDIT-02 | `01-02-PLAN.md:11` | `.planning/REQUIREMENTS.md:18`, `.planning/REQUIREMENTS.md:81` | `01-02-SUMMARY.md:28` | passed |
| EDIT-03 | `01-02-PLAN.md:11` | `.planning/REQUIREMENTS.md:19`, `.planning/REQUIREMENTS.md:82` | `01-02-SUMMARY.md:28` | passed |
| EDIT-04 | `01-02-PLAN.md:11` | `.planning/REQUIREMENTS.md:20`, `.planning/REQUIREMENTS.md:83` | `01-02-SUMMARY.md:28` | passed |
| SETT-04 | `01-01-PLAN.md:10` | `.planning/REQUIREMENTS.md:46`, `.planning/REQUIREMENTS.md:84` | `01-01-SUMMARY.md:38` | passed |
| UI-01 | `01-01-PLAN.md:10`, `01-03-PLAN.md:11` | `.planning/REQUIREMENTS.md:50`, `.planning/REQUIREMENTS.md:85` | `01-01-SUMMARY.md:38`, `01-03-SUMMARY.md:24` | passed |
| UI-02 | `01-01-PLAN.md:10` | `.planning/REQUIREMENTS.md:51`, `.planning/REQUIREMENTS.md:86` | `01-01-SUMMARY.md:38` | passed |
| UI-03 | `01-01-PLAN.md:10`, `01-04-PLAN.md:11` | `.planning/REQUIREMENTS.md:52`, `.planning/REQUIREMENTS.md:87` | `01-01-SUMMARY.md:38`, `01-04-SUMMARY.md:28` | passed |

## Must-Haves & Key Links Verification
| Check | Status | Evidence |
|-------|--------|----------|
| 01-01 truth: `Settings.light_variant` má `#[serde(default)]` a default `WarmIvory` | passed | `src/settings.rs:101`, `src/settings.rs:148`, `src/settings.rs:285`, `src/settings.rs:349` |
| 01-01 truth: `syntect_theme_name()` mapuje dark/light správně | passed | `src/settings.rs:198`, `src/settings.rs:200`, `src/settings.rs:202` |
| 01-01 truth: `to_egui_visuals()` vrací dark/light visuals | passed | `src/settings.rs:208`, `src/settings.rs:210`, `src/settings.rs:212` |
| 01-01 truth: `Settings::apply()` deleguje visuals přes metodu | passed | `src/settings.rs:257`, `src/settings.rs:259` |
| 01-01 key-link: `Settings::apply()` -> `ctx.set_visuals()` via `to_egui_visuals()` | passed | `src/settings.rs:259` |
| 01-01 key-link: `LightVariant` -> serde default wiring | passed | `src/settings.rs:76`, `src/settings.rs:101` |
| 01-02 truth: `Highlighter::highlight()` přijímá `theme_name` | passed | `src/highlighter.rs:43`, `src/highlighter.rs:49` |
| 01-02 truth: light používá `Solarized (light)`, dark `base16-ocean.dark` | passed | `src/settings.rs:200`, `src/settings.rs:202`, `src/app/ui/editor/ui.rs:96`, `src/highlighter.rs:97` |
| 01-02 truth: cache invalidace jen při změně tématu | passed | `src/highlighter.rs:37`, `src/highlighter.rs:39`, `src/app/mod.rs:718`, `src/app/mod.rs:721` |
| 01-02 truth: startup apply je v `EditorApp::new()` | passed | `src/app/mod.rs:88`, `src/app/mod.rs:119` |
| 01-02 key-link: `EditorApp::new()` -> `Settings::apply()` | passed | `src/app/mod.rs:117`, `src/app/mod.rs:119` |
| 01-02 key-link: `Highlighter::highlight()` <- `Settings::syntect_theme_name()` přes `theme_name` | passed | `src/app/ui/editor/ui.rs:96`, `src/app/ui/editor/render/normal.rs:104`, `src/app/ui/editor/render/normal.rs:109`, `src/highlighter.rs:49` |
| 01-03 truth: floating terminal frame nepoužívá hardcoded tmavou fill | passed | `src/app/ui/terminal/window.rs:37`, `src/app/ui/terminal/window.rs:52` |
| 01-03 truth: runtime dark/light switch mění frame fill bez restartu (fill čten z aktuálních visuals při renderu) | passed | `src/app/ui/terminal/window.rs:37`, `src/app/ui/terminal/bottom/mod.rs:34`, `src/app/ui/terminal/ai_chat/mod.rs:46` |
| 01-03 truth: build + AI floating terminál sdílí `StandardTerminalWindow` | passed | `src/app/ui/terminal/bottom/mod.rs:28`, `src/app/ui/terminal/ai_chat/mod.rs:40`, `src/app/ui/terminal/right/mod.rs:43` |
| 01-03 key-link: `StandardTerminalWindow::show()` -> `Frame::window(...).fill(...)` | passed | `src/app/ui/terminal/window.rs:24`, `src/app/ui/terminal/window.rs:52` |
| 01-04 truth: status bar primární/sekundární text barvy jsou odvozené z `ui.visuals()` | passed | `src/app/ui/editor/ui.rs:214`, `src/app/ui/editor/ui.rs:216`, `src/app/ui/editor/ui.rs:217` |
| 01-04 truth: diagnostiky + save/LSP stavy mají kontrastní dark/light branch | passed | `src/app/ui/editor/ui.rs:219`, `src/app/ui/editor/ui.rs:228`, `src/app/ui/editor/ui.rs:339`, `src/app/ui/editor/ui.rs:353` |
| 01-04 truth: runtime dark/light switch aktualizuje status bar bez restartu | passed | `src/app/ui/workspace/mod.rs:297`, `src/app/ui/workspace/mod.rs:302`, `src/app/ui/editor/ui.rs:214` |
| 01-04 key-link: `Editor::status_bar()` -> `egui::Visuals` | passed | `src/app/ui/editor/ui.rs:203`, `src/app/ui/editor/ui.rs:214` |

## Required Artifacts
| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/settings.rs` | LightVariant + theme methods + apply delegace | passed | `LightVariant`, `syntect_theme_name()`, `to_egui_visuals()`, `apply()` delegace a testy zpětné kompatibility jsou přítomné |
| `src/highlighter.rs` | `theme_name` parametrizace + `set_theme` invalidace | passed | `highlight(...)`, `background_color(...)` i `set_theme(...)` odpovídají must-haves |
| `src/app/mod.rs` | startup apply + settings_version propagace | passed | `settings.apply(&cc.egui_ctx)` v `new()` + `highlighter.set_theme(...)` při změně verze |
| `src/app/ui/terminal/window.rs` | theme-aware floating frame fill | passed | `.frame(egui::Frame::window(&ctx.style()).fill(viewer_bg))` s `viewer_bg` z `ctx.style().visuals.panel_fill` |
| `src/app/ui/editor/ui.rs` | theme-aware status bar contrast | passed | `ui.visuals()` palette + dark/light branch pro semantické stavy |
| `.planning/phases/01-zaklad/01-01-SUMMARY.md` | Summary plánu 01 | passed | Soubor existuje a obsahuje `requirements-completed` |
| `.planning/phases/01-zaklad/01-02-SUMMARY.md` | Summary plánu 02 | passed | Soubor existuje a obsahuje `requirements-completed` |
| `.planning/phases/01-zaklad/01-03-SUMMARY.md` | Summary gap-closure plánu 03 | passed | Soubor existuje a obsahuje `requirements-completed: [UI-01]` |
| `.planning/phases/01-zaklad/01-04-SUMMARY.md` | Summary gap-closure plánu 04 | passed | Soubor existuje a obsahuje `requirements-completed: [UI-03]` |

## Requirements Coverage
| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| THEME-01 | passed | - |
| THEME-02 | passed | - |
| THEME-03 | passed | - |
| THEME-04 | passed | - |
| EDIT-01 | passed | - |
| EDIT-02 | passed | - |
| EDIT-03 | passed | - |
| EDIT-04 | passed | - |
| SETT-04 | passed | - |
| UI-01 | passed | - |
| UI-02 | passed | - |
| UI-03 | passed | - |

## Result
Goal-backward verifikace fáze `01-zaklad` po gap-closure plánech `01-03` a `01-04` je **passed**: požadavky z PLAN frontmatterů jsou účetně pokryté (`REQUIREMENTS.md` + `SUMMARY`), must-haves i key-links z `01-01..01-04` jsou ověřené proti aktuálnímu kódu a nebyla nalezena mezera vyžadující `gaps_found`.

Poznámka: verifikace je statická (kód + plánovací artefakty). Manuální GUI smoke pro subjektivní kontrast lze doplnit v UAT, ale v této kontrole neblokuje status.
