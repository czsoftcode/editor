# Phase 4: Infrastructure — Research

## Kontekst a cíl
Cíl fáze: základní přepínač sandbox režimu v Settings > Projekt a jeho napojení na UI/terminály. OFF = práce v root projektu, terminály v rootu, sandbox UI prvky skryté, sync vypnutý. ON umožní návrat do sandboxu.

Tato research má podklady pro plán, který pokryje požadavky: SETT-01..05 a TERM-01..03.

## Standard Stack
- **Rust + eframe/egui**: UI je v `src/app/ui/*`, stav a logika v `WorkspaceState`.
- **Settings persistence**: `src/settings.rs` ukládá do `~/.config/polycredo-editor/settings.toml` (TOML). Migrace ze `settings.json` je už implementovaná.
- **Settings UI**: `src/app/ui/workspace/modal_dialogs/settings.rs` (Save/Cancel, draft/snapshot).
- **Workspace init**: `src/app/ui/workspace/state/init.rs` nastavuje `file_tree_in_sandbox` a `build_in_sandbox` z `settings.sandbox_mode`.
- **Terminals**: `src/app/ui/terminal/*` + `Terminal::new(...)` v několika místech (AI i build).
- **I18n**: `locales/*/ui.ftl` + `I18n::get()`.
- **Toasty**: `Toast::info/error` v `src/app/types.rs`.

## Architecture Patterns
- **Settings modal Save/Cancel**: 
  - `settings_draft` a `settings_original` (snapshot) v `WorkspaceState`.
  - Save nastaví `shared.settings = Arc::new(draft)` a bumpne `settings_version`.
  - Cancel revertuje snapshot.
- **Apply settings**: `EditorApp::update()` a `register_deferred_viewports()` aplikují změny pouze při změně `settings_version`.
- **Sandbox runtime flags**:
  - `build_in_sandbox` a `file_tree_in_sandbox` jsou runtime přepínače v `WorkspaceState`.
  - Inicializují se v `init_workspace()` z `settings.sandbox_mode`.
- **Lazy terminal init**:
  - AI terminal se vytváří při otevření panelu (viz `render_workspace()` a `terminal/right/mod.rs`).
  - Build terminal se vytváří při zobrazení bottom panelu.
- **Sync flow**:
  - Sync plán a potvrzení (`ws.sync_confirmation`) vznikají při startu AI agenta, pokud sandbox není čistý.
  - Background watcher auto‑syncuje projekty ↔ sandbox.

## Don’t Hand-Roll
- **Perzistence**: používej `Settings::save()`/`load()` (TOML), neimplementuj vlastní JSON.
- **UI modal**: použij stávající `StandardModal` + Save/Cancel pattern (settings modal už existuje).
- **I18n**: všechny nové texty musí jít přes `locales/*/ui.ftl` (tooltipy, labely, toasty).
- **Toasty**: používej `Toast::info/error`, žádné ad‑hoc pop‑upy.

## Common Pitfalls
- **Neukládá se sandbox_mode**: V `settings` modalu se teď ukládá jen změna tématu (`should_persist_theme_change`). To znamená, že `sandbox_mode` se po Save neuloží. Je nutné upravit perzistenci tak, aby se ukládaly i změny mimo theme.
- **`project_read_only` mismatch**: Kód používá `settings.project_read_only`, ale v `Settings` structu pole není. Požadavek SETT-05 říká „merge Safe Mode do sandbox toggle“ — je potřeba rozhodnout, jak to sjednotit (např. odvodit read‑only z `sandbox_mode`).
- **Okamžitá aplikace vs. „apply on reopen“**: Save v settings rovnou mění `shared.settings`. Pokud UI začne brát hodnotu přímo ze settings, projeví se změna okamžitě (v rozporu s rozhodnutím). Doporučení: UI řídit přes `WorkspaceState` a nový „sandbox_mode_effective“ flag z initu, ne přes živé `shared.settings`.
- **AI terminál root**: AI terminál se vytváří s `ws.sandbox.root` v několika místech (např. `render_workspace()` + `terminal/right/mod.rs` + legacy `ui/ai_panel.rs`). Pro TERM‑01 musí být working dir root, když je sandbox OFF.
- **Build terminál toggle**: `render_build_bar()` vždy nabízí Sandbox ON/OFF. Pokud sandbox OFF (globálně), toggle se musí skrýt/zakázat a zůstat v rootu.
- **Sandbox UI prvky**: file tree toggle, staged bar, sync potvrzení a sandbox git/info prvky jsou renderované vždy. OFF režim má tyto prvky skrýt a synchronizaci vypnout.
- **Chybějící i18n klíče**: `build-terminal-title` není v locale souborech. Pokud se bude měnit label/okno, přidej klíče ve všech jazycích.

## Code Examples
- **Init podle sandbox_mode**:
  - `src/app/ui/workspace/state/init.rs`: `file_tree_in_sandbox` a `build_in_sandbox` se nastavují z `settings.sandbox_mode`, file tree se načítá z rootu vs sandboxu.
- **Settings Save**:
  - `src/app/ui/workspace/modal_dialogs/settings.rs`: Save nastaví `shared.settings` + `settings_version`, ale `save()` volá jen při změně theme.
- **AI terminal working dir**:
  - `src/app/ui/workspace/mod.rs` a `src/app/ui/terminal/right/mod.rs`: `Terminal::new(..., &ws.sandbox.root, ...)`.
- **Build terminal working dir**:
  - `src/app/ui/terminal/bottom/build_bar.rs`: switching mezi `ws.sandbox.root` a `ws.root_path` podle `ws.build_in_sandbox`.

## Validation Architecture
- **Unit test**: doplnit test v `src/settings.rs` pro round‑trip `sandbox_mode` (serialize/deserialize) a ověřit, že `save()` zapisuje změnu (nejen theme).
- **Manual smoke test (GUI)**:
  1. Otevři projekt, otevři Settings > Projekt, přepni „Režim Sandbox“, Save.
  2. Ověř, že `~/.config/polycredo-editor/settings.toml` obsahuje `sandbox_mode = false/true`.
  3. Zavři projekt a znovu otevři: `file_tree_in_sandbox` a `build_in_sandbox` odpovídají uložené hodnotě.
  4. Sandbox OFF: AI terminal i build terminal mají working dir v root projektu; label odpovídá „Terminal + zkrácená cesta“; sandbox UI prvky nejsou vidět; sync dialogy se neobjevují.
  5. Sandbox ON: label „Sandbox“, běžné sandbox UI prvky zpět.
- **Regression checks**:
  - Settings Save/Cancel chování nesmí okamžitě přepnout běžící workspace (změna se projeví až po reopen).
  - Žádný crash v AI panelu při přepnutí režimu / při otevírání nových terminálových tabů.

## Doporučení pro plán
- Explicitně pokrýt SETT‑01..05 a TERM‑01..03 jednou změnovou sadou.
- V plánu vymezit, jak se řeší „Safe Mode merge“ (SETT‑05) a kde bude zdroj pravdy (Settings vs WorkspaceState).
- Počítat s úpravou i18n klíčů pro nové labely/tooltipy/toasty.