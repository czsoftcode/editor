# Feature Research

**Domain:** Desktop editor save UX
**Researched:** 2026-03-09
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Ctrl+S manual save | Standard editor behavior | LOW | Must always save current active file/tab |
| Unsaved changes indicator | User musí vidět, co není na disku | LOW | Tab marker + statusbar message |
| Confirm on close with dirty file | Ochrana proti ztrátě dat | MEDIUM | Potřeba řešit tab close i app close |
| Save mode setting (auto/manual) | Různé workflow preference | MEDIUM | Persistovat do settings a aplikovat runtime |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Unified close decision flow | Stejné chování pro tab i aplikaci | MEDIUM | Snižuje zmatení uživatele |
| Explicit save-mode label v UI | Okamžitá orientace v režimu | LOW | Předchází nechtěnému chování |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-save without user-visible mode | „Ať to ukládá samo“ | Uživatel neví, proč se změny propsaly | Jasný přepínač + indikace aktivního režimu |
| Silently discard on close | „Méně dialogů“ | Přímá ztráta práce | Confirm dialog s volbou Save/Discard/Cancel |

## Feature Dependencies

```
Save mode setting
    └──requires──> Persistent settings update
                         └──requires──> Runtime propagation to editor state

Unsaved close guard
    └──requires──> Reliable dirty-state per tab/file

Ctrl+S default
    └──enhances──> Manual save mode
```

### Dependency Notes

- **Save mode requires persistence:** bez persist by se režim po restartu vracel jinam.
- **Close guard requires dirty-state:** bez přesného dirty modelu by dialogy byly falešné nebo chyběly.
- **Ctrl+S enhances manual mode:** je to hlavní vstupní cesta v ručním režimu.

## MVP Definition

### Launch With (v1)

- [ ] Ctrl+S uloží aktivní soubor ve všech běžných stavech
- [ ] Nastavení auto/manual režimu s persistencí
- [ ] Confirm před zavřením tabu s neuloženým souborem
- [ ] Confirm před zavřením aplikace při neuložených změnách

### Add After Validation (v1.x)

- [ ] Per-project save mode override
- [ ] Bulk action v close dialogu („Save all“, „Discard all“)

### Future Consideration (v2+)

- [ ] Granulární auto-save policy (on focus loss / interval / debounce profile)
- [ ] Recovery snapshot browser

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Ctrl+S default manual save | HIGH | LOW | P1 |
| Auto/manual mode switch | HIGH | MEDIUM | P1 |
| Dirty close guard (tab + app) | HIGH | MEDIUM | P1 |
| Save mode UI indicator | MEDIUM | LOW | P2 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Competitor A | Competitor B | Our Approach |
|---------|--------------|--------------|--------------|
| Ctrl+S save | VS Code-style manual save | JetBrains-style manual save | Standard Ctrl+S behavior jako default |
| Dirty close confirm | Save/Discard/Cancel dialog | Save all / Discard / Cancel flow | Minimalní, konzistentní guard pro tab i app |

## Sources

- Existing editor UX conventions and current PolyCredo workflow
- `.planning/PROJECT.md`, `.planning/ROADMAP.md`, current tab/save logic expectations

---
*Feature research for: desktop editor save UX*
*Researched: 2026-03-09*
