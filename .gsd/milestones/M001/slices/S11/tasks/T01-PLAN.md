# T01: 20-gsd-core-state-engine 01

**Slice:** S11 — **Milestone:** M001

## Description

Build the custom YAML-like frontmatter parser with full round-trip fidelity.

Purpose: This is the foundational data layer for all GSD commands. Every GSD command that reads or writes `.planning/` markdown files depends on this parser. It must handle the full YAML subset (strings, integers, floats, booleans, lists, nested maps, quoted strings, inline lists/maps) while preserving comments, whitespace, and key ordering for lossless round-trip.

Output: `frontmatter.rs` with FmValue, FmNode, FmDocument types and comprehensive unit tests.

## Must-Haves

- [ ] "Frontmatter parser correctly extracts key-value pairs from --- delimited YAML-like blocks"
- [ ] "Parser handles full YAML subset: string, integer, float, boolean, list, nested map, quoted strings, inline lists/maps"
- [ ] "Round-trip: parse then serialize produces identical output for unchanged documents"
- [ ] "Tolerant parsing: invalid lines are skipped, partial result + warnings returned"
- [ ] "Dot-notation get/set works for nested values (e.g. progress.completed_phases)"
- [ ] "Body content after closing --- is preserved unchanged"

## Files

- `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs`
