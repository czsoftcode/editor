# Requirements: PolyCredo Editor v1.3.0

**Defined:** 2026-03-11
**Core Value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## v1 Requirements

### CLI Removal

- [x] **CLI-01**: Kod v `src/app/cli/*` je odstraneny z repozitare.
- [x] **CLI-02**: Build prochazi bez importu `crate::legacy CLI namespace::*`.
- [x] **CLI-03**: Neexistuji mrtve exporty/moduly vazane na puvodni CLI vrstvu.

### AI Terminal Behavior

- [x] **TERM-01**: Uzivatel muze otevrit AI terminal panel a odeslat prompt.
- [x] **TERM-02**: Streaming odpovedi funguje bez zamrznuti UI.
- [x] **TERM-03**: Assistant-only slash/GSD prikazy zustavaji funkcni bez provider-picker couplingu.

### Safety and Tooling

- [x] **SAFE-01**: Approval flow pro citlive AI operace zustava funkcni.
- [x] **SAFE-02**: Security filtry/rate limit/path sandbox zustavaji funkcni po migraci.
- [x] **SAFE-03**: Audit/error handling zustava funkcni v AI terminal workflow.
Poznamka: Approval/security kontrakt se v phase 31 nemeni; gap closure resi pouze odstraneni provider-picker couplingu.

### Cleanup and Stability

- [x] **STAB-01**: `cargo check` a `./check.sh` prochazi po odstraneni CLI vrstvy.
- [x] **STAB-02**: Relevantni testy jsou aktualizovany na novy namespace.
Poznamka: Evidence-first sign-off je veden v `.planning/phases/32-cleanup-tests-and-stabilization/32-VERIFICATION.md`.

### Launcher-only Removal (Phase 33)

- [x] **R33-A**: Jediny aktivni AI tok v aplikaci je `ai_bar -> terminal.send_command`.
- [x] **R33-B**: `src/app/odstraneny runtime modul/*` a `src/app/ui/terminal/odstraneny chat modul/*` jsou odstranene a bez aktivnich referenci.
- [x] **R33-C**: Legacy AI chat entrypointy jsou odstranene bez fallback UX/toastu/deprecated vetvi.
- [x] **R33-D**: Zminky `puvodni CLI integrace|odstraneny runtime modul|odstraneny chat modul|legacy CLI namespace` jsou vycistene v aktivnim i historickem planning scope podle planu phase 33.

## v2 Requirements

### Future Enhancements

- **FUT-01**: Revize AI terminal UX po stabilizaci cleanupu.
- **FUT-02**: Pripadna podpora dalsich provideru az po v1.3.0.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Novy AI provider | Mimo scope cleanup milestone |
| UI redesign AI panelu | Cilem je zachovat chovani, ne menit UX |
| Velky cross-module refactor mimo AI/CLI | Zvysuje riziko regresi bez prime hodnoty |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-01 | Phase 30 | Complete |
| CLI-02 | Phase 30 | Complete |
| CLI-03 | Phase 30 | Complete |
| TERM-01 | Phase 31 | Complete |
| TERM-02 | Phase 31 | Complete |
| TERM-03 | Phase 31 | Complete |
| SAFE-01 | Phase 31 | Complete |
| SAFE-02 | Phase 31 | Complete |
| SAFE-03 | Phase 31 | Complete |
| STAB-01 | Phase 32 | Complete |
| STAB-02 | Phase 32 | Complete |
| R33-A | Phase 33 | Complete |
| R33-B | Phase 33 | Complete |
| R33-C | Phase 33 | Complete |
| R33-D | Phase 33 | Complete |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after milestone v1.3.0 kickoff*
