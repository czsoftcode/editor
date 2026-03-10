---
phase: 11-file-operations-watcher-guard-removal
plan: 02
subsystem: plugins
tags: [wasm, plugin-registry, ai-tools, refactoring]

# Dependency graph
requires:
  - phase: 09-core-sandbox-logic-settings-removal
    provides: Registry::new takes project root instead of sandbox root
provides:
  - Plugin registry with project_root field (no sandbox_root)
  - AI tool "exec" (renamed from exec_in_sandbox)
  - WASM host function "exec" for gemini/ollama plugins
affects: [ai-agent, plugin-system]

# Tech tracking
tech-stack:
  added: []
  patterns: [project_root naming convention in plugin system]

key-files:
  created: []
  modified:
    - src/app/registry/mod.rs
    - src/app/registry/plugins/mod.rs
    - src/app/registry/plugins/security.rs
    - src/app/registry/plugins/host/fs.rs
    - src/app/registry/plugins/host/sys.rs
    - src/app/registry/plugins/host/search.rs
    - src/app/ai/tools.rs
    - src/app/ai/types.rs
    - src/app/ui/ai_panel.rs
    - src/plugins/gemini/src/lib.rs
    - src/plugins/ollama/src/lib.rs

key-decisions:
  - "Renamed sandbox_root to project_root consistently across entire plugin registry"
  - "Renamed exec_in_sandbox to exec in WASM host function, AI tool declaration, and all WASM plugins"

patterns-established:
  - "project_root: All plugin host state uses project_root for working directory"

requirements-completed: [GIT-01, GIT-02]

# Metrics
duration: 3min
completed: 2026-03-05
---

# Phase 11 Plan 02: Plugin Registry & AI Tools Sandbox Removal Summary

**Renamed sandbox_root to project_root in plugin registry and exec_in_sandbox to exec in AI tools and WASM plugins**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-05T22:27:15Z
- **Completed:** 2026-03-05T22:30:07Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- PluginManager and PluginHostState use project_root instead of sandbox_root
- AI tool renamed from exec_in_sandbox to exec with updated description
- System prompts in all 3 expertise roles updated to reference exec
- Gemini and Ollama WASM plugins updated to use exec host function
- All 57 tests pass, cargo check clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Rename sandbox_root to project_root in plugin registry** - `ca45604` (feat)
2. **Task 2: Rename exec_in_sandbox to exec in AI tools and WASM plugins** - `538b267` (feat)

## Files Created/Modified
- `src/app/registry/mod.rs` - Registry::new parameter renamed
- `src/app/registry/plugins/mod.rs` - PluginManager sandbox_root -> project_root
- `src/app/registry/plugins/security.rs` - HostState sandbox_root -> project_root
- `src/app/registry/plugins/host/fs.rs` - All file operations use project_root
- `src/app/registry/plugins/host/sys.rs` - host_exec_in_sandbox -> host_exec, project_root
- `src/app/registry/plugins/host/search.rs` - Comment updated
- `src/app/ai/tools.rs` - Tool name exec_in_sandbox -> exec
- `src/app/ai/types.rs` - System prompt references updated
- `src/app/ui/ai_panel.rs` - Removed sandbox comment
- `src/plugins/gemini/src/lib.rs` - WASM extern fn and match branch updated
- `src/plugins/ollama/src/lib.rs` - WASM extern fn and match branch updated

## Decisions Made
None - followed plan as specified.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plugin registry and AI tools are fully cleaned of sandbox terminology
- Ready for remaining phases of sandbox removal

---
*Phase: 11-file-operations-watcher-guard-removal*
*Completed: 2026-03-05*
