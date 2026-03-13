# T05: 03-light-varianty-settings-ui 05

**Slice:** S05 — **Milestone:** M001

## Description

Opravit dva UAT gaby z fáze 03: (1) karta picker zobrazuje pouze WarmIvory místo tří karet — fix `ui.with_layout(right_to_left)` uvnitř horizontálního layoutu; (2) terminál v WarmIvory je příliš světlý a chladný — fix base background barvy nebo blend ratio.

Purpose: UAT 2 a 6 reportovaly reálné vizuální defekty. Obě opravy jsou malé, lokalizované a nezávislé.
Output: Tři karty viditelné vedle sebe v Settings, terminál s teplým krémovým tónem.

## Must-Haves

- [ ] "V Settings modalu (light mode) jsou vidět 3 klikatelné karty variant: WarmIvory, CoolGray, Sepia"
- [ ] "Terminál v WarmIvory light mode má teplý krémový tón pozadí"

## Files

- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/terminal/instance/theme.rs`
