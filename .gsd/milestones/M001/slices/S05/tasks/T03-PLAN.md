# T03: 03-light-varianty-settings-ui 03

**Slice:** S05 — **Milestone:** M001

## Description

Uzavrit persistence a modal semantiku tak, aby `Save/Cancel` fungovaly korektne i pri live preview, vcetne fingerprint diff guardu a jasneho storage kontraktu.

Purpose: Dokoncit SETT-03 bez rozsirovani scope mimo settings lifecycle.
Output: Snapshot-aware Settings modal, cleanup v global discard flow, Save-only-on-theme-change guard, a testy canonical/legacy persistence kontraktu.

## Must-Haves

- [ ] "Pri otevreni Settings se ulozi puvodni snapshot; `Cancel` vrati runtime settings na snapshot a zahodi draft zmeny."
- [ ] "`Save` porovnava theme fingerprint `(dark_theme, light_variant)` mezi snapshotem a draftem; disk persist do `settings.toml` probehne jen pri realne zmene fingerprintu."
- [ ] "Storage kontrakt je explicitni: canonical soubor je `settings.toml`, `settings.json` je pouze legacy kompatibilni vstup pro migraci na TOML."
- [ ] "Globalni confirm-discard flow pro zavreni modalu uklidi `settings_draft` i `settings_original`, aby nezustal leak preview stavu."
- [ ] "Persistence cesta je testovana: roundtrip save/load v `settings.toml` + legacy migrace `settings.json -> settings.toml` zachova variantu i rezim."

## Files

- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/modal_dialogs.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/settings.rs`
