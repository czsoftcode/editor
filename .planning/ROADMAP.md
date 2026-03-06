# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- ✅ **v1.1.0 Sandbox Removal** — Phases 9-12 (shipped 2026-03-06)
- 🚧 **v1.2.0 AI Chat Rewrite** — Phases 13-17 (in progress)

## Phases

<details>
<summary>✅ v1.0.2 Dark/Light Mode (Phases 1-5) — SHIPPED 2026-03-05</summary>

- [x] Phase 1: Základ — 4/4 plans — completed 2026-03-04
- [x] Phase 2: Terminal + Git barvy — 2/2 plans — completed 2026-03-04
- [x] Phase 3: Light varianty + Settings UI — 5/5 plans — completed 2026-03-05
- [x] Phase 4: Infrastructure — 2/2 plans — completed 2026-03-05
- [x] Phase 5: Okamžité aplikování sandbox režimu — 4/4 plans — completed 2026-03-05

Archive: `.planning/milestones/v1.0.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.0.6 Focus Management (Phase 6) — SHIPPED 2026-03-05</summary>

- [x] Phase 6: Docked Terminal Focus Suppression — 1/1 plans — completed 2026-03-05
- ~~Phase 7: Float Terminal Focus Suppression~~ — cancelled (covered by Phase 6)
- ~~Phase 8: Focus Restore & Regression Verification~~ — cancelled (covered by Phase 6)

Archive: `.planning/milestones/v1.0.6-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1.0 Sandbox Removal (Phases 9-12) — SHIPPED 2026-03-06</summary>

- [x] Phase 9: Core Sandbox Logic & Settings Removal — 3/3 plans — completed 2026-03-05
- [x] Phase 10: UI & State Cleanup — 1/1 plans — completed 2026-03-05
- [x] Phase 11: File Operations, Watcher & Guard Removal — 2/2 plans — completed 2026-03-05
- [x] Phase 12: I18n Cleanup & Integrity Verification — 2/2 plans — completed 2026-03-05

Archive: `.planning/milestones/v1.1.0-ROADMAP.md`

</details>

### 🚧 v1.2.0 AI Chat Rewrite (In Progress)

**Milestone Goal:** Kompletni prestava AI Chat asistenta — nativni Rust providery misto WASM, terminalovy hybrid UI, plna integrace s editorem (kontext + tools)

- [x] **Phase 13: Provider Foundation** - AiProvider trait, OllamaProvider s NDJSON streaming, auto-detect (completed 2026-03-06)
- [x] **Phase 14: State Refactor** - AiChatState sub-struct, konsolidace ~30 ai_* poli z WorkspaceState (completed 2026-03-06)
- [ ] **Phase 15: Streaming Chat UI** - Hybrid CLI layout, streaming rendering, dark/light mode, markdown, historie, model picker
- [ ] **Phase 16: Tool Execution** - Editor kontext, file read/write tools, command execution, approval UI
- [ ] **Phase 17: i18n & WASM Cleanup** - Nove i18n klice, odstraneni starych WASM klicu, odstraneni WASM plugin systemu

## Phase Details

### Phase 13: Provider Foundation
**Goal**: AI provider abstrakce funguje a komunikuje s Ollama serverem
**Depends on**: Nothing (first phase of v1.2.0)
**Requirements**: PROV-01, PROV-02, PROV-03
**Success Criteria** (what must be TRUE):
  1. AiProvider trait existuje s metodami send_chat(), stream_chat(), name(), available_models()
  2. OllamaProvider dokaze streamovat odpoved z Ollama /api/chat endpointu token po tokenu na background threadu
  3. Editor automaticky detekuje bezici Ollama server na localhost:11434 a zobrazi dostupne modely
  4. Streaming nepblokuje UI thread — editor zustava responzivni behem generovani odpovedi
**Plans:** 3/3 plans complete
Plans:
- [ ] 13-01-PLAN.md — AiProvider trait + OllamaProvider s NDJSON streaming
- [ ] 13-02-PLAN.md — Ollama auto-detect polling + status ikona + model ComboBox v AI baru
- [ ] 13-03-PLAN.md — Gap closure: validace ollama_base_url (odmitne nevalidni URL z plugin settings)

### Phase 14: State Refactor
**Goal**: AI stav je konsolidovany v dedicke strukture, codebase pripraveny pro napojeni provideru na UI
**Depends on**: Phase 13
**Requirements**: CLEN-01
**Success Criteria** (what must be TRUE):
  1. Vsechna ai_* pole z WorkspaceState jsou presunuta do AiChatState sub-structu (ws.ai.*)
  2. Existujici AI chat funkcionalita funguje identicky po refaktoru — zadna regrese
  3. Codebase kompiluje bez warningu po rename
**Plans:** 2/2 plans complete
Plans:
- [x] 14-01-PLAN.md — Vytvorit AiState struct + extrahovat ChatState a OllamaState (completed 2026-03-06)
- [x] 14-02-PLAN.md — Extrahovat AiSettings + top-level pole + finalni overeni (completed 2026-03-06)

