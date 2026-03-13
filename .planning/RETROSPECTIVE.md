# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

---

## Milestone: v1.0 — Dark/Light Mode

**Shipped:** 2026-03-05
**Phases:** 3 | **Plans:** 11 | **Commits:** 55
**Timeline:** 2026-03-04 → 2026-03-05 (2 dny)

### What Was Built

- Settings model: `LightVariant` enum + `to_egui_visuals()` + `syntect_theme_name()` + startup apply v `new()`
- Theme-aware syntax highlighting (Solarized Light) s řízenou invalidací cache
- Floating terminal frame + status bar text contrast — theme-aware bez hardcoded barev
- Terminal runtime theming: shared `Terminal::ui()` wrapper volá `set_theme()` každý frame z `ui.visuals()`
- Theme-aware scrollbar + `GitVisualStatus` sémantický model s explicit light/dark paletou pro file tree
- 3-karta light variant picker s live preview (fingerprint guard, `settings_version` bump)
- Snapshot save/cancel semantika pro Settings modal (`settings_original` + `discard_settings_draft()`)
- Tonální přizpůsobení terminálu per-varianta (`warm_ivory_bg()`, blend ratio 0.55)

### What Worked

- **TDD workflow (Red→Green)** — failing testy před implementací fungovaly dobře; testy pokrývají kontrast, palety i roundtrip persistence
- **`settings_version` AtomicU64 jako centrální integrační sběrnice** — elegantní řešení, jedno místo pro propagaci změny tématu do všech viewportů
- **Phase závislosti respektovány** — Phase 1 základ → Phase 2 terminál/git → Phase 3 varianty; žádné cyklické závislosti
- **Gap closure workflow** — 2 UAT gapy (picker layout, WarmIvory terminál) byly identifikovány, naplánovány a uzavřeny v plan 03-05 bez přerušení milestonu
- **Fingerprint guard** — brání zbytečným `settings_version` bumpům; čisté oddělení "theme change" vs "non-theme change"

### What Was Inefficient

- **Phase 01 měla 4 plány místo 2** — gap closure plány (01-03 floating frame, 01-04 status bar kontrast) mohly být součástí původního plánu, kdyby byl scope lépe definován od začátku
- **Manuální GUI smoke test nebyl formalizován** — 4 vizuální body v Phase 03 vyžadují lidské hodnocení, ale nejsou součástí formálního test plánu
- **Nyquist VALIDATION.md draft** — vytvořeny automaticky, ale wave_0 nebyla dokončena v žádné fázi; přidává tech debt

### Patterns Established

- **`ui.visuals()` pro live theming** — čtení aktuálních visuals přímo v render path (ne uložená hodnota) zajišťuje zero-lag theme propagaci
- **`warm_ivory_bg()` pattern** — detekce varianty přes `r - b > threshold` jako lehký heuristický test bez enum přenosu
- **`settings_version` gate v každém frame** — `applied_settings_version != v` jako standard pro lazy-apply update pattern
- **`horizontal_wrapped` pro card picker** — správný layout kontejner pro variable-count karty (ne `horizontal` + `with_layout`)

### Key Lessons

1. **Hardcoded barvy v "terminal-specific" kódu jsou skrytá hrozba** — egui_term, scrollbar, floating frame a status bar všechny měly hardcoded tmavé barvy. Je třeba audit při každém přidání UI komponenty.
2. **`with_layout(right_to_left)` konzumuje zbývající prostor v `horizontal`** — pokud je checkmark uvnitř karty, inline podmíněný label je správnější než layout wrapper.
3. **Fingerprint diff guard ušetří re-render** — porovnání `theme_fingerprint` před a po změně brání kaskádě překreslení.
4. **Deferred viewport apply nastane jeden frame po změně** — je to záměrná vlastnost egui deferred rendering, ne bug; dokumentovat v architektonické poznámce.

### Cost Observations

- Model: sonnet-4.6 (executor + verifier + integration checker)
- Sessions: 1 spojitá práce
- Notable: Paralelní wave execution (phases 03-03 + 03-04) ušetřil čas; gap closure plan 03-05 byl rychlý díky přesnému scope

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Gap Closure Plans | Verification Score |
|-----------|--------|-------|-------------------|--------------------|
| v1.0 | 3 | 11 | 3 (01-03, 01-04, 03-05) | 25/25 requirements |

### Recurring Issues

- Hardcoded barvy v nových UI komponentách → přidat do PR checklist

### What Gets Better Each Time

