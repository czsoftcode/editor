# PolyCredo Editor — Performance Optimization

## What This Is

Investigace a oprava nadměrné spotřeby CPU a paměti v PolyCredo Editoru (Rust/eframe/egui).
Editor v současnosti zatěžuje procesor i v idle stavu i při psaní kódu, což způsobuje zahřívání notebooku.
Cílem je identifikovat hlavní viníky a zavést cílená opatření pro snížení tepelné a výpočetní zátěže.

## Core Value

Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Identifikovat hlavní příčiny vysokého CPU v idle (profilování / měření)
- [ ] Omezit zbytečné překreslování egui renderovací smyčky (conditional repaint)
- [ ] Snížit frekvenci nebo optimalizovat git polling (aktuálně každých 5s)
- [ ] Optimalizovat autosave timer (aktuálně 500ms interval)
- [ ] Prověřit FileWatcher/ProjectWatcher overhead
- [ ] Měřitelně snížit idle CPU zátěž

### Out of Scope

- LSP integrace — jiný projekt, nesouvisí s performance
- Nové UI funkce — pouze optimalizace existujícího chování
- Minimap — patří do jiné oblasti

## Context

PolyCredo Editor je Rust/eframe/egui desktop editor. Klíčové oblasti s potenciálním výkonovým problémem:

- **egui render loop** — egui defaultně renderuje každý frame (60fps+) i když se nic nezměnilo; `request_repaint()` nebo `request_repaint_after()` umožňují omezit zbytečné překreslování
- **Git polling** — refresh každých 5s (`watcher.rs` / `workspace.rs`), probíhá async nebo na hlavním vlákně?
- **Autosave** — 500ms timer v `editor.rs`, neustálé kontroly stavu
- **FileWatcher/ProjectWatcher** — `notify` crate, potenciální overhead při velkých projektech
- **egui_term** — terminálový emulátor může blokovat nebo busy-loop pokud čeká na výstup

## Constraints

- **Tech stack**: Rust + eframe/egui — řešení musí být kompatibilní s tímto stackem
- **Zpětná kompatibilita**: Funkčnost editoru nesmí být narušena
- **Bez externích závislostí**: Nechceme přidávat nové heavy dependencies jen kvůli optimalizaci

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Profilovat před optimalizací | Neopravovat naslepo — změřit skutečné hotspoty | — Pending |

---
*Last updated: 2026-03-04 after initialization*
