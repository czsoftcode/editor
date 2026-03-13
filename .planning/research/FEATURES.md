# Feature Research

**Domain:** Rust desktop text editor (local-first file operations)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Move-to-trash instead of hard delete | Users expect accidental delete recovery in modern editors | MEDIUM | Replace current hard delete flow in `src/app/ui/file_tree/dialogs.rs` (`remove_file/remove_dir_all`) with atomic move into `.polycredo/trash`. |
| Internal trash lifecycle (`create/list/restore/purge`) | Safe delete is incomplete without restore and cleanup path | MEDIUM | Reuse project-local config pattern from `src/app/project_config.rs`; keep trash under project root for local-first behavior. |
| Non-blocking UI for delete/restore operations | Editor must stay responsive during file ops | MEDIUM | Keep modal UX in `src/app/ui/file_tree/*`, run heavier FS operations via background task pattern in `src/app/ui/background.rs` (`spawn_task`). |
| Clear user feedback (toast + predictable errors) | File actions must be observable and debuggable | LOW | Use existing toast channel in `src/app/ui/panels.rs` + `pending_error` pattern from file tree dialogs. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Project-scoped hidden trash (`.polycredo/trash`) | Strong privacy/locality, no OS-global coupling, portable with project | LOW | Aligns with existing `.polycredo` usage (`local_history`, semantic index, profiles). Already hidden in tree via `src/app/ui/file_tree/node.rs`. |
| Metadata-rich trash entries (original path + timestamp + op id) | Fast and accurate restore to original location | MEDIUM | Can mirror simple index approach from `src/app/local_history.rs` (`index.json` via serde_json). |
| Restore-aware tab/workspace refresh | Recovery feels native, not bolted on | MEDIUM | Integrate with existing reload/close hooks in `src/app/ui/background.rs` and `src/app/ui/panels.rs` (`close_tabs_for_path`, tree reload requests). |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| OS recycle bin integration as first milestone scope | "Native" trash behavior across platforms | Cross-platform inconsistency, permissions edge-cases, bigger QA matrix | Keep v1.3.1 project-internal trash only; evaluate OS integration post-validation. |
| Unlimited trash retention with no policy | "Never lose anything" expectation | Unbounded disk growth, slower maintenance, hidden failures | Add explicit purge action and optional retention cap in `.polycredo/trash`. |
| Recursive hard-delete fallback on move failure | "Delete must always succeed" pressure | Defeats safety goal and can cause irreversible data loss | Fail closed: keep source untouched on failed move, show actionable error toast. |

## Feature Dependencies

```
[Safe Delete UX (confirm + action)]
    └──requires──> [Trash Storage (.polycredo/trash)]
                       └──requires──> [Project Root + Config Paths]

[Restore Action]
    └──requires──> [Trash Metadata Index]
                       └──requires──> [Path Validation + Conflict Handling]

[Watcher/Tree Refresh] ──enhances──> [Safe Delete UX]
[Hard Delete Fallback] ──conflicts──> [Safe Delete Guarantees]
```

### Dependency Notes

- **Safe Delete UX requires Trash Storage:** bez fyzickeho cile pro move nelze garantovat recoverability.
- **Trash Storage requires Project Root + Config Paths:** trash musi byt deterministicky pod projektem (reuse `project_config` path conventions).
- **Restore Action requires Trash Metadata Index:** restore potrebuje mapovani `trash_entry -> puvodni cesta` a cas operace.
- **Trash Metadata Index requires Path Validation + Conflict Handling:** restore musi osetrit kolize (soubor uz existuje) a neplatne cesty.
- **Watcher/Tree Refresh enhances Safe Delete UX:** UI musi po move/restore konzistentne zavirat/otevirat taby a reloadovat strom.
- **Hard Delete Fallback conflicts with Safe Delete Guarantees:** fallback by porusil hlavni hodnotu milestone (bezpecne mazani).

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] Replace hard delete with project-local move-to-trash for files and directories.
- [ ] Ensure `.polycredo/trash` auto-create and hidden behavior in file tree stays intact.
- [ ] Provide at least one restore path (single item) with clear conflict/error handling.
- [ ] Surface all delete/restore failures through existing toast UX.

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] Bulk restore/purge actions — add when single-item flow is stable and tested.
- [ ] Retention policy (age/count/size caps) — add when real trash growth data is available.

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Optional OS recycle bin bridge per platform — defer due to platform complexity.
- [ ] Trash timeline UI with preview/diff — defer until basic workflow adoption is confirmed.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Move-to-trash replace for delete | HIGH | MEDIUM | P1 |
| Basic restore (single item) | HIGH | MEDIUM | P1 |
| Error/toast propagation for all FS failures | HIGH | LOW | P1 |
| Metadata index for restore integrity | MEDIUM | MEDIUM | P2 |
| Bulk trash operations | MEDIUM | MEDIUM | P2 |
| Retention policy automation | MEDIUM | MEDIUM | P3 |
| OS recycle bin integration | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Competitor A | Competitor B | Our Approach |
|---------|--------------|--------------|--------------|
| Safe delete default | VS Code: integrated trash/recycle-bin behavior | Sublime Text: safe delete via system/setting-dependent flows | Project-internal `.polycredo/trash` to keep behavior deterministic across OSes. |
| Restore workflow | VS Code: restore from OS trash out-of-app | JetBrains IDEs: Local History + recovery actions | Lightweight in-app restore from internal trash, aligned with local-first UX. |
| Cleanup policy | Usually delegated to OS trash policy | Mixed; often manual cleanup | Explicit in-app purge controls, optional retention later (v1.x). |

## Sources

- `.planning/PROJECT.md` (milestone goal and scope for v1.3.1)
- Current codebase modules:
  - `src/app/ui/file_tree/dialogs.rs` (current hard delete path)
  - `src/app/ui/file_tree/node.rs` (`.polycredo` hidden from tree)
  - `src/watcher.rs` (`.polycredo` ignored by project watcher)
  - `src/app/project_config.rs` (project-scoped config directory conventions)
  - `src/app/local_history.rs` (project-local index + serde_json persistence pattern)
  - `src/app/ui/background.rs`, `src/app/ui/panels.rs` (async pattern + toast/error surfacing)
- Product baseline references used only for qualitative comparison:
  - Visual Studio Code behavior expectations for file delete/trash
  - JetBrains IDE recovery/local history mental model

---
*Feature research for: Safe Trash Delete (v1.3.1)*
*Researched: 2026-03-11*
