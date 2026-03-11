# Milestones

## v1.2.2 Additional Themes (Shipped: 2026-03-11)

**Phases completed:** 3 phases, 5 plans, 15 tasks

**Key accomplishments:**
- 4. light varianta WarmTan: picker + swatch + persistence + i18n
- 2. dark varianta Midnight: picker + persistence + vizuální odlišení
- Syntect mapování podle variant (light i dark), bez fallback kolizí
- Regresní testy pro light/dark varianty a lokalizaci

**Poznámka:** v1.2.2-dev (Phase 19-23) byla slepá ulička; nebrat jako základ pro další plánování.

---

## v1.2.1 Save Modes + Unsaved Changes Guard (Shipped: 2026-03-10)

**Phases completed:** 3 phases, 18 plans, 0 tasks

**Key accomplishments:**
- (none recorded)

---

## v1.2.0 AI Chat Rewrite (Shipped: 2026-03-06)

**Phases completed:** 6 phases, 19 plans, 74 commits (42 feat/fix)

**Key accomplishments:**
- AiProvider trait abstrakce + OllamaProvider s NDJSON streaming a auto-detect serveru
- AiState konsolidace — 19 polí z WorkspaceState do dedikované struktury s ChatState, OllamaState, AiSettings
- Hybrid CLI chat UI se streaming renderingem, markdown formátováním, dark/light mode, model picker
- Tool execution — read/write/exec/search/ask-user s approval workflow a security infrastrukturou (PathSandbox, SecretsFilter, CommandBlacklist, RateLimiter, AuditLogger)
- Kompletní odstranění WASM plugin systému (~6,500 LOC), plná i18n lokalizace v 5 jazycích
- Gap closure — formální verifikace Phase 16 (TOOL-01..06), i18n bug fixy, hardcoded string elimination

**Stats:** 133 files changed, +12,405/-9,193 lines (net +3,212), 74 commits
**LOC:** 58,187 Rust
**Requirements:** 20/20 satisfied
**Audit:** gaps_found -> closed by Phase 18

---

## v1.1.0 Sandbox Removal (Shipped: 2026-03-06)

**Phases completed:** 4 phases, 8 plans, 0 tasks

**Key accomplishments:**
- Kompletní odstranění sandbox.rs modulu a všech sandbox datových struktur (Sandbox, SyncPlan)
- Vyčištění UI — settings toggle, modální dialogy, build bar label, file tree sandbox prvky
- Odstranění sandbox logiky z file operations, watcheru a git/build guardů
- Přejmenování sandbox_root → project_root, exec_in_sandbox → exec v plugin systému
- Vyčištění 43+ sandbox i18n klíčů ze všech 5 jazyků (cs, en, de, ru, sk)
- Zero compile warnings, 57 passing testů, plná funkčnost editoru

**Stats:** 56 files changed, -2,878 net lines (196+, 3,074-), 15 feat/fix commits
**Timeline:** 18 days (2026-02-16 → 2026-03-06)
**LOC:** 25,552 Rust
**Requirements:** 26/26 satisfied
**Audit:** passed (26/26 requirements, 4/4 phases, 6/6 E2E flows)

---

## v1.0.6 Focus Management (Shipped: 2026-03-05)

**Phases completed:** 1 phase (of 3 planned), 1 plan, 3 commits
**Phases cancelled:** 2 (covered by Phase 6)

**Key accomplishments:**
- Terminal hover-to-focus removed — all 4 Hovered handlers are no-op (docked+float, right+bottom)
- dialog_open guard on all terminal focus paths — modals and AI Chat block terminal focus capture
- Modal overlay backdrop as interactive Area (Order::Middle) blocks interaction behind modal
- close_on_click_outside control — Settings/Plugins only close via Save/Cancel/X buttons
- Settings discard confirmation on unsaved changes

**Requirements:** 9/9 satisfied (all covered by Phase 6)

**Tech debt:**
- No SUMMARY.md/VERIFICATION.md for phase 6
- Warning text kontrast v light mode (carried from v1.0.2)

---

## v1.0.2 Dark/Light Mode (Shipped: 2026-03-05)

**Phases completed:** 5 phases, 17 plans, 42 tasks

**Key accomplishments:**
- Theme model: LightVariant enum (WarmIvory/CoolGray/Sepia), to_egui_visuals(), syntect_theme_name() — dark/light bez bliknutí při startu
- Theme-aware terminály a file tree: explicitní palety light/dark, scrollbar z ui.visuals(), sémantické git barvy
- Tři varianty světlé palety + live picker v Settings, okamžitý live preview, canonical settings.toml persist + legacy migrace
- Sandbox persistence: sandbox mode v settings.toml, discoverable tooltip, čitelná inline poznámka
- Okamžité sandbox apply po Save: restart terminálů, remap tabů, blokace OFF při staged, sync při ON, multi-window propagace

**Tech debt:**
- Nyquist VALIDATION.md: 5 fází ve stavu draft (testy nebyly generovány)
- Warning text kontrast v light mode (nahlášeno při UAT fáze 5)

---
