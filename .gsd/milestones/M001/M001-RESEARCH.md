# Project Research Summary

**Project:** PolyCredo Editor
**Domain:** Rust desktop text editor (local-first file operations)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Executive Summary

Research pro milestone `v1.3.1 safe trash delete` jednoznacne ukazuje, ze nejmensi riziko a nejvyssi predikovatelnost ma projektove-local pristup: nahradit hard delete za move-to-trash do `.polycredo/trash`, pridat minimalni restore flow a explicitni cleanup. Nejde o architektonicky rework, ale o cileny capability patch, ktery respektuje stavajici single-process multi-window architekturu.

Doporuceny pristup je tenky UI layer + dedikovana `TrashService` vrstva nad `std::fs`, s centralizaci cest v `project_config` a robustnim error surfacingem do toastu. Pro metadata trash polozek je doporuceno pouzit `serde_json` (stabilni serializace, bez manualniho JSON skladani).

Hlavni rizika jsou datova nekonzistence (kolize jmen, restore konflikty), destruktivni fallbacky pri `rename` fail path (EXDEV) a UX/performance regrese pri blokujicim I/O a watcher event stormech. Mitigace: deterministic id+metadata, non-destructive restore policy, async/background execution a watcher batching/dedup.

## Key Findings

### Recommended Stack

Pro scope `v1.3.1` nejsou potreba nove runtime zavislosti. Core implementace ma zustat na Rust 2024 + `std::fs/std::path`, s UI integraci v existujicim `eframe/egui` stacku. Metadata per trash item nebo index ma byt serializovan pres `serde_json`, ktere uz je v projektu.

Stack volba je orientovana na minimalni zmenu povrchu: zadny novy async runtime, zadna DB, zadny globalni refactor. Pro testy je vhodne pouzit `tempfile` scenare (move/restore/cleanup/collision) a drzet quality gate `cargo check` + `./check.sh`.

**Core technologies:**
- Rust 2024 + `std::fs`/`std::path`: move/restore/cleanup + path safety bez nove runtime vrstvy.
- `eframe`/`egui`: existujici UI workflow (dialogs, toasty, background task hooky) bez rewritu.
- `serde` + `serde_json`: robustni metadata persistence pro trash itemy.

### Expected Features

MVP musi dodat tri veci: bezpecny move-to-trash misto hard delete, minimalni restore path a spolehlivy error feedback. Diferenciator je project-scoped hidden trash (`.polycredo/trash`) s deterministic chovanim napric OS. V2+ tema (OS recycle bin bridge, pokrocila timeline) je mimo milestone.

**Must have (table stakes):**
- Move-to-trash jako default delete tok (soubor i adresar).
- Minimalni restore workflow s conflict handlingem.
- Neblokujici UI + viditelne chyby (toast) pro vsechny FS fail pathy.

**Should have (competitive):**
- Metadata-rich trash zaznam (original path, cas, id) pro presnou obnovu.
- Konzistentni refresh tree/tabu po move/restore pres watcher + fallback reload.

**Defer (v2+):**
- OS recycle bin integrace.
- Pokrocily trash timeline/preview/diff.

### Architecture Approach

Doporucena architektura je zavest `src/app/trash.rs` jako centralni service pro `move_to_trash`, `restore_from_trash`, `cleanup_trash`, a v UI (`file_tree/dialogs.rs`, `ops.rs`) pouze volat service API a zobrazovat vysledek. Cesty do `.polycredo/trash` centralizovat v `project_config.rs`. Refresh stavu drzet event-driven pres watchery, ale s deduplikaci/batching, aby nevznikal storm a stale UI stav.

**Major components:**
1. `app/trash.rs` (new): bezpecne FS workflow + metadata + conflict policy.
2. `app/project_config.rs` (extended): jednotna konvence cest pro `.polycredo/trash`.
3. `app/ui/file_tree/*` + `ui/background.rs`: trigger akce, toasty, refresh stromu/tabu.

### Critical Pitfalls

