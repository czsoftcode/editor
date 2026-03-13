# S11: Gsd Core State Engine

**Goal:** Build the custom YAML-like frontmatter parser with full round-trip fidelity.
**Demo:** Build the custom YAML-like frontmatter parser with full round-trip fidelity.

## Must-Haves


## Tasks

- [x] **T01: 20-gsd-core-state-engine 01** `est:7min`
  - Build the custom YAML-like frontmatter parser with full round-trip fidelity.

Purpose: This is the foundational data layer for all GSD commands. Every GSD command that reads or writes `.planning/` markdown files depends on this parser. It must handle the full YAML subset (strings, integers, floats, booleans, lists, nested maps, quoted strings, inline lists/maps) while preserving comments, whitespace, and key ordering for lossless round-trip.

Output: `frontmatter.rs` with FmValue, FmNode, FmDocument types and comprehensive unit tests.
- [x] **T02: 20-gsd-core-state-engine 02** `est:8min`
  - Create the GSD module skeleton with dispatch, config management, and path helpers.

Purpose: Establishes the GSD command entry point in the slash system, wires up subcommand routing, and provides config.json management and path utilities that all GSD commands will use. Runs in parallel with Plan 01 (frontmatter parser) since these have no file overlap.

Output: Working `/gsd`, `/gsd help`, `/gsd config get/set` commands, path helper utilities with tests.
- [x] **T03: 20-gsd-core-state-engine 03** `est:3min`
  - Implement the `/gsd state` and `/gsd progress` slash commands with full read/write capability.

Purpose: Users can query and update GSD project state directly from the chat panel. `/gsd state` shows a detailed overview (milestone, phase, status, progress, velocity, blockers). `/gsd progress` shows a compact progress bar + phase table. `/gsd state update` and `/gsd state patch` modify STATE.md on disk.

Output: Working state.rs with all STATE requirements, wired into GSD dispatch.

## Files Likely Touched

- `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs`
- `src/app/ui/terminal/ai_chat/gsd/mod.rs`
- `src/app/ui/terminal/ai_chat/gsd/config.rs`
- `src/app/ui/terminal/ai_chat/gsd/paths.rs`
- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/terminal/ai_chat/mod.rs`
- `src/app/ui/terminal/ai_chat/gsd/state.rs`
- `src/app/ui/terminal/ai_chat/gsd/mod.rs`
