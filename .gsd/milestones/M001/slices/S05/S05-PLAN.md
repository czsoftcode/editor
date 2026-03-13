# S05: Light Varianty Settings Ui

**Goal:** Dodat konecne mapovani tri light variant do `Settings::to_egui_visuals()` tak, aby kazda varianta mela vlastni citelnou paletu a splnila LITE-01.
**Demo:** Dodat konecne mapovani tri light variant do `Settings::to_egui_visuals()` tak, aby kazda varianta mela vlastni citelnou paletu a splnila LITE-01.

## Must-Haves


## Tasks

- [x] **T01: 03-light-varianty-settings-ui 01** `est:2min`
  - Dodat konecne mapovani tri light variant do `Settings::to_egui_visuals()` tak, aby kazda varianta mela vlastni citelnou paletu a splnila LITE-01..LITE-04.

Purpose: Uzavrit modelovou cast Phase 03 pred UI integraci; vsechny vizualni konstanty drzet centralne v `settings.rs`.
Output: `src/settings.rs` s explicitnim mapovanim variant a unit testy overujicimi barvy + oddeleni panelu.
- [x] **T02: 03-light-varianty-settings-ui 02** `est:6min`
  - Dodat Settings UI pro kartovy vyber light variant a live preview bez restartu, vcetne i18n labelu a guardu proti zbytecnym update stormum.

Purpose: Splnit SETT-01 a SETT-02 na existujicim `settings_draft` flow bez velkeho refaktoru.
Output: `settings.rs` modal UI rozsireny o conditional picker + runtime preview; doplnene locale klice.
- [x] **T03: 03-light-varianty-settings-ui 03** `est:5min`
  - Uzavrit persistence a modal semantiku tak, aby `Save/Cancel` fungovaly korektne i pri live preview, vcetne fingerprint diff guardu a jasneho storage kontraktu.

Purpose: Dokoncit SETT-03 bez rozsirovani scope mimo settings lifecycle.
Output: Snapshot-aware Settings modal, cleanup v global discard flow, Save-only-on-theme-change guard, a testy canonical/legacy persistence kontraktu.
- [x] **T04: 03-light-varianty-settings-ui 04** `est:15min`
  - Doplnit lock na jemne prizpusobeni terminalu a git barev podle zvolene light varianty, bez rozsirovani scope mimo existujici render pipeline.

Purpose: Dokoncit light-variant polish tak, aby se tonalita light variant propsala i do casti dorucenych ve Phase 2.
Output: Variant-aware tone adaptace v terminal a file tree git barvach s regresnimi testy citelnosti.
- [x] **T05: 03-light-varianty-settings-ui 05** `est:5min`
  - Opravit dva UAT gaby z fáze 03: (1) karta picker zobrazuje pouze WarmIvory místo tří karet — fix `ui.with_layout(right_to_left)` uvnitř horizontálního layoutu; (2) terminál v WarmIvory je příliš světlý a chladný — fix base background barvy nebo blend ratio.

Purpose: UAT 2 a 6 reportovaly reálné vizuální defekty. Obě opravy jsou malé, lokalizované a nezávislé.
Output: Tři karty viditelné vedle sebe v Settings, terminál s teplým krémovým tónem.

## Files Likely Touched

- `src/settings.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `locales/de/ui.ftl`
- `locales/sk/ui.ftl`
- `locales/ru/ui.ftl`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/modal_dialogs.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/settings.rs`
- `src/app/ui/terminal/instance/theme.rs`
- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/git_status.rs`
- `src/app/ui/file_tree/render.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/terminal/instance/theme.rs`