Nejvetsi rizika jsou destruktivni fallback chovani, nekonzistentni metadata a blokujici integrace do UI.

1. **Kolize trash identit/metadat** - pouzit unikatni id a restore ridit podle id, ne podle basename.
2. **EXDEV/rename fail path ztrati data** - fallback musi byt `copy -> verify -> delete source`; pri chybe source nesmazat.
3. **Restore prepise existujici cil** - default non-destructive policy (`rename-copy/cancel`), nikdy tichy overwrite.
4. **Cleanup porusi konzistenci** - retention ridit pres `deleted_at` metadata a mazat transakcne data+meta.
5. **Watcher storm + UI freeze** - event batching/dedup a zadne tezke I/O na UI vlakne.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Trash Foundation
**Rationale:** Bez jednotne cesty a metadata kontraktu nelze bezpecne implementovat move/restore.
**Delivers:** `trash_dir` helper + metadata schema + zaklad `TrashService`.
**Addresses:** Internal trash lifecycle, path safety baseline.
**Avoids:** Kolize identit a ad-hoc path skladani.

### Phase 2: Safe Move Engine
**Rationale:** Primarni value milestone je nahradit hard delete bez regresi.
**Delivers:** delete->move workflow + EXDEV-safe fallback + error surfacing.
**Uses:** `std::fs`, `serde_json`, stavajici toast pipeline.
**Implements:** Service operation `move_to_trash`.

### Phase 3: Restore MVP
**Rationale:** Safe delete bez obnovy je produktove nekompletni.
**Delivers:** single-item restore + conflict policy + watcher refresh hook.
**Uses:** Metadata index/id-based restore.
**Implements:** `restore_from_trash` + UI trigger.

### Phase 4: Cleanup + Reliability
**Rationale:** Bez retention/cleanup trash dlouhodobe degradije UX a disk.
**Delivers:** cleanup policy, orphan handling, consistency checks.
**Uses:** `deleted_at` metadata + atomicke mazani data/meta.
**Implements:** `cleanup_trash` + verify scenare.

### Phase Ordering Rationale

- Poradi respektuje zavislosti: schema/path policy -> move core -> restore -> cleanup.
- Grouping oddeluje kriticke datove safety body od UX polish, aby P1 flow byl rychle verifikovatelny.
- Nejdriv se eliminuji pitfall tridy s nejvyssim dopadem (data loss), potom performance/operacni robustnost.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** EXDEV fallback a fsync/verify nuance podle platformy.
- **Phase 4:** Cleanup policy thresholdy (age/count/size) a UX preview kompromisy.

Phases with standard patterns (skip research-phase):
- **Phase 1:** path helpers + metadata schema jsou etablovane a lokalni.
- **Phase 3:** restore conflict policy je standardni non-destructive UX pattern.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Opira se o existujici zavislosti a zavedeny crate patterny v projektu. |
| Features | HIGH | Scope je jasne vymezeny na safe trash delete MVP. |
| Architecture | HIGH | Navazuje na aktualni moduly bez potreby globalniho refactoru. |
| Pitfalls | MEDIUM | Rizika jsou dobre identifikovana, ale cast fallback edge-case je platform-sensitive. |

**Overall confidence:** HIGH

### Gaps to Address

- Cross-device fallback validace: overit chovani na realnych mount scenarich v integracnich testech.
- Watcher burst robustnost: doplnit stress testy na delete/restore sekvence s deduplikaci eventu.

## Sources

### Primary (HIGH confidence)
- `.planning/research/STACK.md` - doporuceny stack a anti-stack rozhodnuti.
- `.planning/research/FEATURES.md` - MVP feature baseline a priorita.
- `.planning/research/ARCHITECTURE.md` - navrhovana komponentizace a datove toky.
- `.planning/research/PITFALLS.md` - rizika, warning signs a mitigace.
- `.planning/PROJECT.md` - aktivni milestone scope v1.3.1.