- (K naplnění po dalším milestonu)

---

*Last updated: 2026-03-05 after v1.0*

---

## Milestone: v1.0.2 — Dark/Light Mode (Phase 5 + Sandbox Runtime)

**Shipped:** 2026-03-05
**Phases:** 5 (přidána Phase 5 k původnímu v1.0) | **Plans:** 17 total | **Commits:** ~14 task commits
**Timeline:** 2026-03-05 (Phase 5, single session)

### What Was Built

- Okamžité sandbox apply po Save: persist → runtime apply flow s revert/keep volbou
- Multi-window detekce přes settings_version + pending_sandbox_apply per viewport
- Runtime restart terminálů (graceful exit starých, nové s target_root), file tree reload
- Tab remap flow: toast s Přemapovat/Nepřemapovávat, cleanup zombie pending_tab_remap po expiraci
- Blokace sandbox OFF při staged souborech, sync dialog při ON, staged bar viditelná v OFF
- Kompletní i18n pokrytí (cs, en, de, ru, sk) pro sandbox-staged/sync/off klíče
- Gap closure: pending_tab_remap zombie fix + dokumentační komentář label-timing

### What Worked

- **Gap closure workflow** — 2 SANDBOX-03 gapy identifikované verifikátorem uzavřeny v plan 05-04 bez rušení milestonu; pattern etablován
- **Verifikátor zachytil zombie stav** — static analysis odhalila, že pending_tab_remap nebylo čištěno po expiraci toastu; dobrý příklad hodnoty automatic verification
- **Checkpoint handling autonomního plánu** — 05-04 s autonomous:true prošel bez bloků; jednoduché dokumentační + cleanup úkoly jsou ideální pro autonomous mode

### What Was Inefficient

- **Phase 5 plány měly [ ] checkboxy** po completion CLI — ROADMAP.md je v .gitignore, takže CLI nemohl committing; checkboxy nebyly automaticky aktualizovány
- **GAP closure dokumentace** (label-timing komentář) je tech debt workaround — lepší by bylo dokumentovat záměrné chování v architektonickém doc od začátku

### Key Lessons

1. **Zombie state po toast expiraci** — každý "pending_X" field čištěný akcí toastu musí mít taky cleanup po retain() pro případ tiché expirace bez kliknutí. Pattern: `if !toasts.iter().any(|t| t.action == X) { pending_x = None }`.
2. **Label-timing dokumentace** — záměrné chování (label se mění před exit PTY) je zdrojem budoucí záměny; dokumentační komentář vedle kódu je levnější než opakované re-verifikace.
3. **.planning v .gitignore** — commit_docs flow přes gsd-tools selže tiše; artifacts jsou lokální. Pokud je project history důležitá, přidat .planning do VCS.

### Cost Observations

- Model: sonnet-4.6
- Sessions: pokračování po v1.0 milestonu
- Notable: Phase 5 gap closure (1 plán, 2 úkoly) byl nejrychlejší plán v celém milestonu

---

## Cross-Milestone Trends (aktualizováno po v1.0.2)

### Process Evolution

| Milestone | Phases | Plans | Gap Closure Plans | Verification Score |
|-----------|--------|-------|-------------------|--------------------|
| v1.0 | 3 | 11 | 3 (01-03, 01-04, 03-05) | 25/25 requirements |
| v1.0.2 | 5 | 17 | 4 (01-03, 01-04, 03-05, 05-04) | 30/30 requirements |

### Recurring Issues

- Hardcoded barvy v nových UI komponentách → přidat do PR checklist
- Zombie "pending_X" fields po toast expiraci → checklist při každém novém pending_ field

### What Gets Better Each Time

- Gap closure workflow je etablovaný: verify → gaps_found → plan-phase --gaps → execute --gaps-only → re-verify
- Verifikátor zachytává runtime-only problémy (zombie state), nejen strukturální gapy

---

## Milestone: v1.1.0 — Sandbox Removal

**Shipped:** 2026-03-06
**Phases:** 4 | **Plans:** 8 | **Commits:** 15 feat/fix
**Timeline:** 2026-02-16 → 2026-03-06 (18 dní)

### What Was Built

- Kompletní odstranění sandbox.rs modulu a všech sandbox datových struktur (Sandbox, SyncPlan)
- Vyčištění UI — settings toggle, modální dialogy, build bar label, file tree sandbox prvky
- Odstranění sandbox logiky z file operations, watcheru a git/build guardů
- Přejmenování sandbox_root → project_root, exec_in_sandbox → exec v plugin systému
- Vyčištění 43+ sandbox i18n klíčů ze všech 5 jazyků

