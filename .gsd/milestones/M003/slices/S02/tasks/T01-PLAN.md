---
estimated_steps: 6
estimated_files: 7
---

# T01: Tlačítko Obnovit, confirm dialog, restore logika a i18n

**Slice:** S02 — Obnovení historické verze s potvrzením a i18n
**Milestone:** M003

## Description

Přidá kompletní restore flow do history split view: tlačítko "Obnovit" v toolbaru, potvrzovací dialog přes existující `show_modal()`, restore logiku v workspace/mod.rs (zápis historického obsahu do tab + synchronní snapshot + refresh entries), a i18n klíče pro všech 5 jazyků.

Klíčový constraint: `render_history_split_view()` dostává `&LocalHistory` (immutable) — `take_snapshot(&mut self)` se musí volat v workspace/mod.rs, kde je `&mut ws.local_history`. Signalizace přes rozšířený `HistorySplitResult.restore_confirmed`.

## Steps

1. **Rozšířit datový model:**
   - `HistoryViewState`: přidat `pub show_restore_confirm: bool` (default false).
   - `HistorySplitResult`: přidat `pub restore_confirmed: bool` (default false).
   - Inicializovat nové fieldy v `render_history_split_view()` result a v workspace/mod.rs při vytváření HistoryViewState.

2. **Přidat tlačítko "Obnovit" do toolbaru:**
   - V `render_history_split_view()`, v `ui.with_layout(right_to_left)` bloku, PŘED navigační šipky.
   - `ui.add_enabled(history_view.selected_index.is_some(), Button::new(i18n.get("history-restore-btn")))`.
   - Klik → `history_view.show_restore_confirm = true`.
   - Přidat `ui.add_space(8.0)` mezi tlačítko a šipky pro vizuální separaci.

3. **Confirm dialog v render funkci:**
   - Import `show_modal` a `ModalResult` z `widgets::modal`.
   - Když `history_view.show_restore_confirm` je true, volat `show_modal(ui.ctx(), "history_restore_confirm", title, ok_label, cancel_label, |ui| { label text; Some(()) })`.
   - `ModalResult::Confirmed(())` → `result.restore_confirmed = true; history_view.show_restore_confirm = false`.
   - `ModalResult::Cancelled` → `history_view.show_restore_confirm = false`.
   - `ModalResult::Pending` → nic.

4. **Restore handling v workspace/mod.rs:**
   - Po stávajícím `if hv_result.content_changed { ... }` bloku, přidat `if hv_result.restore_confirmed { ... }`.
   - Extrahovat do locals: `selected_index`, `relative_path` (clone), entry (clone) z `ws.history_view`.
   - `let historical_content = ws.local_history.get_snapshot_content(&relative_path, &entry)?` — handle Result (při chybě toast nebo log, neprovádět restore).
   - Zapsat `historical_content` do `tab.content`, nastavit `tab.modified = true`, `tab.last_edit`, `tab.save_status = Modified`.
   - `ws.local_history.take_snapshot(&relative_path, &historical_content)` — append nový snapshot (deduplikace handled interně).
   - `let new_entries = ws.local_history.get_history(&relative_path)` — refresh.
   - Update `ws.history_view`: `hv.entries = new_entries`, `hv.selected_index = Some(0)`, `hv.current_content = historical_content`, `hv.content_hash = 0` (vynutí diff přepočet), `hv.diff_for_index = None`, `hv.cached_diff = None`.
   - Borrow checker: extrahovat data z history_view do locals PŘED mutable operacemi na local_history a tabs.

5. **i18n klíče:**
   - Přidat do `locales/XX/ui.ftl` pro cs, en, sk, de, ru:
     - `history-restore-btn` — label tlačítka ("Obnovit", "Restore", ...)
     - `history-restore-confirm-title` — titulek dialogu
     - `history-restore-confirm-text` — text potvrzení
     - `history-restore-confirm-ok` — ok tlačítko
     - `history-restore-confirm-cancel` — cancel tlačítko

6. **Verifikace:**
   - `cargo check` — kompilace
   - `cargo test` — existující testy (žádné nové unit testy nutné — restore flow je UI+IO integrace, testované přes stávající take_snapshot/get_history testy)
   - `./check.sh` — fmt + clippy + testy
   - `grep 'history-restore' locales/*/ui.ftl` — 5 jazyků × 5 klíčů

## Must-Haves

- [ ] `HistorySplitResult.restore_confirmed` signalizuje potvrzení restore z UI
- [ ] Tlačítko "Obnovit" disabled pokud není vybraná žádná verze
- [ ] `show_modal()` confirm dialog před restore
- [ ] Restore v workspace/mod.rs: zápis do tab + take_snapshot + get_history refresh
- [ ] Po restore: selected_index=Some(0), diff cache invalidována, current_content aktualizován
- [ ] Mezilehlé verze zachovány (append, ne replace)
- [ ] i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru)
- [ ] `cargo check` + `cargo test` + `./check.sh` prochází

## Verification

- `cargo check` bez chyb
- `cargo test` — všechny testy pass
- `./check.sh` — fmt + clippy + testy čisté
- `grep -c 'history-restore' locales/*/ui.ftl` — každý soubor >= 4 klíče

## Inputs

- `src/app/ui/workspace/history/mod.rs` — HistoryViewState, HistorySplitResult, toolbar layout (S01 output)
- `src/app/ui/workspace/mod.rs` — post-render handling hv_result (S01 output, řádky ~773-800)
- `src/app/ui/widgets/modal.rs` — `show_modal()` signatura a `ModalResult` enum
- `src/app/local_history.rs` — `take_snapshot()`, `get_history()`, `get_snapshot_content()` API
- `locales/*/ui.ftl` — existující history- klíče jako vzor
- S01-SUMMARY.md Forward Intelligence — HistorySplitResult potřebuje restore_confirmed, borrow checker řešení extrakcí do locals

## Expected Output

- `src/app/ui/workspace/history/mod.rs` — rozšířené struktury, tlačítko v toolbaru, confirm dialog
- `src/app/ui/workspace/mod.rs` — restore handling po render
- `locales/{cs,en,sk,de,ru}/ui.ftl` — nové history-restore-* klíče
