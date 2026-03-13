# Phase 2: Terminal + Git barvy — Research

**Researched:** 2026-03-04
**Domain:** egui_term theme integration + file tree git color readability (Rust + eframe/egui)
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TERM-01 | Claude panel (egui_term) má světlé pozadí v light mode | Claude panel i Build terminál používají stejný `Terminal::ui(...)` wrapper (`src/app/ui/terminal/instance/mod.rs`) a `egui_term::TerminalView`, takže řešení je společné |
| TERM-02 | Build terminál má světlé pozadí v light mode | Build panel také renderuje stejný `Terminal` typ (`src/app/ui/terminal/bottom/mod.rs`), takže změna v instance vrstvě pokryje oba terminály |
| TERM-03 | Scrollbar terminálu není hardcoded tmavý | `draw_scrollbar` v `src/app/ui/terminal/instance/render.rs` má hardcoded tmavé barvy (`0x18`, grayscale) — je třeba napojit na aktivní téma |
| TERM-04 | Terminálový text je čitelný na světlém pozadí | `vendor/egui_term` podporuje `TerminalTheme` a `TerminalView::set_theme(...)`; aktuálně se nepoužívá, takže běží default dark palette |
| TREE-01 | Git barvy jsou čitelné na světlém pozadí | Dnes se aplikuje jediná sada barev + násobení `0.55` pro light (`src/app/ui/file_tree/render.rs`), což je heuristika bez explicitní light palety |
| TREE-02 | Untracked (`??`) je viditelný v light mode | `parse_git_status` vrací barvu pro `??`, ale render ji dál ztmavuje společným faktorem; explicitní light paleta zlepší stabilitu čitelnosti |
</phase_requirements>

---

## Summary

Fáze 2 lze implementovat bez nových závislostí a bez restartování terminálových procesů. Klíčové je využít už dostupné API ve vendored `egui_term`: `TerminalTheme` + `TerminalView::set_theme(...)`. To umožní light-safe paletu aplikovat čistě na render vrstvě, zatímco PTY proces běží dál.

Aktuálně je problém ve dvou bodech:
- `Terminal::ui(...)` nikdy nevolá `set_theme`, takže se používá default dark palette (`#181818` background).
- Scrollbar v terminálu je kreslen hardcoded tmavými barvami.

File tree část je technicky hotová jen napůl: git status barvy existují, ale light mode používá globální ztmavení místo samostatné light palety podle statusu.

**Primary recommendation:** Rozdělit práci na dvě vlny:
1. Terminálové theming + scrollbar (shared wrapper, pokryje Claude i Build panel).
2. Git barvy ve file tree (semantic status mapping + light palette).

---

## Existing State (Code-Backed)

### Terminál

- Společný wrapper:
  - `src/app/ui/terminal/instance/mod.rs` (`Terminal::ui`)
  - Volán z:
    - `src/app/ui/terminal/right/mod.rs` (Claude panel)
    - `src/app/ui/terminal/bottom/mod.rs` (Build terminál)
- V `vendor/egui_term/src/view.rs`:
  - `TerminalView::set_theme(mut self, theme: TerminalTheme) -> Self` existuje.
- V projektu se `set_theme` nikde nepoužívá.
- Scrollbar:
  - `src/app/ui/terminal/instance/render.rs` → hardcoded track/thumb barvy.

### Git barvy ve file tree

- Produkce git status barev:
  - `src/app/ui/background.rs` → `parse_git_status(...)`.
- Konzumace barev:
  - `src/app/ui/file_tree/render.rs` → `adapt_git_color`.
- Light mode dnes:
  - mechanické `* 0.55` pro všechny statusy, bez explicitní light palette per status.

---

## Architecture Patterns

### Pattern 1: Theme injection do TerminalView (bez restartu backendu)

**Co:** Přidat theme objekt do `Terminal` instance a při renderu volat `TerminalView::set_theme(theme)`.

**Proč:** Změna je čistě vizuální; běžící PTY procesy zůstanou nedotčené.

**Dopad:** TERM-01, TERM-02, TERM-04.

### Pattern 2: Runtime theme sync přes existující `settings_version`

**Co:** Při změně settings verze (tam, kde už se volá `settings.apply`) propagovat terminal theme update na `ws.claude_tabs` a `ws.build_terminal`.

**Proč:** V projektu už je this mechanismus canonical cesta pro změnu tématu.

