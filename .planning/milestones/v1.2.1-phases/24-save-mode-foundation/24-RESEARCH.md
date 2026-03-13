# Phase 24 Research: Save Mode Foundation

## Objective
Naplánovat implementaci základů ukládání pro požadavky `SAVE-01`, `SAVE-02`, `SAVE-03`, `MODE-01`, `MODE-02`, `MODE-03` tak, aby:
- `Ctrl+S` konzistentně ukládal aktivní tab.
- Režim ukládání (Automatic/Manual) šel přepnout v Settings, perzistoval a runtime se okamžitě aplikoval po `Save`.
- Chyby ukládání byly vždy viditelné v UI (toast), bez tichého selhání.
- Idle CPU zůstalo nízké (žádné nové polling smyčky, jen gate na existující autosave flow).

## Current Baseline (What Already Exists)
- `Ctrl+S` už volá `ws.editor.save(...)` v `src/app/ui/workspace/mod.rs`.
- Save API vrací `Option<String>` s chybou a nastavuje `SaveStatus` (`Saving`, `Saved`, `Modified`) v `src/app/ui/editor/files.rs`.
- Background autosave běží v `process_background_events` přes `ws.editor.try_autosave(...)` v `src/app/ui/background.rs`.
- Settings mají draft + Save/Cancel lifecycle a runtime apply přes `AppShared.settings` + `settings_version` v `src/app/ui/workspace/modal_dialogs/settings.rs`.
- Persist settings existuje přes `Settings::try_save()` do TOML (`src/settings.rs`).
- Toast mechanika existuje (`ws.toasts.push(Toast::error(...))`).

## Requirement Mapping
- `SAVE-01`: Primárně hotovo, ale je potřeba explicitně sjednotit semantiku mezi menu Save a `Ctrl+S` (stejný handler + stejné toast chování).
- `SAVE-02`: Save status se přepíná už teď; doplnit explicitní info feedback pro případ „soubor už je uložen“.
- `SAVE-03`: Chyby z `save()`/`try_autosave()` už propagují string; je nutné sjednotit deduplikaci, aby error toasty nespamovaly.
- `MODE-01`: Chybí datový model save režimu v `Settings` + UI přepínač.
- `MODE-02`: Jakmile bude save režim pole v `Settings` se serde default, persist je pokryt existujícím `try_save()`.
- `MODE-03`: Runtime apply je přirozeně řešitelný přes existující Save flow v Settings (přepsání `shared.settings` + `settings_version`).

## Standard Stack
Použít stávající stack a vzory projektu, bez nové dependency:
- `serde` (`Serialize/Deserialize`) pro nový enum save režimu.
- `toml` persist přes existující `Settings::try_save()`.
- `egui` radio controls v Settings modalu.
- Existující background tick (`process_background_events`) pro autosave gating.
- Existující i18n (`locales/*/*.ftl`) pro nové texty/hlášky.

## Architecture Patterns
1. **Single source of truth pro save mode:**
- Přidat `SaveMode` enum do `src/settings.rs`.
- Uložit jako pole v `Settings` se safe defaultem `Manual`.

2. **Settings draft pattern bez live persist:**
- V `modal_dialogs/settings.rs` editovat pouze draft.
- Režim aplikovat až při `Save` (už existující pattern).

3. **Runtime behavior gate v background loopu:**
- V `src/app/ui/background.rs` volat `try_autosave()` pouze pokud `shared.settings.save_mode == Automatic`.
- V `Manual` režimu autosave path přeskočit.

4. **Unified manual save action:**
- `Ctrl+S` i menu Save musí volat stejnou helper funkci (např. `perform_manual_save(...)`).
- Helper rozhodne toast policy (`error`, `already saved`, případně `saved` nechat jen ve status baru).

5. **Feedback policy:**
- Error: vždy `Toast::error` se jménem souboru + důvodem (už je připraveno přes `error-file-save`).
- No-op save (soubor není modified): `Toast::info` („soubor už je uložen“), deduplikovaný v krátkém okně.

## Don't Hand-Roll
- Nepřidávat nový persist mechanismus mimo existující `Settings` TOML flow.
- Nepřidávat nový scheduler/timer thread pro autosave mód; použít stávající background event loop.
- Nestavět vlastní keybinding vrstvu; držet se existujícího input zpracování v `workspace/mod.rs`.
- Nedělat globální refaktor editor save pipeline; stačí minimální integrační patch.

## Common Pitfalls
- Chybějící `serde(default)` u nového pole může rozbít load starších settings.
- Přímé čtení `shared.settings` na více místech bez helperu může vést k nekonzistentnímu chování (menu vs shortcut).
- Toast spam při opakovaných autosave chybách (nutná deduplikace).
- Zapomenuté i18n klíče ve všech jazycích rozbijí test parity klíčů.
- Uložení v modal kontextu: `Ctrl+S` nesmí ukládat editor file, když fokus patří settings modalu.

## Implementation Blueprint (for planning)
1. **Data model (MODE-01/02)**
- `src/settings.rs`:
  - Přidat `enum SaveMode { Automatic, Manual }` s `#[serde(rename_all = "snake_case")]`.
  - Přidat default funkci vracející `SaveMode::Manual`.
  - Přidat pole `save_mode` do `Settings` + `Default` impl.