### Secondary (MEDIUM confidence)
- Existing codebase conventions (`project_config`, `local_history`, `file_tree`, `watcher`) pro alignment implementace.

### Tertiary (LOW confidence)
- Kvalitativni srovnani UX ocekavani s beznymi editory (VS Code/JetBrains) pro table-stakes framing.

---
*Research completed: 2026-03-11*
*Ready for roadmap: yes*

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

# Stack Research

**Domain:** Rust desktop editor (`eframe/egui`) - milestone `v1.3.1 safe trash delete`
**Researched:** 2026-03-11
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (edition) | 2024 | Implementace move-to-trash, restore a cleanup logiky | Maximalni kontrola nad FS operacemi a error handlingem bez nove runtime vrstvy. |
| Rust `std::fs` + `std::path` | std (toolchain) | Atomicky-ish move (`rename`), fallback copy+remove, validace cest | Pro lokalni projektovy trash je to nejmensi a nejstabilnejsi stack bez dalsich zavislosti. |
| `eframe`/`egui` | `0.31` | UI trigger/delete/restore/cleanup workflow + toasty s chybami | Zapada do existujici architektury single-process multi-window, bez UI rewritu. |
| `serde` + `serde_json` | `1.x` + `1.x` | Volitelny metadata manifest trash polozek (puvodni cesta, timestamp) | Zavislost uz existuje; bezpecnejsi a robustnejsi nez rucni serializace. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `notify` | `7` | Reakce na zmeny v projektu po move/restore operacich | Pouzit jen pro sladeni UI stavu po presunech; nepridavat novy watcher subsystem. |
| `rfd` | `0.15` | Asynchronni file dialog pro restore/cisteni akce | Pouzit pouze pokud akce vyzaduje vyber uzivatele; UI vlakno zustava neblokujici. |
| `tempfile` (dev) | `3` | Izolovane testy pro trash scenare | Pouzit v testech move/restore/collision/cleanup policy. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo check` | Rychla validace kompilace a typu | Povinny minimalni gate po kazdem patchi. |
| `./check.sh` | Projektovy quality gate | Spoustet po logickych zmenach trash workflow. |

## Installation

```bash
# Zadna nova runtime zavislost neni pro v1.3.1 nutna.
# Pouzit existujici stack z Cargo.toml.

cargo check
./check.sh
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `std::fs::rename` + fallback copy/remove | crate `trash` (OS recycle bin) | Jen pokud by produktovy smer vyzadoval OS-integraci misto interniho `.polycredo/trash`. |
| `serde_json` metadata | rucne skladany JSON string | Nikdy pro tento milestone; alternativa nedava vyhodu a zvysuje riziko chyb. |
| Jednoducha cleanup policy (age/count/size) v app logice | DB index (SQLite apod.) | Jen pokud trash naroste do radove tisicu+ polozek a bude potreba pokrocile dotazovani. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Novy async runtime nebo job system | Overengineering mimo scope, zvyseni complexity a riziko regresi | Existujici vlakna/task model aplikace + kratke FS operace mimo UI blokace. |
| Externi DB pro trash metadata | Zbytecna operacni a implementacni rezie pro milestone `v1.3.1` | Jednoduchy JSON manifest (`serde_json`) nebo per-item metadata soubor. |
| Hard delete jako default tok | Neresi bezpecnostni cil milestone (moznost obnovy) | Move-to-trash do `.polycredo/trash` jako primarni cesta. |
| Globalni refactor architektury editoru | Mimo zadany scope capability-focused zmen | Cileny patch jen kolem delete/restore/cleanup bodu. |

## Stack Patterns by Variant

**If soubor zustava ve stejnem filesystemu projektu:**
- Use `std::fs::rename` do `.polycredo/trash/<id>-<name>`.
- Because je to nejrychlejsi a nejmene chybova cesta bez kopirovani dat.

**If `rename` selze kvuli cross-device nebo permission edge case:**
- Use fallback `copy` + `remove_file`/`remove_dir_all` s rollback/error toastem.
- Because zajisti robustni chovani i mimo idealni FS podminky.