### Phase 15: Streaming Chat UI
**Goal**: Uzivatel muze vest konverzaci s AI pres novy nativni chat s plnym streamingem a vizualnim formatovanim
**Depends on**: Phase 13, Phase 14
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04, CHAT-05, CHAT-06, CHAT-07, PROV-04
**Success Criteria** (what must be TRUE):
  1. Chat ma hybrid CLI layout — prompt dole, odpovedi nahore s vizualnim oddelenim
  2. Odpovedi se zobrazuji prubezne token po tokenu (streaming), vcetne markdown formatovani (code blocks, bold/italic)
  3. Chat respektuje dark/light mode — barvy se meni s tematem pres ui.visuals()
  4. Uzivatel muze vybrat model z ComboBoxu s dostupnymi Ollama modely
  5. Konverzacni historie funguje (multi-turn), input ma historii promptu (sipky nahoru/dolu), a uzivatel muze prerusit generovani Stop tlacitkem
  6. Ollama settings (API URL, API Key) jsou v Settings dialogu pod "PolyCredo CLI > Ollama" (ne v plugin settings), zmeny se aplikuji bez restartu editoru
**Plans:** 2/4 plans executed
Plans:
- [ ] 15-00-PLAN.md — Wave 0: Testove scaffoldy pro streaming buffer a settings migraci
- [x] 15-01-PLAN.md — Streaming backend: prepojeni na OllamaProvider + background polling (completed 2026-03-06)
- [ ] 15-02-PLAN.md — Chat UI: theme-aware barvy, barevne bloky, Stop/Send, auto-scroll, model picker
- [ ] 15-03-PLAN.md — AI Settings: Ollama konfigurace v Settings modal + synchronizace

### Phase 16: Tool Execution
**Goal**: AI muze cist/editovat soubory a spoustet prikazy s uzivatelem schvalenym approval workflow
**Depends on**: Phase 15
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05, TOOL-06
**Success Criteria** (what must be TRUE):
  1. AI automaticky vidi editor kontext — otevrene soubory, git stav, build errory — bez manualni akce uzivatele
  2. AI muze cist soubory (s approval) a uzivatel vidi obsah souboru v chatovem kontextu
  3. AI muze upravovat soubory (s approval) a uzivatel vidi diff preview pred schvalenim
  4. AI muze spoustet prikazy (s approval) a uzivatel vidi vystup prikazu
  5. Approval UI nabizi Approve/Deny/Always workflow a AI se muze zeptat uzivatele na upresneni (ask-user tool)
**Plans**: TBD

### Phase 17: i18n & WASM Cleanup
**Goal**: Novy chat je plne lokalizovany a stary WASM plugin system je kompletne odstranen
**Depends on**: Phase 16
**Requirements**: CLEN-02, CLEN-03
**Success Criteria** (what must be TRUE):
  1. Vsechny nove UI retezce maji i18n klice ve vsech 5 jazycich (cs, en, de, ru, sk)
  2. Stare WASM-specificke i18n klice jsou odstraneny
  3. extism dependency a PluginManager jsou kompletne odstraneny (~2000 LOC)
  4. Editor kompiluje a funguje bez WASM runtime — vsechny AI funkce bezi nativne
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Základ | v1.0.2 | 4/4 | Complete | 2026-03-04 |
| 2. Terminal + Git barvy | v1.0.2 | 2/2 | Complete | 2026-03-04 |
| 3. Light varianty + Settings UI | v1.0.2 | 5/5 | Complete | 2026-03-05 |
| 4. Infrastructure | v1.0.2 | 2/2 | Complete | 2026-03-05 |
| 5. Okamžité aplikování sandbox režimu | v1.0.2 | 4/4 | Complete | 2026-03-05 |
| 6. Docked Terminal Focus Suppression | v1.0.6 | 1/1 | Complete | 2026-03-05 |
| ~~7. Float Terminal Focus Suppression~~ | v1.0.6 | — | Cancelled | — |
| ~~8. Focus Restore & Regression~~ | v1.0.6 | — | Cancelled | — |
| 9. Core Sandbox Logic & Settings Removal | v1.1.0 | 3/3 | Complete | 2026-03-05 |
| 10. UI & State Cleanup | v1.1.0 | 1/1 | Complete | 2026-03-05 |
| 11. File Operations, Watcher & Guard Removal | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 12. I18n Cleanup & Integrity Verification | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 13. Provider Foundation | 3/3 | Complete   | 2026-03-06 | - |
| 14. State Refactor | v1.2.0 | Complete    | 2026-03-06 | 2026-03-06 |
| 15. Streaming Chat UI | 2/4 | In Progress|  | - |
| 16. Tool Execution | v1.2.0 | 0/? | Not started | - |
| 17. i18n & WASM Cleanup | v1.2.0 | 0/? | Not started | - |
