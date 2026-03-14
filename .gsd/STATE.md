# GSD State

**Active Milestone:** M004 — Klávesové Zkratky a Centrální Keymap
**Active Slice:** S03 — Uživatelská konfigurace keybindings a dynamické labely
**Phase:** planned → ready for execution
**Requirements Status:** 2 active · 13 validated · 0 deferred · 2 out of scope

## Milestone Registry
- ✅ **M001:** Migration
- ✅ **M002:** Local History
- ✅ **M003:** Vylepšení UI Historie Souboru
- 🔄 **M004:** Klávesové Zkratky a Centrální Keymap

## Recent Decisions
- Override na CommandRegistry commands (single source of truth)
- Re-init defaults před apply overrides v save flow
- Ctrl+A/C/V/X/Z/Y jako reserved neoverridovatelné
- S03 jako jeden task

## Blockers
- None

## Next Action
Execute T01 (keybinding overrides + rebuild + testy).
