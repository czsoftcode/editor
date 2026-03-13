# Phase 3: Light varianty + Settings UI - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Faze 3 dorucuje tri varianty light palety (WarmIvory, CoolGray, Sepia) v Settings panelu a jejich okamzitou aplikaci napric aplikaci bez restartu. Scope zahrnuje UI picker variant, live preview, persistenci do `settings.toml` a vizualni odliseni panelu pomoci `faint_bg_color`.

Mimo scope:
- Nove capability mimo light varianty (napr. auto-detect OS theme, custom theme editor)
- Velke zmeny terminal/file-tree feature setu mimo theme ladeni

</domain>

<decisions>
## Implementation Decisions

### Live Preview behavior
- Kliknuti na variantu se projevi okamzite ve vsech otevrenych oknech (root i secondary viewporty).
- `Cancel` vraci puvodni stav pred otevrenim Settings (zahozene draft zmeny).
- Pri prepnuti `Light -> Dark -> Light` se drzi naposledy zvolena light varianta (bez resetu na WarmIvory).
- `Save` ma ukladat zmeny jen pri zmene varianty (ostatni settings mohou mit oddelenou persist strategii).
- Storage kontrakt je: canonical persist do `settings.toml`; `settings.json` je legacy kompatibilni vstup pouze pro migraci.

### Settings picker UX
- Picker variant bude kartovy (3 barevne karty se vzorkem + nazvem).
- Picker se zobrazuje pouze v light mode.
- Nazvy variant budou lokalizovane (napr. Tepla slonova kost, Studena seda, Sepiova) pres i18n.
- Vybrana karta bude mit jasny selected stav: ramecek + check.

### Light palette and panel contrast
- Rozdil mezi 3 variantami ma byt stredne vyrazny (jasne rozpoznatelny, ale nerusive profesionalni).
- Panely (editor/file tree/side panel) se maji odlisit jemne, ale citelne pomoci `faint_bg_color`.
- Varianta Sepia ma byt teplejsi nez ostatni, ale stale ciselne citelna pro dlouhou praci.
- Terminal a git status barvy v light mode se maji lehce prizpusobit zvolene variante (ne uplne oddelene sady).

### Claude's Discretion
- Presne RGB/alpha hodnoty pro kazdou variantu a `faint_bg_color` mapu.
- Detaily hover/pressed/focus stavu variant karet (pokud zustane vyse rozhodnuta UX semantika).
- Detailni pravidla, jak oddelit persistenci "theme-only" zmen od ostatnich Settings pri `Save`.

</decisions>

<specifics>
## Specific Ideas

- Uzivatel chce, aby live preview bylo konzistentni mezi vsemi okny, ne jen v aktivnim view.
- `Cancel` ma skutecne fungovat jako revert, ne jen zavreni modalu.
- Dulezity je vizualni rozdil variant bez agresivniho tonovani UI.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/settings.rs`: `LightVariant` enum uz existuje (`WarmIvory`, `CoolGray`, `Sepia`) + `to_egui_visuals()` a `apply()`.
- `src/app/ui/workspace/modal_dialogs/settings.rs`: existujici Settings modal s draft modelem, `Save/Cancel`, kategorii `editor` a dark/light radiem.
- `src/app/mod.rs`: aplikace settings pres `settings_version` do root i deferred viewportu (vhodny integracni bod pro live preview).

### Established Patterns
- Theme propagace je centralizovana pres `AppShared.settings` + `settings_version` a `Settings::apply(ctx)`.
- Settings modal pracuje s `ws.settings_draft`; persist je dnes navazana na `Save`.
- i18n stringy pro settings jsou v `locales/*/ui.ftl`; pro variant picker bude potreba pridat nove klice.

### Integration Points
- UI picker: `src/app/ui/workspace/modal_dialogs/settings.rs` (sekce `selected_cat == "editor"`).
- Theme model: `src/settings.rs` (`to_egui_visuals`, mapovani light variant na visuals/panel barvy).
- Runtime propagation: `src/app/mod.rs` (root update + deferred viewports) pres `settings_version`.
- Lokalizace popisku variant: `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/de/ui.ftl`, `locales/sk/ui.ftl`, `locales/ru/ui.ftl`.

</code_context>

<deferred>
## Deferred Ideas

- OS auto-detect dark/light preference.
- Uzivatelsky custom theme editor.

</deferred>

---

*Phase: 03-light-varianty-settings-ui*
*Context gathered: 2026-03-04*
