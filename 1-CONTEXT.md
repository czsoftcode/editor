# Context: Phase 1 — Základ (Theme & Syntax Highlighting)

## Goal
Implementace robustního systému témat pro PolyCredo Editor, který umožní plynulé přepínání mezi Dark mode a třemi variantami Light mode (Teplá slonová kost, Studená šedá, Sépiová). Cílem je zajistit čitelný syntax highlighting a UI bez hardcoded barev a vizuálních artefaktů (flash při startu).

## Decisions

### 1. Model Témat a Barvy
- **Modul `src/app/ui/theme.rs`**: Vytvoříme dedikovaný modul pro definici témat.
- **Struktura `AppColors`**: Zavedeme sémantické barvy (např. `git_added`, `terminal_bg`, `status_ok`, `editor_bg`, `sidebar_bg`) pro každé téma.
- **Plná definice `egui::Visuals`**: Pro každou variantu definujeme kompletní sadu vizuálů egui, nikoliv jen parciální přepis.
- **Varianty světlého režimu**:
  - **Teplá slonová kost (Default Light)**: RGB(255, 252, 240)
  - **Studená šedá**: RGB(242, 242, 242)
  - **Sépiová**: RGB(240, 230, 210)
- **Sémantické rozlišení**: Editor a postranní panel budou mít mírně odlišné pozadí (podle varianty) pro lepší vizuální orientaci.

### 2. Syntax Highlighting
- **Stateful Highlighter**: `Highlighter` v `src/highlighter.rs` bude mít vnitřní stav pro aktuální téma.
- **Rozšířený Cache Key**: Klíč pro cache v highlighteru bude obsahovat i název aktivního tématu (umožní okamžitý návrat k předchozímu tématu).
- **Tailored Syntect Themes**:
  - Dark mode: `base16-ocean.dark` (stávající)
  - Ivory / Sepia variants: `Solarized (light)`
  - Cool Gray variant: `InspiredGitHub`
- **Aktualizace**: Překreslení po změně tématu zajistí existující mechanismus `settings_version`.

### 3. UI a Refaktoring Widgetů
- **Odstranění hardcoded barev**: Provedeme totální refaktoring všech widgetů v `src/app/ui/widgets/`. Budou používat barvy výhradně z `ui.visuals()` nebo nové struktury `AppColors`.
- **Terminál (ANSI mapa)**: Definujeme samostatné ANSI barevné mapy pro Dark a Light režimy pro zajištění maximální čitelnosti.
- **Git status barvy**: Specifické sady barev pro M/A/??/D stavy pro každou variantu tématu (např. tmavě zelená pro "Added" na světlém pozadí).

### 4. Inicializace a Perzistence
- **Flash Avoidance**: Téma se aplikuje v `eframe::App::new(cc)` před prvním vykreslením frame, aby nedošlo k tmavému záblesku při startu v light mode.
- **Settings Structure**: V `settings.toml` vznikne nová sekce `[theme]` obsahující `mode` (dark/light) a `light_variant`.
- **Graceful Migration**: Pokud v `settings.toml` chybí pole `light_variant`, aplikace automaticky použije "Warm Ivory" bez upozornění uživatele.

## Code Context

### Reusable Patterns
- Použití `ctx.set_visuals()` pro globální změnu témat.
- Využití `settings_version` pro invalidaci cache a překreslení.
- Mapování ANSI barev v `egui_term` (bude nutno napojit na `AppColors`).

### Integration Points
- `src/settings.rs`: Rozšíření struktury `Settings` o sekci `[theme]`.
- `src/highlighter.rs`: Úprava `Highlighter::highlight` pro zohlednění témat.
- `src/app/ui/theme.rs`: Nový modul pro definice barev.
- `src/main.rs`: Inicializace tématu v `App::new()`.

## Deferred Ideas
- Automatická detekce systémového dark/light režimu (odloženo do v2).
- Vlastní uživatelský theme editor (odloženo, příliš komplexní).
- Animované přechody mezi tématy (egui nepodporuje nativně).
