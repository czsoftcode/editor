# Deferred Items (25-08)

- `./check.sh` fails at `cargo fmt --check` due to pre-existing out-of-scope formatting drift in unrelated files (`src/app/cli/*`, `src/settings.rs`, several `src/app/ui/widgets/*` and other already-dirty workspace files). Not auto-fixed in plan `25-08` to avoid scope creep and touching user worktree changes.
- `./check.sh` in plan `25-10` still fails at `cargo fmt --check` for the same pre-existing formatting drift outside the modified guard files. Left deferred to avoid non-plan mass reformat and unrelated file churn.
- `./check.sh` in plan `25-09` fails at `cargo fmt --check` due to the same pre-existing formatting drift in unrelated files; verification of plan scope (`unsaved guard` files/tests) passed, but global fmt gate remains deferred to avoid unrelated mass edits.