**If restore cil uz existuje (collision):**
- Use explicitni conflict policy (rename restored item nebo potvrzeni uzivatelem).
- Because tiche prepisy jsou rizikove a nesmi byt default.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `eframe@0.31` | `egui_extras@0.31` | Drzet stejny minor kvuli API konzistenci UI vrstvy. |
| `serde@1.x` | `serde_json@1.x` | Stabilni serializace trash metadata bez custom parseru. |
| `notify@7` | Rust 2024 crate stack | Dostacujici pro navazani na existujici watcher logiku. |

## Sources

- `.planning/PROJECT.md` - aktivni milestone scope, cile a out-of-scope hranice.
- `Cargo.toml` - aktualni verze stacku a overeni, ze nove crate nejsou nutne.
- Rust std docs (`std::fs`, `std::path`) - validace, ze move/copy/remove scenare lze pokryt bez dalsich knihoven.

---
*Stack research for: PolyCredo Editor v1.3.1 safe trash delete*
*Researched: 2026-03-11*

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

# Pitfalls Research

**Domain:** Safe Trash Delete v desktop editoru (Rust/eframe/egui)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Kolize identit pri move do trash

**What goes wrong:**
Dva soubory se stejnym nazvem (nebo opakovane mazani stejne cesty) se v trash prepisi, nebo nelze jednoznacne urcit puvodni lokaci pri restore.

**Why it happens:**
Implementace uklada jen basename bez stabilniho metadata zaznamu (puvodni cesta, cas, duvod kolize).

**How to avoid:**
Pouzit atomicky rename do unikatniho trash id (`timestamp + nonce`) a vedle datoveho souboru ukladat metadata (`original_path`, `deleted_at`, `size`, `hash optional`). Obnovu ridit pres id, ne pres jmeno.

**Warning signs:**
- Restore UI ukazuje vice stejnych nazvu bez puvodni cesty.
- Pri opakovanem delete stejny soubor v trash "zmizi" nebo se prepise.
- Logika restore vybira prvni shodu podle nazvu.

**Phase to address:**
Faze v1.3.1-01 (trash schema + metadata kontrakt) a v1.3.1-02 (move-to-trash implementace).

---

### Pitfall 2: Cross-device move fallback udela hard delete

**What goes wrong:**
Pri rename mezi odlisnymi filesystemy (`EXDEV`) fallback nechtene smaze zdroj driv, nez je cil bezpecne zapsan.

**Why it happens:**
Implementace predpoklada, ze `rename` vzdy funguje atomicky, a fallback copy+delete nema fsync/verify krok.

**How to avoid:**
Implementovat explicitni fallback: `copy -> fsync file -> fsync parent dir -> verify size(optional hash) -> delete source`. Pokud kterykoli krok selze, source nesmazat a vratit chybu do UI toastu.

**Warning signs:**
- Chyby `Invalid cross-device link` v logu bez retry/fallback stopy.
- Uzivatel hlasi ztratu souboru pri mazani z mounted adresare.
- Telemetrie/trace ukazuje delete source i pri failed copy.

**Phase to address:**
Faze v1.3.1-02 (move-to-trash engine) a v1.3.1-06 (error-path testy).

---

### Pitfall 3: Restore prepise existujici soubor bez potvrzeni

**What goes wrong:**
Obnova do puvodni cesty prepise aktualni existujici soubor, uzivatel prijde o novejsi data.

**Why it happens:**
Restore tok neresi konflikt na cilove ceste a chybi UX volba (skip/rename/replace).

**How to avoid:**
Pred restore kontrolovat cil (`exists + is_file/is_dir`) a nabidnout deterministic varianty: `restore as copy (name suffix)`, `cancel`, `replace after confirm`. Vychozi volba musi byt non-destructive.

**Warning signs:**
- Restore probehne bez dialogu i kdyz cil existuje.
- User reporty "po obnove chybi moje novejsi verze".
- Testy pokryvaji restore success, ale ne conflict scenar.

