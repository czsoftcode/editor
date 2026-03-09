# Pitfalls Research

**Domain:** Save behavior and unsaved-change protection in desktop editor
**Researched:** 2026-03-09
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Silent data loss on close

**What goes wrong:**
Tab/aplikace se zavře bez potvrzení i když jsou změny v paměti.

**Why it happens:**
Close path neprovádí centralizovanou dirty kontrolu.

**How to avoid:**
Jednotný close guard se Save/Discard/Cancel a testy pro tab + app cesty.

**Warning signs:**
Uživatel hlásí „soubor se po restartu vrátil“.

**Phase to address:**
Phase 24 (Save Mode Foundation)

---

### Pitfall 2: Nekonzistentní chování auto/manual režimu

**What goes wrong:**
Některé cesty ukládají automaticky, jiné čekají na Ctrl+S bez jasné logiky.

**Why it happens:**
Save režim není single source of truth a je roztroušen v UI podmínkách.

**How to avoid:**
Persistovaný `save_mode` + centralizované rozhodnutí v save dispatcheru.

**Warning signs:**
Stejná akce vede v různých panelech k odlišnému výsledku.

**Phase to address:**
Phase 24

---

### Pitfall 3: Ctrl+S nefunguje spolehlivě při fokus změnách

**What goes wrong:**
Shortcut se ztratí při aktivním modalu nebo jiném panelu a uživatel má pocit, že save nefunguje.

**Why it happens:**
Nejednotné focus guardy nebo kolize shortcut handlerů.

**How to avoid:**
Jasná precedence shortcut handleru + explicitní testy pro modal/focus edge cases.

**Warning signs:**
Intermittent bug reporty „někdy Ctrl+S nic neudělá“.

**Phase to address:**
Phase 25 (Close Guard + UX hardening)

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Ignore save errors during close | Méně UI větvení | Tichá ztráta dat | Never |
| Duplicate close dialog code | Rychlé dodání | Regrese a nekonzistence | Never |
| Runtime-only save mode (bez persistence) | Rychlý prototyp | Po restartu jiné chování | Only for throwaway prototype |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Settings persistence | Missing serde default for new field | Add explicit default and migration-safe load |
| Tab close events | Handle only mouse close, not programmatic close | Route all close paths through same guard |
| App close hook | Ignore multi-tab dirty aggregation | Aggregate dirty state across all open tabs |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full-file rescans on each frame for dirty detection | UI micro-stutters | Maintain incremental dirty flag | Medium-large files, frequent typing |
| Save-all synchronous on app close without progress | Freeze on shutdown | Batch write + clear feedback | Many dirty tabs |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting stale path when saving tab | Write to wrong location | Validate current tab path before write |
| Ignoring I/O errors on final close | User believes save succeeded | Surface error and keep close cancellable |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Vague dialog text on close | User neví, co ztratí | Show affected file names/count |
| Hidden save mode | Uživatel nerozumí chování | Visible toggle + status indicator |

## "Looks Done But Isn't" Checklist

- [ ] **Ctrl+S:** funguje i po focus přepnutí mezi panely — verify shortcut tests.
- [ ] **Manual mode:** žádný implicitní autosave side-effect — verify save trigger matrix.
- [ ] **Close guard:** pokrývá tab close i app close — verify both paths.
- [ ] **Error path:** save failure při close nezavře app bez potvrzení — verify failure flow.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Lost unsaved changes | HIGH | Add guard immediately, add regression test, communicate behavior change |
| Inconsistent mode behavior | MEDIUM | Refactor to central save decision point and backfill tests |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Silent data loss on close | Phase 24 | Manual + unit tests for dirty close matrix |
| Inconsistent auto/manual logic | Phase 24 | Mode-switch behavioral tests |
| Ctrl+S focus regressions | Phase 25 | Shortcut integration tests |

## Sources

- Existing PolyCredo editor behavior and milestone goals
- Internal architecture constraints from `.planning/PROJECT.md`

---
*Pitfalls research for: save behavior in desktop editor*
*Researched: 2026-03-09*
