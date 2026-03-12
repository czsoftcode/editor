# Requirements: PolyCredo Editor

**Defined:** 2026-03-11
**Core Value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## v1 Requirements (Milestone v1.3.1 Safe Trash Delete)

### TRASH (Safe Delete Core)

- [x] **TRASH-01**: Uzivatel muze smazat soubor bez hard delete; operace presune soubor do `.polycredo/trash`.
- [x] **TRASH-02**: Uzivatel muze smazat adresar bez hard delete; operace presune adresar do `.polycredo/trash`.
- [x] **TRASH-03**: Aplikace automaticky vytvori `.polycredo/trash`, pokud neexistuje.
- [x] **TRASH-04**: Pri selhani move-to-trash operace zustane puvodni soubor/adresar zachovan a uzivatel dostane chybu v toastu.

### RESTORE (Recovery)

- [x] **RESTORE-01**: Uzivatel muze obnovit jednu polozku z trash zpet na puvodni cestu.
- [x] **RESTORE-02**: Pokud cilova cesta pri restore uz existuje, aplikace pouzije nedestruktivni conflict policy (bez ticheho prepisu) a zobrazi jasnou volbu/chybu.
- [ ] **RESTORE-03**: Po restore se UI (strom souboru/otevrene taby) konzistentne obnovi bez nutnosti restartu aplikace.

### TRASHUI (Trash Preview)

- [x] **TRASHUI-01**: Uzivatel ma v aplikaci nahled do obsahu `.polycredo/trash` (seznam smazanych polozek) a z tohoto nahledu muze spustit restore vybrane polozky.

### RELIAB (Reliability + UX)

- [x] **RELIAB-01**: Delete/restore operace neblokuji UI vlakno (tezke I/O bezi mimo UI vlakno).
- [x] **RELIAB-02**: Vsechny I/O chyby v delete/restore/cleanup toku jsou propagovany do UI toastu.
- [ ] **RELIAB-03**: Watcher/event handling po delete/restore nezpusobi event storm vedouci k viditelnemu lagovani UI.

## v2 Requirements

### TRASH

- **TRASH-05**: Uzivatel muze provest bulk restore vice polozek najednou.
- **TRASH-06**: Uzivatel muze spustit policy-based retention cleanup (age/count/size limity).

### PLATFORM

- **PLAT-01**: Aplikace umi volitelne napojeni na nativni OS recycle bin podle platformy.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Tichy overwrite existujiciho souboru pri restore | Riziko nevratne ztraty dat; porusuje safe-delete cil milestone. |
| Full trash timeline UI s preview/diff | Vyssi slozitost; neni nutne pro validaci jadra v1.3.1. |
| Povinne napojeni na OS recycle bin | Zvysuje platform-specific slozitost a QA matici; odlozeno do v2+. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TRASH-01 | Phase 36 | Complete |
| TRASH-02 | Phase 36 | Complete |
| TRASH-03 | Phase 35 | Complete |
| TRASH-04 | Phase 36 | Complete |
| RESTORE-01 | Phase 37 | Complete |
| RESTORE-02 | Phase 37 | Complete |
| RESTORE-03 | Phase 37 | Pending |
| TRASHUI-01 | Phase 37 | Complete |
| RELIAB-01 | Phase 35 | Complete |
| RELIAB-02 | Phase 36 | Complete |
| RELIAB-03 | Phase 38 | Pending |

**Coverage:**
- v1 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after milestone v1.3.1 initialization*
