# T02: 19-slash-command-infrastructure 02

**Slice:** S10 — **Milestone:** M001

## Description

Implement the three async/stateful slash commands: /model (list/switch), /git (background diff --stat), and /build (background cargo build with in-place message update).

Purpose: These commands require either state access (model list) or background thread execution (git, build) and build on the dispatch infrastructure from Plan 01.
Output: All 7 built-in slash commands fully functional.

## Must-Haves

- [ ] "User types /model and sees a list of available models with active model marked by *"
- [ ] "User types /model <name> and the active model switches with confirmation in chat"
- [ ] "User types /model <invalid> and sees an error with suggestion of closest match"
- [ ] "User types /git and sees git diff --stat output with branch name in a code block"
- [ ] "User types /build and sees 'Building...' immediately, then result updates in-place"
- [ ] "/build and /git do not freeze the UI (run on background threads)"

## Files

- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/background.rs`
