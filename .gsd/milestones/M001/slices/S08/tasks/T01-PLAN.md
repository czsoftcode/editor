# T01: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu 01

**Slice:** S08 — **Milestone:** M001

## Description

Zavést okamžité apply sandbox režimu po Save v Settings včetně potvrzení při OFF, správného pořadí persist → runtime apply, a bezpečné propagace do všech oken stejného projektu s korektními toasty a možností odložení.

## Must-Haves

- [ ] "Změna sandbox režimu se aplikuje až po Save, Cancel vrací původní režim a runtime změny jsou revertované."
- [ ] "Po Save se režim nejdřív persistuje na disk, teprve poté se spustí runtime přepnutí."
- [ ] "Při změně režimu je změna propagovaná do všech oken stejného projektu."
- [ ] "OFF režim vyžaduje potvrzení; při otevřeném jiném dialogu je nabídnuto odložit/udělat hned."
- [ ] "Selhání persistu zobrazí toast s volbou revert / ponechat dočasně."

## Files

- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/mod.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/workspace/modal_dialogs/conflict.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