2. **Settings UI (MODE-01/03)**
- `src/app/ui/workspace/modal_dialogs/settings.rs`:
  - Do Editor (nebo General) sekce přidat radio: `Automatic Save` / `Manual Save`.
  - Ujistit se, že změna je jen v draftu a promítne se až při `Save`.
  - Po úspěšném `draft.try_save()` přidat info toast o změně režimu.

3. **Autosave gate (MODE-03)**
- `src/app/ui/background.rs`:
  - Před `try_autosave()` načíst aktivní režim ze `shared.settings`.
  - V `Manual` režimu autosave nevolat.

4. **Manual save behavior (SAVE-01/02/03)**
- `src/app/ui/workspace/mod.rs` + menubar save action:
  - Zavést jednotný helper pro ruční save aktivního tabu.
  - Pokud tab není modified: vrátit info toast „already saved“.
  - Pokud save selže: error toast.

5. **Localization + UX copy**
- Přidat nové i18n klíče do `locales/cs|en|de|ru|sk/*.ftl`:
  - labely save mode
  - info message „file already saved“
  - optional: „save mode changed“ toast

6. **Tests and verification hooks**
- `settings.rs` unit test: nový `save_mode` se serializuje/deserializuje, default je `manual`.
- Pokud existují UI helper testy: ověřit, že změna draftu se projeví až po Save.
- Smoke test manuálně: restart app, režim přetrvá.

## Code Examples (target patterns)

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum SaveMode {
    Automatic,
    #[default]
    Manual,
}
```

```rust
let save_mode = {
    let s = shared.lock().expect("lock");
    s.settings.save_mode
};
if save_mode == crate::settings::SaveMode::Automatic {
    if let Some(err) = ws.editor.try_autosave(i18n, &shared.lock().expect("lock").is_internal_save) {
        // toast policy
    }
}
```

```rust
fn perform_manual_save(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>, i18n: &I18n) {
    let internal_save = Arc::clone(&shared.lock().expect("lock").is_internal_save);
    if ws.editor.active().is_some_and(|t| !t.modified) {
        ws.toasts.push(Toast::info(i18n.get("info-file-already-saved")));
        return;
    }
    if let Some(err) = ws.editor.save(i18n, &internal_save) {
        ws.toasts.push(Toast::error(err));
    }
}
```

## Validation Architecture
Nyquist-ready validační strategie pro plán fáze:

- **Requirement-to-check matice**
  - `SAVE-01`: integrační check: `Ctrl+S` uloží aktivní tab (status přechod `Modified -> Saved`).
  - `SAVE-02`: bez focus change se po `Ctrl+S` okamžitě změní save stav a obsah je na disku.
  - `SAVE-03`: simulace I/O failu -> error toast viditelný, tab zůstává `Modified`.
  - `MODE-01`: settings UI obsahuje přepínač Auto/Manual.
  - `MODE-02`: po restartu app zůstává vybraný režim.
  - `MODE-03`: po kliknutí `Save` v settings se autosave chování změní bez restartu.

- **Test layers**
  - Unit: `Settings` serde/default pro `save_mode`.
  - Integration-lite (workspace/editor): manual save helper (success/fail/no-op).
  - Manual UAT: přepnutí režimu + editace + čekání na debounce v obou módech.

- **Failure injection points**
  - Soubor read-only / write denied pro `SAVE-03`.
  - Corrupted/legacy settings bez `save_mode` pole (musí spadnout na default `Manual`).

- **Observability**
  - Toast logika: ověřit, že opakované stejné chyby nejsou spamované v krátkém okně.
  - Status bar: validovat konzistenci `Unsaved/Saving/Saved` proti interním flagům tabu.

## Open Questions to Resolve During Planning
- Kde přesně UI umístit save mode toggle (General vs Editor kategorie) bez narušení UX.
- Přesná deduplikační politika pro toasty (časové okno, klíč dedupu).
- Zda `Ctrl+S` při otevřeném settings modalu má explicitně ukládat settings draft (dle kontextu fáze ano), nebo být no-op.

## Recommended Plan Shape
- Plan A: Data model + persist (`settings.rs` + testy).
- Plan B: Settings UI + runtime apply + i18n.
- Plan C: Autosave gate + unified manual save handler + toast policy.
- Plan D: Validace (`cargo check`, `./check.sh`, cílené testy + manuální UAT skript).

## Confidence
- Vysoká: architektonický směr, integrační body, persist/runtime apply pattern.
- Střední: přesné UX detaily (umístění toggle, wording toastů, dedupe okno) vyžadují krátké produktové rozhodnutí při plánování.

## RESEARCH COMPLETE
Research soubor definuje: mapování SAVE/MODE požadavků na konkrétní kódové body, doporučené architektonické vzory, co nehandrollovat, hlavní rizika, implementační blueprint po krocích, ukázky cílových patternů a Nyquist-ready sekci `Validation Architecture` s testovací maticí a failure injection body.
