# T03: 20-gsd-core-state-engine 03

**Slice:** S11 — **Milestone:** M001

## Description

Implement the `/gsd state` and `/gsd progress` slash commands with full read/write capability.

Purpose: Users can query and update GSD project state directly from the chat panel. `/gsd state` shows a detailed overview (milestone, phase, status, progress, velocity, blockers). `/gsd progress` shows a compact progress bar + phase table. `/gsd state update` and `/gsd state patch` modify STATE.md on disk.

Output: Working state.rs with all STATE requirements, wired into GSD dispatch.

## Must-Haves

- [ ] "User runs /gsd state and sees milestone, phase, status, progress bar, velocity, blockers formatted as markdown"
- [ ] "User runs /gsd state update status executing and STATE.md frontmatter is updated on disk"
- [ ] "User runs /gsd state patch and multiple fields are batch-updated in STATE.md"
- [ ] "User runs /gsd progress and sees progress bar + phase table with completion status"
- [ ] "State module can append decisions, blockers, metrics to STATE.md body sections"

## Files

- `src/app/ui/terminal/ai_chat/gsd/state.rs`
- `src/app/ui/terminal/ai_chat/gsd/mod.rs`
