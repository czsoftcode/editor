# Requirements: PolyCredo Editor v1.3.0

**Defined:** 2026-03-11
**Core Value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## v1 Requirements

### CLI Removal

- [x] **CLI-01**: Kod v `src/app/cli/*` je odstraneny z repozitare.
- [x] **CLI-02**: Build prochazi bez importu `crate::app::cli::*`.
- [x] **CLI-03**: Neexistuji mrtve exporty/moduly vazane na puvodni CLI vrstvu.

### AI Terminal Behavior

- [x] **TERM-01**: Uzivatel muze otevrit AI terminal panel a odeslat prompt.
- [ ] **TERM-02**: Streaming odpovedi funguje bez zamrznuti UI.
- [x] **TERM-03**: Model picker a slash/GSD prikazy zustavaji funkcni.

### Safety and Tooling

- [ ] **SAFE-01**: Approval flow pro citlive AI operace zustava funkcni.
- [ ] **SAFE-02**: Security filtry/rate limit/path sandbox zustavaji funkcni po migraci.
- [x] **SAFE-03**: Audit/error handling zustava funkcni v AI terminal workflow.

### Cleanup and Stability

- [ ] **STAB-01**: `cargo check` a `./check.sh` prochazi po odstraneni CLI vrstvy.
- [ ] **STAB-02**: Relevantni testy jsou aktualizovany na novy namespace.

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
| TERM-02 | Phase 31 | Pending |
| TERM-03 | Phase 31 | Complete |
| SAFE-01 | Phase 31 | Pending |
| SAFE-02 | Phase 31 | Pending |
| SAFE-03 | Phase 31 | Complete |
| STAB-01 | Phase 32 | Pending |
| STAB-02 | Phase 32 | Pending |

**Coverage:**
- v1 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after milestone v1.3.0 kickoff*
