# Architecture Research

**Domain:** Desktop textovy editor (Rust `eframe/egui`) - milestone v1.3.1 Safe Trash Delete
**Researched:** 2026-03-11
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                             │
├─────────────────────────────────────────────────────────────┤
│  Workspace            FileTree            Dialogs/Toasts    │
│  (`workspace/mod`)    (`file_tree/*`)     (`panels.rs`)     │
└───────────────┬───────────────┬───────────────┬─────────────┘
                │               │               │
┌───────────────┴───────────────┴───────────────┴─────────────┐
│                    Application Layer                          │
├─────────────────────────────────────────────────────────────┤
│  File ops orchestrace + new TrashService (`src/app/trash.rs`)│
│  - move_to_trash                                              │
│  - restore_from_trash                                         │
│  - cleanup_trash                                              │
└───────────────┬───────────────────────────────┬──────────────┘
                │                               │
┌───────────────┴───────────────────────────────┴──────────────┐
│                  Filesystem & Watchers                        │
├─────────────────────────────────────────────────────────────┤
│  real FS (`std::fs`)   ProjectWatcher   FileWatcher   IPC    │
│  + `.polycredo/trash`  (`watcher.rs`)   (`watcher.rs`) (`ipc`)|
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `src/app/ui/file_tree/*` | User akce delete/restore, potvrzovaci dialog, validace vstupu | UI event -> call service -> `pending_deleted`/toast |
| `src/app/ui/background.rs` | Reakce na watcher eventy, refresh stromu/indexu/tabu | `project_watcher.poll()` + `project_index.handle_change()` |
| `src/app/trash.rs` (new) | Bezpecny move/restore/cleanup, path safety, metadata | Ciste sync API nad `std::fs`, testy s `tempdir` |
| `src/app/project_config.rs` (extended) | Jednotna cesta `.polycredo/*` (`profiles`, `trash`) | helper funkce pro `trash_dir` a metadata soubor |
| `src/watcher.rs` | Automaticka synchronizace UI s FS zmenami | `FsChange::{Created,Removed,Modified}` |

## Recommended Project Structure

```
src/
├── app/
│   ├── trash.rs                    # nova trash service vrstva (core workflow)
│   ├── project_config.rs           # helpery pro .polycredo/trash cesty
│   └── ui/
│       ├── file_tree/
│       │   ├── dialogs.rs          # delete->trash + restore dialog trigger
│       │   └── ops.rs              # context akce Delete/Restore/Cleanup
│       └── workspace/
│           └── mod.rs              # napojeni akci na editor/toasty
├── watcher.rs                      # beze zmeny API, pouze vyuziti eventu
└── ipc.rs                          # beze zmeny (trash je per-project, ne global)
```

### Structure Rationale

- **`app/trash.rs`:** izoluje filesystem pravidla od UI, snadne jednotkove testy, minimalni zasah do `workspace`.
- **`project_config.rs`:** uz dnes drzi `.polycredo` convention, je prirozene misto pro `trash_dir()` helper.
- **`ui/file_tree/*`:** delete UX uz existuje zde, zmena hard-delete -> move je lokalni patch.

## Architectural Patterns

### Pattern 1: Thin UI, Fat Service

**What:** UI vrstva preda validovane parametry, vsechny FS rozhodnuti dela service.
**When to use:** Mazani, obnova, cleanup, kolize jmen, path safety.
**Trade-offs:** Plus testovatelnost a mensi coupling; minus o 1 modul navic.

**Example:**
```rust
match trash::move_to_trash(&ws.root_path, &path) {
    Ok(record) => ws.toasts.push(Toast::info(format!("Moved to trash: {}", record.display_name))),
    Err(e) => ws.toasts.push(Toast::error(e.to_string())),
}
```

### Pattern 2: Deterministic Path Policy

**What:** Vsechny path transformace jdou pres jednu politiku (`canonicalize`, root boundary checks, unique names).
**When to use:** move/restore/cleanup, prace s relativnimi cestami.
**Trade-offs:** Plus bezpecnost a predikovatelnost; minus vice explicitnich checku.

**Example:**
```rust
let rel = src.strip_prefix(project_root)?;
if rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
    return Err(TrashError::InvalidRelativePath);
}
```

### Pattern 3: Event-driven UI Refresh

**What:** Po operaci se primarne spolihat na watcher eventy; manual reload jen fallback.
**When to use:** delete->trash a restore, aby UI zustalo konzistentni i pri externich zmenach.
**Trade-offs:** Plus mene special-case kodu; minus zavislost na notify quality (uz je v projektu standard).

## Data Flow

### Request Flow (Delete -> Trash)

```
[User klikne Delete]
    ↓
[file_tree/dialogs.rs potvrzeni]
    ↓
[trash::move_to_trash(project_root, src_path)]
    ↓
[FS: create .polycredo/trash + rename(src,dst)]
    ↓
[watcher FsChange::Removed(src)]
    ↓
[background.rs: close tab + reload tree/index + toast]
```

