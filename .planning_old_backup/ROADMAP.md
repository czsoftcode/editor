# Roadmap: PolyCredo Editor — Performance Optimization

## Overview

Tři sekvenční fáze eliminují nadměrnou CPU zátěž editoru. Fáze 1 zastaví zbytečné překreslování GUI (největší dopad, nejnižší riziko). Fáze 2 přesune git polling a file watching z UI threadu do pozadí a odstraní burst events. Fáze 3 izoluje terminálový emulátor tak, aby repaintoval pouze při skutečném výstupu — ne v busy-loop.

## Phases

- [ ] **Phase 1: Repaint Gate** - Podmíněné překreslování a vypnutí accesskit eliminují kontinuální 60fps render loop v idle stavu
- [ ] **Phase 2: Background Task Throttle** - Git polling, FileWatcher a autosave přestanou zbytečně probouzet render loop
- [ ] **Phase 3: Terminal Isolation** - egui_term repaintuje pouze při příchodu PTY dat, ne periodicky

## Phase Details

### Phase 1: Repaint Gate
**Goal**: Editor nepřekresluje v idle stavu — render loop se probouzí pouze při uživatelském vstupu nebo nových datech z pozadí
**Depends on**: Nothing (first phase)
**Requirements**: RPNT-01, RPNT-02, RPNT-03, RPNT-04
**Success Criteria** (what must be TRUE):
  1. Editor v idle stavu (bez pohybu myší, bez psaní) nezatěžuje CPU nad 1–2 % po dobu 10 s
  2. Minimalizované nebo nezaměřené okno nerepaintuje vůbec nebo max 1x za 5 s
  3. Accesskit feature je odstraněna z Cargo.toml a editor se zkompiluje a spustí bez ní
  4. Při aktivním psaní se FPS nepřekračuje nastavený cap (ne bezpodmínečný `request_repaint()` po každém keystroke)
**Plans**: TBD

### Phase 2: Background Task Throttle
**Goal**: Git polling, file watching a autosave neblokují render loop ani ho zbytečně neprobouzejí
**Depends on**: Phase 1
**Requirements**: BKGD-01, BKGD-02, BKGD-03
**Success Criteria** (what must be TRUE):
  1. Git status refresh probíhá v background threadu — UI thread při refreshi nepauznuje ani netrhlne
  2. Uložení souboru (Ctrl+S) způsobí maximálně jeden repaint pro git refresh, ne sérii 3–10 burst events
  3. Autosave se nespustí, ani nerepaintuje, když soubor nebyl od posledního uložení změněn
**Plans**: TBD

### Phase 3: Terminal Isolation
**Goal**: Terminálový emulátor neprobouzí render loop, pokud na PTY nepřicházejí nová data
**Depends on**: Phase 2
**Requirements**: BKGD-04
**Success Criteria** (what must be TRUE):
  1. Otevřený terminál bez aktivního procesu nezpůsobuje žádné extra repaints oproti zavřenému terminálu
  2. Aktivní příkaz v terminálu (např. `cargo build`) repaintuje UI plynule při příchodu výstupu, ale přestane okamžitě po dokončení
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Repaint Gate | 0/TBD | Not started | - |
| 2. Background Task Throttle | 0/TBD | Not started | - |
| 3. Terminal Isolation | 0/TBD | Not started | - |
