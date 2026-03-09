# Requirements: PolyCredo Editor v1.3.0

**Defined:** 2026-03-09
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1.3.0 Requirements

Requirements for Save Modes + Unsaved Changes Guard milestone. Each maps to roadmap phases.

### Save Controls

- [ ] **SAVE-01**: User can press `Ctrl+S` and save currently active file tab.
- [ ] **SAVE-02**: Manual save updates tab/file state from modified to saved without requiring focus change.
- [ ] **SAVE-03**: Save errors are surfaced in UI (toast/message) and do not silently fail.

### Save Mode Settings

- [ ] **MODE-01**: User can switch between `Automatic Save` and `Manual Save` in Settings.
- [ ] **MODE-02**: Selected save mode is persisted across app restart.
- [ ] **MODE-03**: Runtime behavior immediately reflects selected save mode after Settings Save.
- [ ] **MODE-04**: UI clearly indicates active save mode so user understands current behavior.

### Unsaved Changes Protection

- [ ] **GUARD-01**: Closing a tab with unsaved changes opens a confirmation decision flow.
- [ ] **GUARD-02**: Closing the application with any unsaved tabs opens a confirmation decision flow.
- [ ] **GUARD-03**: Confirmation flow allows user to save changes, discard changes, or cancel close.
- [ ] **GUARD-04**: If save fails during close flow, user is informed and close is not silently completed.

## Future Requirements

### Save Workflow Extensions

- **SAVE-EXT-01**: User can choose per-project save mode override.
- **SAVE-EXT-02**: Application close dialog supports `Save all` and `Discard all` bulk actions.
- **SAVE-EXT-03**: User can configure auto-save strategy (interval / focus-loss / debounce profile).

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full editor command palette redesign for save actions | Not required to deliver core save safety behavior |
| Cloud backup/version history browser | Large scope increase, unrelated to milestone goal |
| New persistence backend beyond existing settings/filesystem model | Existing storage model is sufficient for this milestone |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SAVE-01 | Phase 24 | Pending |
| SAVE-02 | Phase 24 | Pending |
| SAVE-03 | Phase 24 | Pending |
| MODE-01 | Phase 24 | Pending |
| MODE-02 | Phase 24 | Pending |
| MODE-03 | Phase 24 | Pending |
| MODE-04 | Phase 26 | Pending |
| GUARD-01 | Phase 25 | Pending |
| GUARD-02 | Phase 25 | Pending |
| GUARD-03 | Phase 25 | Pending |
| GUARD-04 | Phase 25 | Pending |

**Coverage:**
- v1.3.0 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-09*
*Last updated: 2026-03-09 after initial milestone definition*
