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