### Request Flow (Restore)

```
[User Restore from trash]
    ↓
[trash::restore_from_trash(record_id, strategy)]
    ↓
[FS: rename(trash_item, restored_path)]
    ↓
[watcher FsChange::Created(restored_path)]
    ↓
[workspace open_file_in_ws? optional + tree expand]
```

### State Management

```
[WorkspaceState]
    ↓
[file_tree.pending_* + toasts + editor tabs]
    ↓
[ProjectWatcher events] -> [project_index + file_tree reload]
```

### Key Data Flows

1. **Delete flow:** `FileTree Delete` -> `TrashService move` -> watcher remove event -> UI cleanup.
2. **Restore flow:** `TrashService restore` -> watcher create event -> index/tree refresh + optional open file.
3. **Cleanup flow:** `TrashService cleanup(policy)` -> remove old records -> info/error toast summary.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k trash items/project | Prosty fs adresar + metadata JSON/TOML soubor staci |
| 1k-20k items/project | Pridat lazy listing + paginaci v UI trash dialogu |
| 20k+ items/project | Zavedeni indexu (append-only metadata log + periodic compaction) |

### Scaling Priorities

1. **First bottleneck:** listing trash obsahu v UI; fix = lazy load + sort v service vrstve.
2. **Second bottleneck:** metadata integrity po padu; fix = atomic write (`tmp` + `rename`) stejne jako v `ipc.rs`.

## Anti-Patterns

### Anti-Pattern 1: Hard delete fallback in UI

**What people do:** Kdyz move selze, UI provede `remove_file/remove_dir_all`.
**Why it's wrong:** Rozbije safety guarantee milestone v1.3.1.
**Do this instead:** Vratit error toast a nechat data na miste.

### Anti-Pattern 2: Trash paths computed ad-hoc in vice modulech

**What people do:** Kazdy modul sklada `.polycredo/trash` sam.
**Why it's wrong:** Nekonzistence a bezpecnostni chyby pri restore.
**Do this instead:** Jedno API v `project_config.rs` + `trash.rs`.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Filesystem (`std::fs`) | Prime volani (`rename`, `create_dir_all`, `remove_*`) | Zachovat atomicke write pattern pro metadata |
| notify watcher (`src/watcher.rs`) | Event callback -> `mpsc` -> `poll/try_recv` | `.polycredo` je ignorovano v `ProjectWatcher` (zadouci pro interni trash sum) |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `file_tree/dialogs.rs` <-> `app/trash.rs` | Prime API volani | nahradi dnesni `remove_file/remove_dir_all` |
| `background.rs` <-> `watcher.rs` | Eventy (`FsChange`, `FileEvent`) | zustava bez zmen API |
| `workspace state` <-> `panels.rs` | `pending_*`, `toasts`, editor close/open | idealni misto pro UX feedback |
| `project_config.rs` <-> `trash.rs` | helper funkce | centralizace `.polycredo` path konvenci |

## New vs Changed Components

### New
- `src/app/trash.rs`: core trash operace + metadata model + testy.

### Changed
- `src/app/project_config.rs`: helpery pro trash cesty.
- `src/app/ui/file_tree/dialogs.rs`: delete modal vola `move_to_trash`.
- `src/app/ui/file_tree/ops.rs`: volitelne pridani `Restore/Cleanup` context akci.
- `src/app/ui/panels.rs` a/nebo `workspace/mod.rs`: toasty a eventualni restore UX hook.

## Build Order (minimal invasive)

1. **Foundation:** pridat `project_config::trash_dir()` + `app/trash.rs` se sync API a unit testy path safety.
2. **Delete switch:** v `file_tree/dialogs.rs` nahradit hard delete za `move_to_trash`; zachovat existujici modal + error toast.
3. **Restore MVP:** pridat jednoduchy restore entry point (z dialogu/menu) bez redesignu panelu.
4. **Cleanup MVP:** pridat `cleanup_trash(policy)` (napr. starsi nez N dni / max count) + info toast.
5. **Polish:** i18n klice, doplnit testy na kolize jmen a nested dirs; overit `cargo check` + `./check.sh`.

## Testability Plan

- Unit: `trash.rs` (path validation, unique naming, metadata encode/decode, restore collision).
- Integration (tempdir): delete -> file zmizi z puvodni cesty a je v trash; restore vrati obsah.
- UI regression smoke: `file_tree` delete potvrzeni stale funguje, error je surfaced do toastu.

## Sources

- `.planning/PROJECT.md`
- `src/app/ui/file_tree/dialogs.rs`
- `src/app/ui/file_tree/ops.rs`
- `src/app/ui/panels.rs`
- `src/app/ui/background.rs`
- `src/app/project_config.rs`
- `src/app/ui/file_tree/node.rs`
- `src/watcher.rs`
- `src/app/ui/workspace/mod.rs`

---
*Architecture research for: Milestone v1.3.1 Safe Trash Delete*
*Researched: 2026-03-11*
