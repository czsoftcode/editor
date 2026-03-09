# Stack Research

**Domain:** Desktop code editor save workflow (Rust + eframe/egui)
**Researched:** 2026-03-09
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | existing project toolchain | Implementace save state logiky a guardů | Už je primární jazyk projektu a drží bezpečnost/threading bez nového runtime |
| eframe/egui | existing project dependency | UI tlačítka, toggle režimu, potvrzovací dialogy | Stávající UI stack a modal pattern už je v projektu zavedený |
| serde + settings.toml persistence | existing project dependency | Uložení volby auto/manual save do Settings | Již použitý způsob konfigurace bez nových závislostí |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| notify (existing) | existing | Eventy změn souborů mimo editor | Jen pro externí změny; ne pro interní save režim přepínání |
| tokio::sync / std::sync patterns (existing) | existing | Bezpečné předání signalizace mezi UI a background částmi | Když save guard ovlivní více viewportů/windows |
| anyhow/thiserror style handling already in repo | existing | Přenos I/O chyb do UI toastů | Pro fail-safe save a close-flow chybové stavy |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo check | Rychlé ověření kompilace | Povinné po každém patchi |
| ./check.sh | Projektová validační sada | Aktuálně padá na existujících fmt rozdílech mimo scope |
| targeted cargo test | Ověření save/close edge case logiky | Přidat testy pro modal rozhodnutí a dirty-states |

## Installation

```bash
# No new packages required for this milestone
cargo check
./check.sh
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Persistovaný save mode v settings | Runtime-only toggle bez persist | Pouze pro experiment, ne pro uživatelsky očekávané nastavení |
| Explicitní close-confirm dialog | Tiché auto-save on close | Jen pokud je produkt striktně auto-save-first a uživatel to očekává |
| Existing egui modal pattern | Nový custom modal framework | Nedoporučeno, pokud současný pattern pokrývá požadavky |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Nový async runtime jen kvůli save flow | Porušuje stávající architekturu a zvyšuje složitost | Stávající sync/background patterny v projektu |
| Implicitní zahození změn při close | Vede ke ztrátě dat | Povinný confirm flow při dirty stavech |
| Dvojí zdroj pravdy pro save režim | Rozpad konzistence mezi UI a persistencí | Jediný source-of-truth v Settings + runtime apply |

## Stack Patterns by Variant

**If Manual Save mode:**
- Use explicit Ctrl+S and close confirmations.
- Because uživatel vědomě řídí okamžik zápisu na disk.

**If Auto Save mode:**
- Keep background autosave, but still guard against pending write/failure on close.
- Because auto-save nesmí skrýt chyby ani vést k tichým ztrátám dat.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| eframe/egui (repo) | current settings + modal implementation | Bez změny dependency tree |
| serde settings model | existing Settings migration logic | Nové pole musí mít `#[serde(default)]` |

## Sources

- `.planning/PROJECT.md` — current architecture and constraints
- `.planning/ROADMAP.md` — existing phase conventions and status
- Existing code patterns: settings persistence, modal dialogs, tab dirty-state handling

---
*Stack research for: desktop editor save workflow*
*Researched: 2026-03-09*