**Phase to address:**
Faze v1.3.1-03 (restore flow + conflict policy) a v1.3.1-05 (UI wiring + i18n texty).

---

### Pitfall 4: Cleanup maze nespravne polozky

**What goes wrong:**
Cleanup podle veku/velikosti smaze i polozky, ktere jeste nemely byt expirovane, nebo smaze metadata bez dat (nekonzistence trash).

**Why it happens:**
Pouziti mtime misto `deleted_at`, neatomicke mazani dvojice `data+meta`, chybejici dry-run/preview.

**How to avoid:**
Opirat TTL o `deleted_at` z metadata, cleanup provadet jako transakcni jednotku nad zaznamem (nejdriv validace vazby data/meta, pak delete oboji). Pro UI nabidnout preview poctu/objemu pred potvrzenim.

**Warning signs:**
- Nahodne "mizeni" cerstve smazanych souboru.
- Trash listing obsahuje sirotci metadata nebo data bez metadata.
- Cleanup kod filtruje pres filesystem times bez metadata.

**Phase to address:**
Faze v1.3.1-04 (cleanup policy + retention) a v1.3.1-06 (integracni testy konzistence).

---

### Pitfall 5: Watcher event storm a stale UI stav

**What goes wrong:**
Move/restore vyvola lavinu filesystem eventu; UI seznam souboru/trash osciluje, duplikuje polozky nebo se neaktualizuje.

**Why it happens:**
Watchery zpracovavaji jednotlive eventy bez deduplikace, nefiltruji interni `.polycredo/trash` zmeny, nebo mixuji interni a user-visible eventy.

**How to avoid:**
Zavedeni event batching (`HashSet<PathBuf>` + debounce tick), explicitni filtrace internich trash operaci pro workspace tree, a oddeleny refresh pipeline pro trash panel.

**Warning signs:**
- Po delete/restore skace vyber v exploreru.
- V logu stovky watcher eventu pro jednu akci.
- Intermitentni flaky testy na file tree refresh.

**Phase to address:**
Faze v1.3.1-05 (watcher/UI integrace) a v1.3.1-06 (stress + race testy).

---

### Pitfall 6: Blokujici I/O v UI vlakne

**What goes wrong:**
Mazani/obnova/cleanup blokuje main thread; UI zamrzne pri vetsich souborech nebo pomalem disku.

**Why it happens:**
Disk operace bez background job queue + bez progress/error signalu z workeru do UI.

**How to avoid:**
Spoustet move/restore/cleanup asynchronne mimo UI vlakno, navratovy stav komunikovat pres kanal (`pending/success/error`) a chyby zobrazovat toastem. UI akce locknout jen lokalne (konkretni item), ne globalne.

**Warning signs:**
- Input lag po stisku Delete/Restore.
- Dlouhe frame times pri disk operacich.
- "Not responding" okna na velkem projektu.

