# Deferred Items - Phase 29

## 2026-03-10

- `./check.sh` selhává na `cargo fmt` driftu mimo scope plánu:
  - `src/app/ui/git_status.rs`
  - `src/app/ui/workspace/modal_dialogs/settings.rs`
- Existující warningy mimo scope plánu:
  - `src/app/cli/executor.rs` (`unused variable: i`)
  - `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs` (`unused variable: warnings`)
  - `src/app/mod.rs` (`unused variable: ctx`)