**Dopad:** okamžitá reakce po přepnutí tématu, bez scope creep.

### Pattern 3: Theme-aware scrollbar

**Co:** Nahradit hardcoded barvy scrollbaru v `draw_scrollbar` theme-aware výpočtem (idle/hover/drag varianty).

**Proč:** TERM-03 explicitně zakazuje hardcoded tmavý scrollbar.

### Pattern 4: Semantic git status palette pro dark/light

**Co:** Držet status-semantiku (`modified`, `added`, `deleted`, `untracked`) a z ní odvozovat barvu podle aktivního mode (`dark` vs `light`).

**Proč:** Lepší kontrola čitelnosti než globální multiplikace.

**Dopad:** TREE-01, TREE-02.

---

## Risks and Mitigations

### Risk 1: egui_term history recolor po runtime switchi
- **Riziko:** část historie může vyžadovat repaint/refresh cyklus.
- **Mitigace:** update theme při každém `Terminal::ui(...)` renderu; fallback bez restartu procesu.

### Risk 2: regressions v kontrastu ANSI barev
- **Riziko:** některé ANSI kombinace budou na světlém pozadí slabé.
- **Mitigace:** explicitní light-safe palette (hlavně yellow/cyan/bright variants), manuální smoke test na typických výstupech (`cargo check`, warning/error logy).

### Risk 3: file tree ztratí konzistenci mezi statusy
- **Riziko:** ad-hoc barvy bez semantiky povedou k vizuálním kolizím.
- **Mitigace:** zavést jediný status->semantic map a dvě palety (dark/light) nad ním.

---

## Recommended Plan Decomposition

### Wave 1 (foundation)
- Plan A: Terminal theme model + runtime apply
  - `src/app/ui/terminal/instance/mod.rs`
  - `src/app/ui/terminal/instance/backend.rs` (jen pokud bude třeba init defaults)
  - `src/app/mod.rs` (napojení na settings_version propagate path)
- Plan B: Scrollbar theming
  - `src/app/ui/terminal/instance/render.rs`
  - případně sdílené helpery pro thumb/track colors

### Wave 2 (dependent on visual foundation)
- Plan C: Git color semantics + light palette
  - `src/app/ui/background.rs`
  - `src/app/ui/file_tree/mod.rs`
  - `src/app/ui/file_tree/render.rs`

Dependency: Wave 2 může běžet po stabilizaci theme behavior ve Wave 1 (lepší konzistence vizuálních testů).

---

## Testing Strategy (for planner)

### Automated (unit-level)
- test mapování git status -> semantic typ -> dark/light barvy
- test fallback theme při neznámém módu
- test že terminal runtime update nemění process state (nevolá restart path)

### Manual smoke
- Přepnutí dark/light s běžícím dlouhým příkazem v Claude i Build terminálu
- Kontrast warning/error/success ANSI na světlém pozadí
- Viditelnost `??` ve file tree na světlém pozadí
- Scrollbar track/thumb čitelný + hover/drag reakce

---

## Validation Architecture

### Observable Truths
1. Claude panel i Build terminál používají light-safe pozadí a čitelný text v light mode.
2. Přepnutí dark/light neukončí běžící terminálový proces.
3. Scrollbar terminálu nepoužívá hardcoded tmavé barvy; odvozuje se z aktivního tématu.
4. File tree používá explicitní light palette pro git statusy.
5. `??` (untracked) je v light mode viditelný a odlišitelný od ostatních statusů.

### Automated Evidence Targets
- Jednotkové testy pro mapování git statusů a palette resolver.
- Jednotkové testy pro theme resolver terminálu (dark vs light).
- `cargo check` a relevantní test subset pro dotčené moduly.

### Manual Evidence Targets
- Screen-level ověření obou terminálů při runtime switchi.
- Ověření kontrastu scrollbaru a git barev v light mode na reálném projektu.

### Pass Criteria
- TERM-01..04 a TREE-01..02 mají přímý důkaz v kódu + minimálně 1 manuální smoke krok na každý requirement cluster (Terminal, File Tree).

---

## Notes for Planner

- Nedělat scope expansion mimo terminál + file tree barvy.
- Využít existující `settings_version` flow, ne paralelní mechanismus.
- Preferovat malé, atomické plány s jasným must_haves mapováním na TERM/TREE requirements.

