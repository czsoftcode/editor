# Project Research Summary

**Project:** PolyCredo Editor
**Domain:** Desktop editor save workflow and unsaved-changes safety
**Researched:** 2026-03-09
**Confidence:** HIGH

## Executive Summary

Milestone v1.3.0 má velmi jasný cíl: sjednotit ukládání tak, aby bylo předvídatelné (Ctrl+S jako default), konfigurovatelné (auto/manual režim) a bezpečné (žádná ztráta neuložené práce při zavření tabu nebo aplikace).

Doporučený přístup nevyžaduje nové závislosti. Nejlepší cesta je rozšířit stávající settings model o jednoznačný save mode, centralizovat rozhodování save/close do jednoho flow a použít existující modal pattern pro Save/Discard/Cancel.

Největší riziko je nekonzistence mezi více close cestami a focus stavy. Mitigace: shared close guard handler, explicitní testy na tab close + app close + Ctrl+S focus edge cases.

## Key Findings

### Recommended Stack

Použít stávající stack: Rust + eframe/egui + settings persistence. Integrace je nízkoriziková a zapadá do aktuální architektury bez async runtime změn.

**Core technologies:**
- Rust: save/close orchestrace a error handling
- eframe/egui: toggle + confirm dialog UI
- serde settings model: persistovaný auto/manual režim

### Expected Features

**Must have (table stakes):**
- Ctrl+S ukládání jako defaultní cesta
- Režim auto/manual v Settings
- Confirm při close neuloženého tabu
- Confirm při close aplikace s neuloženými změnami

**Should have (competitive):**
- Jednotný close-flow pro tab i app
- Viditelná indikace aktivního save režimu

**Defer (v2+):**
- Per-project save režim
- Save-all/discard-all při app close

### Architecture Approach

Architektura má stát na jednom source-of-truth pro save mode a jednom close guard handleru, který je volán ze všech close entrypointů. Dirty-state zůstává per tab a je vstupem pro decision dialog.

**Major components:**
1. Save mode state (settings + runtime apply)
2. Save dispatcher (manual vs auto behavior)
3. Close guard decision engine (Save/Discard/Cancel)

### Critical Pitfalls

1. **Silent data loss on close** — vždy vynutit confirm flow při dirty stavech
2. **Inconsistent auto/manual behavior** — držet single source-of-truth v settings
3. **Ctrl+S focus regressions** — přidat testy pro modal/focus scénáře

## Implications for Roadmap

### Phase 24: Save Mode Foundation
**Rationale:** Nejprve je potřeba stabilní základ save mode logiky.
**Delivers:** Ctrl+S default, save mode settings, runtime apply.
**Addresses:** Manual save + auto/manual toggle requirements.
**Avoids:** Nekonzistence režimu a skryté save side-effecty.

### Phase 25: Unsaved Close Guard
**Rationale:** Navazuje na hotovou save mode logiku.
**Delivers:** Guard dialog pro tab close a app close.
**Uses:** Dirty-state model a save dispatcher z Phase 24.
**Implements:** Centrální close decision flow.

### Phase 26: UX + Regression Hardening
**Rationale:** Po zavedení chování zpevnit UX a testy.
**Delivers:** Indikace režimu/stavu, edge-case testy, polish.

### Phase Ordering Rationale

- Save mode musí být zaveden před close guard, jinak nebude jasné, co „save“ znamená.
- Close guard až druhý krok: vyžaduje stabilní save API.
- Test/polish fáze nakonec minimalizuje regresní riziko.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 25:** UX wording a multi-file close decision behavior.

Phases with standard patterns (skip research-phase):
- **Phase 24:** standard settings + shortcut + runtime apply pattern.
- **Phase 26:** standard regression hardening/testing workflow.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Bez nových dependencies, plná kompatibilita s existující architekturou |
| Features | HIGH | Požadavky jsou konkrétní a běžně očekávané v editorech |
| Architecture | HIGH | Přímé napojení na stávající settings/editor/modal patterns |
| Pitfalls | HIGH | Rizika dobře známá z editor domény |

**Overall confidence:** HIGH

### Gaps to Address

- Přesné UX texty dialogů (tab close vs app close) doladit při plan-phase.
- Rozhodnout, zda app-close dialog nabídne i „Save all“ už ve v1.3.0.

## Sources

### Primary (HIGH confidence)
- `.planning/PROJECT.md` — constraints, current architecture, milestone intent
- Existing code structure in `src/app/ui/editor`, `src/app/ui/workspace`, `src/settings.rs`

### Secondary (MEDIUM confidence)
- Běžné UX konvence desktop editorů (Ctrl+S, dirty close confirm)

---
*Research completed: 2026-03-09*
*Ready for roadmap: yes*
