# T02: 20-gsd-core-state-engine 02

**Slice:** S11 — **Milestone:** M001

## Description

Create the GSD module skeleton with dispatch, config management, and path helpers.

Purpose: Establishes the GSD command entry point in the slash system, wires up subcommand routing, and provides config.json management and path utilities that all GSD commands will use. Runs in parallel with Plan 01 (frontmatter parser) since these have no file overlap.

Output: Working `/gsd`, `/gsd help`, `/gsd config get/set` commands, path helper utilities with tests.

## Must-Haves

- [ ] "User types /gsd and sees GSD help with list of subcommands"
- [ ] "User types /gsd config get <key> and sees the config value from .planning/config.json"
- [ ] "User types /gsd config set <key> <value> and config.json is updated on disk"
- [ ] "When .planning/ directory is missing, GSD commands show friendly message instead of crashing"
- [ ] "Path helpers resolve phase directories, generate slugs, and handle decimal phase numbering"

## Files

- `src/app/ui/terminal/ai_chat/gsd/mod.rs`
- `src/app/ui/terminal/ai_chat/gsd/config.rs`
- `src/app/ui/terminal/ai_chat/gsd/paths.rs`
- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/terminal/ai_chat/mod.rs`