**Phase to address:**
Faze v1.3.1-02 (engine async contract) a v1.3.1-05 (UI integration + progress state).

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Ukladat jen nazev souboru bez metadata | Rychla implementace | Nelze bezpecne restore ani auditovat puvod | Nikdy |
| Ignore error vetve pri cleanup | Mene kodu v prvni iteraci | Tiche ztraty dat, nekonzistentni trash | Nikdy |
| Globalni mutex kolem celeho trash workflow | Snadne "vyresi" race | UI stalls, head-of-line blocking | Jen kratkodobe pro hotfix s ticketem na odstraneni |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| File watcher (`notify`) | Reakce na kazdy event bez batch/debounce | Agregovat eventy do setu a obnovovat UI po ticku |
| Workspace tree UI | Sledovat `.polycredo/trash` stejne jako bezne soubory | Trash drzet jako interni domenu s oddelenym refresh tokem |
| Toast/error pipeline | Chyby move/restore jen logovat | Propagovat chybu uzivateli + navrhnout recovery akci |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full rescan projektu po kazdem delete/restore | Vysoke CPU, lag exploreru | Cileny refresh jen dotcenych cest + debounce | Stovky az tisice souboru |
| Sync cleanup vseho trash najednou | Zamrzani UI | Batch cleanup po blocich s progress signalem | Trash > 1-2 GB nebo HDD |
| Kopirovani velkych souboru v UI threadu | Frame dropy, "not responding" | Worker thread + chunked progress | Jednotky stovek MB |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Restore bez canonicalize/validace cesty | Path traversal mimo projekt | Pred restore overit canonical path a povoleny root projektu |
| Nasledovani symlinku v trash metadatech | Obnova nebo cleanup nad cizimi cestami | Ukladat `symlink` flag a defaultne symlinky ne-followovat |
| Trust filesystem timestampu pro retention | Obchazeni cleanup politiky | Retention ridit pouze pres interni `deleted_at` metadata |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Delete vypada jako hard delete bez zpetne vazby | Neduvera, panika ze ztraty dat | Toast "Presunuto do kosu" + akce "Obnovit" |
| Restore bez konfliktniho dialogu | Nechtene prepsani souboru | Non-destructive default + jasne volby |
| Cleanup bez preview | Uzivatel nevi co zmizi | Preview poctu/velikosti + potvrzeni |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Move-to-trash:** Chybi EXDEV fallback a verify krok - over `rename` fail path testem.
- [ ] **Restore:** Chybi conflict handling na existujicim cili - over testem `restore_when_target_exists`.
- [ ] **Cleanup:** Chybi konzistence `data+meta` mazani - over testem se sirotky.
- [ ] **Watcher integrace:** Chybi deduplikace eventu - over stress testem s burst operacemi.
- [ ] **UI responsiveness:** Chybi async pipeline - over frame-time smoke testem pri velkem souboru.
- [ ] **Error surfacing:** Chyby zustavaji jen v logu - over toast assertions v UI testech.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Kolize/prehazene metadata | HIGH | Stop cleanup, zmrazit nove delete operace, projet trash index verifier, obnovit metadata z journalu nebo fallback na manualni mapping podle inode/size/time |
| Nechtene prepsani pri restore | HIGH | Okamzite ulozit overwritten verzi do nouzoveho backupu (pokud existuje), zablokovat dalsi restore bez conflict policy, pridat guard test a hotfix |
| Event storm/stale UI | MEDIUM | Zapnout watcher debounce fallback, invalidovat cache tree, provest jednorazovy full refresh a nasadit batch dedup patch |
| Zamrzani UI pri trash operacich | MEDIUM | Presunout operace do workeru, docasne omezit cleanup batch size, pridat progress indikator a timeout guard |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Kolize identit pri move do trash | Faze v1.3.1-01/02 | Jednotkove testy uniqueness + metadata roundtrip pro opakovany delete |
| EXDEV fallback udela hard delete | Faze v1.3.1-02 | Integracni test simulujici rename fail a kontrola ze source zustal pri failed copy |
| Restore prepise existujici data | Faze v1.3.1-03/05 | UI+logic test `target exists` s default non-destructive rozhodnutim |
| Cleanup maze nespravne polozky | Faze v1.3.1-04/06 | TTL testy nad `deleted_at`, orphan detection test, dry-run assertion |
| Watcher event storm rozbije UI stav | Faze v1.3.1-05/06 | Stress test burst delete/restore + assert stabilniho tree/trash listu |
| Blokujici I/O v UI vlakne | Faze v1.3.1-02/05 | Performance smoke: delete/restore behem interakce bez frame stall regressi |

## Sources

- `.planning/PROJECT.md` (milestone scope a quality gate kontext)
- Zkusenosti z desktop file manager/editor implementaci (watcher event storms, cross-device move edge cases)
- Bezne failure modes `rename/copy/delete` workflow v POSIX prostredi

---
*Pitfalls research for: v1.3.1 Safe Trash Delete*
*Researched: 2026-03-11*