### What Worked

- **Systematická fázová dekompozice** — 4 fáze s jasnou závislostní řetězem (core → UI → file ops → integrity) eliminovaly merge konflikty
- **Gap closure plány (09-03)** — verifikátor odhalil neodstraněné sandbox.rs a sandbox field ve WorkspaceState; gap closure plán vše uzavřel
- **Milestone audit workflow** — 3-source cross-reference (VERIFICATION + SUMMARY + REQUIREMENTS) potvrdil 26/26 requirements bez mezer
- **Čistý refactoring scope** — milestone byl čistě subtraktivní (-2,878 net lines), žádné nové funkce = minimální riziko regresí

### What Was Inefficient

- **Phase 9 potřebovala 3 plány** — sandbox.rs měl hluboké závislosti, které nebyly zřejmé při prvním plánování; lépe mapovat dependencies dopředu
- **ROADMAP.md Phase 9 status** — progress tabulka ukazovala "2/3" místo "3/3" kvůli manuální aktualizaci; CLI neaktualizoval po gap closure

### Patterns Established

- **Subtraktivní refactoring milestone** — čistě odstraňovací milestone s integrity verification fází jako finální pojistkou
- **Plugin API rename flow** — přejmenování API (sandbox_root → project_root) + rebuild WASM pluginů jako samostatný plán

### Key Lessons

1. **Hluboce provázané moduly potřebují dependency mapping** — sandbox.rs měl reference ve ~15 souborech; plán 09-01 předpokládal méně závislostí, což vedlo k gap closure plánu 09-03
2. **i18n cleanup jako samostatná fáze** — oddělení i18n čištění od kódového refactoringu umožnilo čistou verifikaci integrity
3. **Stale komentáře přežijí refactoring** — grep po celém codebase na odstraňovaný termín (sandbox) odhalil 2 komentáře mimo scope fázových plánů

### Cost Observations

- Model: sonnet-4.6 (executor) + opus-4.6 (milestone audit)
- Sessions: ~5 sessions across milestone
- Notable: Subtraktivní milestone je rychlejší než aditivní — méně testování, méně edge cases

---

## Milestone: v1.2.0 — AI Chat Rewrite

**Shipped:** 2026-03-06
**Phases:** 6 | **Plans:** 19 | **Commits:** 74 (42 feat/fix)
**Timeline:** 2026-03-06 (1 den intenzivní práce)

### What Was Built

- AiProvider trait abstrakce + OllamaProvider s NDJSON streaming a auto-detect serveru
- AiState konsolidace — 19 polí z WorkspaceState do dedikované struktury
- Hybrid CLI chat UI se streaming renderingem, markdown formátováním, dark/light mode, model picker
- Tool execution — read/write/exec/search/ask-user s approval workflow a security infrastrukturou
- Kompletní odstranění WASM plugin systému (~6,500 LOC)
- Plná i18n lokalizace CLI chatu v 5 jazycích

### What Worked

- **Trait-first design** — AiProvider trait definovaný v Phase 13 umožnil čistou integraci v dalších fázích bez refaktorování rozhraní
- **State refactor jako early phase** — konsolidace AI stavu v Phase 14 před UI wiringem zabránila widespread renames po napojení UI
- **Security-first approach** — PathSandbox, CommandBlacklist, SecretsFilter implementovány před tool handlery zajistily bezpečnost od počátku
- **Gap closure workflow (Phase 18)** — audit odhalil chybějící VERIFICATION.md a i18n bugy; dedikovaná fáze vše uzavřela
- **Collect-then-process pattern** — řešení borrow checker problémů s mutable access v background polling

### What Was Inefficient

- **Phase 16 bez SUMMARY/VERIFICATION** — 8 commitů provedeno bez dokumentačních artefaktů; vyžadovalo retroaktivní Phase 18
- **i18n hardcoded strings prosákly** — Phase 17 měla pokrýt vše, ale 7+ stringů uniklo; odchyceno až auditem
- **Audit status `gaps_found` po Phase 17** — ideálně by audit měl proběhnout po dokončení všech plánovaných fází, ne jen po i18n

### Patterns Established

- **cli-* i18n namespace** — všechny AI chat a CLI-related klíče používají cli-* prefix
- **AI state access pattern** — ws.ai.chat.*, ws.ai.ollama.*, ws.ai.settings.*
- **Security validation chain** — validate → classify → allow/deny pro tool execution
- **Audit-driven gap closure** — milestone audit → gaps_found → gap closure phase → re-verification

