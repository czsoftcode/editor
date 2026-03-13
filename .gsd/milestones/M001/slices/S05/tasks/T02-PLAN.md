# T02: 03-light-varianty-settings-ui 02

**Slice:** S05 — **Milestone:** M001

## Description

Dodat Settings UI pro kartovy vyber light variant a live preview bez restartu, vcetne i18n labelu a guardu proti zbytecnym update stormum.

Purpose: Splnit SETT-01 a SETT-02 na existujicim `settings_draft` flow bez velkeho refaktoru.
Output: `settings.rs` modal UI rozsireny o conditional picker + runtime preview; doplnene locale klice.

## Must-Haves

- [ ] "Settings modal zobrazi volic light variant pouze kdyz `draft.dark_theme == false`; v dark mode je picker skryty."
- [ ] "Picker je kartovy: nabizi presne 3 volby (WarmIvory, CoolGray, Sepia), kazda karta obsahuje barevny swatch + lokalizovany nazev + jasny selected state (ramecek + check)."
- [ ] "Zmena dark/light nebo light varianty se projevi okamzite (live preview) pres update `AppShared.settings` + `settings_version` bez restartu aplikace."
- [ ] "Live preview nepumpuje version kazdy frame; update bezi jen na realne zmene fingerprintu `(dark_theme, light_variant)`."
- [ ] "Sekvence `Light -> Dark -> Light` zachova naposledy zvolenou light variantu (bez resetu na default)."

## Files

- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `locales/de/ui.ftl`
- `locales/sk/ui.ftl`
- `locales/ru/ui.ftl`
