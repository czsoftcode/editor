status: gaps_found

# Phase 35 Verification

## Scope
- Faze: `35-trash-foundation-async-safety`
- Cil: `Zalozit technicky zaklad pro trash workflow bez blokovani UI.`
- Pozadavky: `TRASH-03`, `RELIAB-01`
- Overene artefakty:
  - `.planning/phases/35-trash-foundation-async-safety/35-01-PLAN.md`
  - `.planning/phases/35-trash-foundation-async-safety/35-02-PLAN.md`
  - `.planning/phases/35-trash-foundation-async-safety/35-01-SUMMARY.md`
  - `.planning/phases/35-trash-foundation-async-safety/35-02-SUMMARY.md`
  - `src/app/project_config.rs`
  - `src/app/trash.rs`
  - `src/app/ui/file_tree/dialogs.rs`
  - `src/app/ui/file_tree/mod.rs`
  - `tests/phase35_trash_path.rs`
  - `tests/phase35_async_delete.rs`
  - `tests/phase35_delete_foundation.rs`
  - `.planning/REQUIREMENTS.md`
  - `.planning/ROADMAP.md`

## Evidence Map

### TRASH-03 -> `.polycredo/trash` se vytvari automaticky
- `src/app/project_config.rs`: `trash_dir_path(project_root)` vraci deterministickou cestu `.polycredo/trash`.
- `src/app/trash.rs`: `ensure_trash_dir(project_root)` vola `std::fs::create_dir_all(...)` a je volano z `move_path_to_trash(...)`.
- `src/app/ui/file_tree/dialogs.rs`: delete flow routuje operaci pres `move_path_to_trash(&root, &path)`.
- `tests/phase35_trash_path.rs`: kontroluje existenci helperu `trash_dir_path` a metadata kontraktu.

Verdikt: `TRASH-03` je pro delete tok pokryty a dohledatelny v kodu.

### RELIAB-01 -> I/O delete/restore neblokuje UI vlakno
- `src/app/ui/file_tree/dialogs.rs`: delete operace je spoustena pres `spawn_task(move || ...)`.
- `src/app/ui/file_tree/mod.rs`: vysledek je cten neblokujicim `try_recv()` a chyby jdou pres `pending_error` do toast pipeline.
- `tests/phase35_async_delete.rs`: hlida, ze delete tok pouziva `spawn_task`.
- `tests/phase35_delete_foundation.rs`: hlida fail-closed error propagation (`DeleteJobResult::Error` -> `pending_error`).

Verdikt: async/non-blocking chovani je pro delete tok pokryte.

## Gaps
- `RELIAB-01` i phase-goal text mluvi o delete/restore operacich, ale v dodanem scope faze 35 neni implementovan ani testovan restore tok. Realne dukazy pokryvaji pouze delete cestu.
- `TRASH-03` v roadmape uvadi "pri prvni delete/restore operaci"; v teto fazi je dokazatelne on-demand vytvoreni trash pouze pri delete operaci.

## Gate Evidence (aktualni beh)
- `cargo test phase35 -- --nocapture` -> PASS (3/3 phase35 testy)
- `cargo check` -> PASS
- `./check.sh` -> PASS

## Finalni zaver
- Castecne splneno: technicky zaklad pro async delete + fail-closed trash foundation je hotovy.
- Neuplne proti presnemu zneni pozadavku: chybi restore cast (implementace a verifikace), proto `status: gaps_found`.
