# 32-01 Verification Evidence

Datum (UTC): 2026-03-11

## STAB-01 hard gate

- `RUSTC_WRAPPER= cargo check` -> PASS
- `RUSTC_WRAPPER= ./check.sh` -> PASS
- `! rg -n "crate::app::cli|app::cli" src/app/ui/terminal src/app/ai_core` -> PASS

## STAB-02 targeted runtime checks

- `RUSTC_WRAPPER= cargo test phase32_namespace_guard -- --nocapture` -> PASS
- `RUSTC_WRAPPER= cargo test phase32_runtime_stability -- --nocapture` -> PASS
- `RUSTC_WRAPPER= cargo test slash::tests -- --nocapture` -> PASS
- `RUSTC_WRAPPER= cargo test approval -- --nocapture` -> PASS
- `RUSTC_WRAPPER= cargo test background::tests -- --nocapture` -> PASS
