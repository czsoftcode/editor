# ROADMAP: PolyCredo Editor

## Milestones

- ✅ **v1.3.0 AI Terminal Cleanup** — Phases 30-34 (shipped 2026-03-11) — [ROADMAP archive](.planning/milestones/v1.3.0-ROADMAP.md), [REQUIREMENTS archive](.planning/milestones/v1.3.0-REQUIREMENTS.md), [AUDIT](.planning/milestones/v1.3.0-MILESTONE-AUDIT.md)
- ◆ **v1.3.1 Safe Trash Delete** — Phases 35-38 (in planning)

## Proposed Roadmap (v1.3.1)

**4 phases** | **11 requirements mapped** | All covered ✓

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 35 | 3/3 | Complete    | 2026-03-11 | 3 |
| 36 | 1/3 | In Progress|  | 4 |
| 37 | Trash Preview + Restore MVP | Dodat nahled trash obsahu a restore workflow s conflict policy | TRASHUI-01, RESTORE-01, RESTORE-02, RESTORE-03 | 4 |
| 38 | Watcher Stability + Verification | Stabilizovat watcher eventy po delete/restore a uzavrit quality gate | RELIAB-03 | 3 |

## Phase Details

### Phase 35: Trash Foundation + Async Safety

**Goal:** Zalozit technicky zaklad pro trash workflow bez blokovani UI.

**Requirements:** TRASH-03, RELIAB-01

**Success criteria:**
1. `.polycredo/trash` se automaticky vytvori pri prvni delete/restore operaci, pokud neexistuje.
2. Delete/restore I/O operace bezi mimo UI vlakno (bez viditelneho freeze v interakci editoru).
3. Zakladni metadata format trash polozek je stabilni a pouzitelny pro restore.

### Phase 36: Safe Move-to-Trash Engine

**Goal:** Zmenit delete tok na move-to-trash a zajistit fail-safe chovani.

**Requirements:** TRASH-01, TRASH-02, TRASH-04, RELIAB-02

**Success criteria:**
1. Mazani souboru i adresaru uz neprovadi hard delete, ale presun do `.polycredo/trash`.
2. Pri selhani presunu nedojde ke ztrate dat (zdrojova polozka zustane zachovana).
3. Vsechny I/O chyby v delete toku se propisi do toastu s citelnou zpravou.
4. `cargo check` a `./check.sh` projdou pro scope zmen teto faze.

### Phase 37: Trash Preview + Restore MVP

**Goal:** Dodat uzivatelsky nahled trash obsahu a obnovu polozek.

**Requirements:** TRASHUI-01, RESTORE-01, RESTORE-02, RESTORE-03

**Success criteria:**
1. Uzivatel vidi v aplikaci seznam polozek v trash a muze vybrat polozku pro obnovu.
2. Restore vrati polozku na puvodni cestu, pokud neni konflikt.
3. Pri konfliktu cilove cesty je pouzita nedestruktivni policy (zadny tichy overwrite).
4. Po restore je UI konzistentni (file tree + otevrene taby odpovidaji realnemu stavu souboru).

### Phase 38: Watcher Stability + Verification

**Goal:** Zajistit, ze delete/restore workflow nezpusobuje watcher regresi a uzavrit verifikaci.

**Requirements:** RELIAB-03

**Success criteria:**
1. Watcher eventy po move-to-trash/restore jsou deduplikovane nebo batchovane tak, aby nevznikal event storm.
2. Pri sekvenci delete -> restore nedochazi k viditelnemu lagovani UI.
3. Final gate (`cargo check` + `./check.sh`) je zaznamenany jako PASS v phase verification artefaktech.

## Next Command

`$gsd-discuss-phase 35`