### Key Lessons

1. **SUMMARY.md musí vzniknout s každým plánem** — retroaktivní dokumentace je dražší a méně přesná
2. **i18n audit potřebuje grep na hardcoded strings** — samotné přidání klíčů nestačí; grep na české/anglické literály v src/ je nutný
3. **WASM removal jako poslední fáze funguje** — oba systémy koexistovaly bezpečně; čisté odstranění bez regresí
4. **Security testy jsou investice** — 28 unit testů v security.rs zachytily edge cases při budoucích změnách

### Cost Observations

- Model: sonnet-4.6 (executor), opus-4.6 (audit, milestone completion)
- Sessions: intensive single-day milestone
- Notable: 19 plánů za 1 den — nejvyšší velocity ze všech milestones

---

## Cross-Milestone Trends (aktualizováno po v1.2.0)

### Process Evolution

| Milestone | Phases | Plans | Gap Closure Plans | Verification Score |
|-----------|--------|-------|-------------------|--------------------|
| v1.0 | 3 | 11 | 3 (01-03, 01-04, 03-05) | 25/25 requirements |
| v1.0.2 | 5 | 17 | 4 (01-03, 01-04, 03-05, 05-04) | 30/30 requirements |
| v1.0.6 | 1 | 1 | 0 | 9/9 requirements |
| v1.1.0 | 4 | 8 | 1 (09-03) | 26/26 requirements |
| v1.2.0 | 6 | 19 | 1 (Phase 18) | 20/20 requirements |

### Recurring Issues

- Hardcoded barvy v nových UI komponentách → přidat do PR checklist
- Zombie "pending_X" fields po toast expiraci → checklist při každém novém pending_ field
- Stale komentáře přežijí major refactoring → grep na odstraňovaný termín jako finální krok
- i18n hardcoded strings prosáknou navzdory dedikované i18n fázi → grep audit jako finální krok
- Dokumentační artefakty (SUMMARY.md) nejsou vytvářeny konzistentně → vynutit v execution workflow

### What Gets Better Each Time

- Gap closure workflow je etablovaný: verify → gaps_found → plan-phase --gaps → execute --gaps-only → re-verify
- Verifikátor zachytává runtime-only problémy (zombie state), nejen strukturální gapy
- Milestone audit (3-source cross-reference) zachytí mezery mezi fázemi
- Security-first design — tool execution s approval workflow od prvního commitu
- Trait-based abstrakce umožňuje čistou extensibilitu (AiProvider pro budoucí providery)

---

*Last updated: 2026-03-06 after v1.2.0*

---

## Milestone: v1.2.2 — Additional Themes

**Shipped:** 2026-03-11
**Phases:** 3 | **Plans:** 5 | **Tasks:** 15
**Timeline:** 2026-03-10 → 2026-03-11 (2 dny)

### What Was Built

- 4. light varianta WarmTan (picker, swatch, persistence, i18n)
- 2. dark varianta Midnight (picker, persistence)
- Syntect mapovani pro light/dark varianty + regresni testy

### What Worked

- Regresni testy drzely viditelnost a lokalizaci variant
- Theme fingerprint okamzite aplikoval preview pri prepnuti rezimu

### What Was Inefficient

- Re-exekuce planu 27-02 kvuli regresi viditelnosti pickeru
- Layout pickeru byl na zacatku nejasny (nutne UI upravy)

### Patterns Established

- Light/dark picker podle aktivniho rezimu
- Centralni mapovani syntect theme na varianty

### Key Lessons

1. Rozlozeni pickeru musi garantovat viditelnost vsech variant (2x2 grid je citelnejsi).
2. Dark varianty nesmi implicitne vynucovat dark rezim pri light prepnuti.

### Cost Observations

- Model mix: n/a
- Sessions: n/a
- Notable: kratky milestone, ale s jednou re-exekuci kvuli UI regresi

---

## Cross-Milestone Trends (aktualizovano po v1.2.2)

### Process Evolution

| Milestone | Phases | Plans | Gap Closure Plans | Verification Score |
|-----------|--------|-------|-------------------|--------------------|
| v1.2.2 | 3 | 5 | 1 (27-02 re-exekuce) | n/a |

### Recurring Issues

- UI layouty mohou skryt varianty, pokud nejsou grid-guarded

### What Gets Better Each Time

- Regresni testy u theme pickeru odchytavaji zmeny brzo
