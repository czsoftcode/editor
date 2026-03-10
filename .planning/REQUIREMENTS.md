# Requirements: PolyCredo Editor v1.3.0

**Defined:** 2026-03-09
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1.3.0 Requirements

Requirements for Save Modes + Unsaved Changes Guard milestone. Each maps to roadmap phases.

### Save Controls

- [x] **SAVE-01**: User can press `Ctrl+S` and save currently active file tab.
- [x] **SAVE-02**: Manual save updates tab/file state from modified to saved without requiring focus change.
- [x] **SAVE-03**: Save errors are surfaced in UI (toast/message) and do not silently fail.

### Save Mode Settings

- [x] **MODE-01**: User can switch between `Automatic Save` and `Manual Save` in Settings.
- [x] **MODE-02**: Selected save mode is persisted across app restart.
- [x] **MODE-03**: Runtime behavior immediately reflects selected save mode after Settings Save.
- [x] **MODE-04**: UI clearly indicates active save mode so user understands current behavior.

### Unsaved Changes Protection

- [x] **GUARD-01**: Closing a tab with unsaved changes opens a confirmation decision flow.
- [x] **GUARD-02**: Closing the application with any unsaved tabs opens a confirmation decision flow.
- [x] **GUARD-03**: Confirmation flow allows user to save changes, discard changes, or cancel close.
- [x] **GUARD-04**: If save fails during close flow, user is informed and close is not silently completed.

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
| SAVE-01 | Phase 24 | Complete |
| SAVE-02 | Phase 24 | Complete |
| SAVE-03 | Phase 24 | Complete |
| MODE-01 | Phase 24 | Complete |
| MODE-02 | Phase 24 | Complete |
| MODE-03 | Phase 24 | Complete |
| MODE-04 | Phase 26 | Complete |
| GUARD-01 | Phase 25 | Complete |
| GUARD-02 | Phase 25 | Complete |
| GUARD-03 | Phase 25 | Complete |
| GUARD-04 | Phase 25 | Complete |

**Coverage:**
- v1.3.0 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-09*
*Last updated: 2026-03-09 after initial milestone definition*
