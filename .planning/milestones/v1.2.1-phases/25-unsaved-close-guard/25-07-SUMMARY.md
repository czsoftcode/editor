---
phase: 25-unsaved-close-guard
plan: 07
subsystem: validation
tags: [validation, nyquist, requirements-mapping]

# Dependency graph
requires:
  - phase: 25-01
  - phase: 25-02
  - phase: 25-03
  - phase: 25-04
  - phase: 25-05
  - phase: 25-06
provides:
  - Final GUARD-01..04 validation map and sign-off
affects: [validation, documentation, roadmap]

tech-stack:
  added: []
  patterns: ["Per-requirement mapping to tasks and evidence"]

key-files:
  created: []
  modified:
    - .planning/phases/25-unsaved-close-guard/25-VALIDATION.md

key-decisions:
  - "Each GUARD requirement is explicitly mapped to concrete tasks and test commands, not only to phases."

patterns-established:
  - "Validation maps for save/guard phases are kept in sync with split plans and test names to ease future audits."

requirements-completed: [GUARD-01, GUARD-02, GUARD-03, GUARD-04]

duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 07: Validation map + sign-off Summary

**Finalizes the Phase 25 validation map by mapping GUARD-01..04 to concrete tasks and tests from plans 25-01..25-06 and recording Nyquist sign-off.**

## Accomplishments

- Updated `25-VALIDATION.md` to reference the current split plans (25-01..25-07), including the new `unsaved_close_guard_tab_triggers` and `unsaved_close_guard_root_flow` tests and the guard-specific save-fail i18n step.
- Marked automated commands and Wave 0 dependencies for all GUARD-01..04 tasks, keeping sampling continuity and ensuring each requirement has at least one automated verify.
- Completed the validation sign-off checklist, including notes about `./check.sh` fmt drift being out-of-scope for this phase but tracked separately.

## Task Commits

1. **Validation map + sign-off for GUARD-01..04** – (included in docs commit for this phase)

## Deviations from Plan

- None – the validation map follows the updated plan structure and test names exactly.

## Issues Encountered

- `./check.sh` still reports existing rustfmt differences in non-guard modules; these are documented as pre-existing and unrelated to the guard behavior validated here.

## User Setup Required

- None – validation is based on automated tests and existing project scripts.

## Self-Check: PASSED

- FOUND: `.planning/phases/25-unsaved-close-guard/25-VALIDATION.md`

