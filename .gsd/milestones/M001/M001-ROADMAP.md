# M001: Migration

**Vision:** Multiplatformni textovy editor v Rustu (eframe/egui) s terminaly, build workflow a AI terminal panelem.

## Success Criteria


## Slices

- [x] **S02: Zaklad** `risk:medium` `depends:[]`
  > After this: Rozšíření Settings struct o LightVariant enum a theme-aware metody.
- [x] **S04: Terminal Git Barvy** `risk:medium` `depends:[S02]`
  > After this: Zavést plně theme-aware rendering terminálu tak, aby v light mode nebyl tmavý background ani tmavý scrollbar, a přepínání tématu fungovalo za běhu bez restartu backendu.
- [x] **S05: Light Varianty Settings Ui** `risk:medium` `depends:[S04]`
  > After this: Dodat konecne mapovani tri light variant do `Settings::to_egui_visuals()` tak, aby kazda varianta mela vlastni citelnou paletu a splnila LITE-01.
- [x] **S07: Infrastructure** `risk:medium` `depends:[S05]`
  > After this: Zavést sandbox režim do Settings (persistovaný do settings.
- [x] **S08: Okam It Aplikov N Zm Ny Re Imu Sandboxu Po P Epnut Checkboxu** `risk:medium` `depends:[S07]`
  > After this: Zavést okamžité apply sandbox režimu po Save v Settings včetně potvrzení při OFF, správného pořadí persist → runtime apply, a bezpečné propagace do všech oken stejného projektu s korektními toasty a možností odložení.
- [x] **S10: Slash Command Infrastructure** `risk:medium` `depends:[S08]`
  > After this: Create the slash command dispatch system with command registry, integrate it into the chat prompt flow, and implement synchronous built-in commands (/help, /clear, /new, /settings).
- [x] **S11: Gsd Core State Engine** `risk:medium` `depends:[S10]`
  > After this: Build the custom YAML-like frontmatter parser with full round-trip fidelity.
