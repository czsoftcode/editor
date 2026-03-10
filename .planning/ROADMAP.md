# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- ✅ **v1.1.0 Sandbox Removal** — Phases 9-12 (shipped 2026-03-06)
- ✅ **v1.2.0 AI Chat Rewrite** — Phases 13-18 (shipped 2026-03-06)
- ✅ **v1.2.1 Save Modes + Unsaved Changes Guard** — Phases 24-26 (shipped 2026-03-10)
- ⏸️ **v1.2.2-dev GSD Integration + Slash Commands** — Phases 19-23 (deferred/cancelled)
- 🔄 **v1.3.0 Additional Themes** — Phases 27-29 (in progress)

---

## Phases

### v1.3.0 Additional Themes

- [ ] **Phase 27: 4th Light Theme** - Přidat 4. světlé téma (mezi Sepia a Brown)
- [ ] **Phase 28: Dark Variant Support** - Přidat 2. dark téma jako variantu
- [ ] **Phase 29: Syntect Theme Mapping** - Opravit mapování syntect témat pro všechny varianty

---

### Phase 27: 4th Light Theme

**Goal:** User can select and use 4th light theme variant

**Depends on:** Nothing (first phase)

**Requirements:** THEME-01, THEME-02, THEME-03, THEME-04

**Success Criteria** (what must be TRUE):
1. User can see 4th light variant option in Settings → Theme picker UI
2. User can select 4th light variant and see immediate visual change
3. User can restart app and 4th light variant is still selected (persisted in settings.toml)
4. 4th light variant shows visual swatch in theme picker
5. 4th light variant has localized label in all 5 languages (cs, en, de, ru, sk)

**Plans:** 1 plan

- [ ] 27-01-PLAN.md — Add WarmTan light variant (enum, colors, UI picker, i18n)

---

### Phase 28: Dark Variant Support

**Goal:** User can select and use 2nd dark theme variant

**Depends on:** Phase 27

**Requirements:** THEME-05, THEME-06, THEME-07

**Success Criteria** (what must be TRUE):
1. User can see 2nd dark variant option in Settings → Theme picker UI
2. User can select 2nd dark variant and see immediate visual change
3. User can restart app and 2nd dark variant is still selected (persisted in settings.toml)
4. 2nd dark variant shows visual swatch in theme picker

**Plans:** TBD

---

### Phase 29: Syntect Theme Mapping

**Goal:** Syntax highlighting uses appropriate theme per color variant

**Depends on:** Phase 27, Phase 28

**Requirements:** SYNTAX-01, SYNTAX-02

**Success Criteria** (what must be TRUE):
1. Each light theme variant (WarmIvory, CoolGray, Sepia, Stone) maps to distinct syntect theme
2. Dark variants (Default, Midnight) map to distinct syntect dark themes
3. Syntax highlighting visually matches the selected color palette

**Plans:** TBD

---

### Pozastavené fáze

Následující fáze jsou pozastaveny (nebudou dokončeny):
- Phase 19-23: GSD Integration + Slash Commands — slepá větev

---

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 27 | v1.3.0 | 1/1 | Planning complete | - |
| 28 | v1.3.0 | 0/1 | Not started | - |
| 29 | v1.3.0 | 0/1 | Not started | - |
| 19-23 | v1.2.2-dev | - | Deferred | - |

## Coverage Map

```
v1.3.0:
  Phase 27: THEME-01, THEME-02, THEME-03, THEME-04
  Phase 28: THEME-05, THEME-06, THEME-07
  Phase 29: SYNTAX-01, SYNTAX-02
  
Mapped: 9/9 ✓
```

---

## Known Issues / TODO

- Legacy quick tasks table ve `STATE.md` obsahuje historické řádky mimo standard tabulku; při dalším housekeepingu sjednotit formát.
- `./check.sh` aktuálně padá na repo-wide `cargo fmt --check` rozdílech mimo scope této roadmap inicializace.
- Warning text kontrast v light mode — existující tech debt z v1.0.2
- Syntax highlighting v AI chatu nefunguje (existující tech debt)

---

*Last updated: 2026-03-10*
