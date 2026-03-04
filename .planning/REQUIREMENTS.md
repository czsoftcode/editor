# Requirements: PolyCredo Editor — Performance Optimization

**Defined:** 2026-03-04
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1 Requirements

### Repaint Optimization

- [ ] **RPNT-01**: Aplikace nerepaintuje každý frame v idle stavu — používá `request_repaint_after(Duration)` podmíněně
- [ ] **RPNT-02**: Accesskit feature je zakázána v `Cargo.toml` (eliminace D-Bus overhead na Linuxu)
- [ ] **RPNT-03**: Při minimalizovaném nebo nezaměřeném okně se render loop zpomalí nebo zastaví
- [ ] **RPNT-04**: FPS je omezen při aktivním psaní (nepotřebujeme 60fps při každém stisku klávesy)

### Background Tasks

- [ ] **BKGD-01**: Git polling běží v dedikovaném background threadu — neblokuje render loop
- [ ] **BKGD-02**: FileWatcher události jsou debouncovány — burst events při Ctrl+S nezpůsobují zbytečné repaints
- [ ] **BKGD-03**: Autosave se spustí pouze při skutečné změně obsahu (dirty flag), ne na každý timer tick
- [ ] **BKGD-04**: egui_term repaintuje pouze při příchodu nových dat z PTY procesu, ne periodicky

## v2 Requirements

### Advanced Optimizations

- **ADV-01**: `very_lazy` eframe feature flag (nová funkce z PR #4880 — skip repaint při pohybu myši nad non-hoverable prvky)
- **ADV-02**: Puffin profiler za feature flag pro budoucí instrumentaci
- **ADV-03**: Git polling interval prodloužen na 30s (místo 5s)
- **ADV-04**: Virtual scroll pro velké soubory (>10 000 řádků)

## Out of Scope

| Feature | Reason |
|---------|--------|
| LSP integrace | Jiný projekt, nesouvisí s performance |
| Nové UI funkce | Pouze optimalizace existujícího chování |
| Migrace na jiný GUI framework | Příliš velký scope, zachováme eframe/egui |
| Profilování / puffin setup | Uživatel se rozhodl přeskočit, rovnou opravovat |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| RPNT-01 | Phase 1 | Pending |
| RPNT-02 | Phase 1 | Pending |
| RPNT-03 | Phase 1 | Pending |
| RPNT-04 | Phase 1 | Pending |
| BKGD-01 | Phase 2 | Pending |
| BKGD-02 | Phase 2 | Pending |
| BKGD-03 | Phase 2 | Pending |
| BKGD-04 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 8 total
- Mapped to phases: 8
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-04*
*Last updated: 2026-03-04 — phase mapping confirmed after roadmap creation*
