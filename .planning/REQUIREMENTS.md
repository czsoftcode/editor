# Requirements: PolyCredo Editor v1.3.0

**Defined:** 2026-03-10
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1.3.0 Requirements

Requirements for Additional Themes milestone.

### Theme Variants

- [ ] **THEME-01**: User can select 4th light theme variant in Settings (between Sepia and Brown, pleasant for eyes, not too dark)
- [ ] **THEME-02**: 4th light theme is persisted across app restart in settings.toml
- [ ] **THEME-03**: 4th light theme shows correctly in theme picker with visual swatch
- [ ] **THEME-04**: 4th light theme has localized label in all 5 languages (cs, en, de, ru, sk)
- [ ] **THEME-05**: User can select 2nd dark theme variant in Settings (optional variant to existing dark)
- [ ] **THEME-06**: 2nd dark theme is persisted across app restart in settings.toml
- [ ] **THEME-07**: 2nd dark theme shows correctly in theme picker with visual swatch

### Syntax Highlighting

- [ ] **SYNTAX-01**: Each light theme variant maps to appropriate syntect theme (not all using same "Solarized (light)")
- [ ] **SYNTAX-02**: Dark variants map to appropriate syntect dark themes

## Future Requirements

### Theme Extensions

- **THEME-EXT-01**: Per-file syntax highlighting theme override
- **THEME-EXT-02**: Theme export/import functionality

## Out of Scope

| Feature | Reason |
|---------|--------|
| Animated theme transitions | Not available in egui |
| Custom theme editor | Outside scope of this milestone |
| OS auto-detect dark/light | Experimental in egui, intentionally excluded |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| THEME-01 | Phase 27 | Pending |
| THEME-02 | Phase 27 | Pending |
| THEME-03 | Phase 27 | Pending |
| THEME-04 | Phase 27 | Pending |
| THEME-05 | Phase 28 | Pending |
| THEME-06 | Phase 28 | Pending |
| THEME-07 | Phase 28 | Pending |
| SYNTAX-01 | Phase 29 | Pending |
| SYNTAX-02 | Phase 29 | Pending |

**Coverage:**
- v1.3.0 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0

---
*Requirements defined: 2026-03-10*
*Last updated: 2026-03-10 after research phase*
