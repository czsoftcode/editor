# Phase 5: Okamžité aplikování změny režimu sandboxu po přepnutí checkboxu - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Změna režimu sandboxu se má projevit **bez reopen projektu**. Uživatel přepne sandbox mód v Settings a po **Save** se režim aplikuje okamžitě v běžící instanci.

Mimo scope:
- Nové capability mimo „okamžité přepnutí“ (např. per‑project režimy, nové UI moduly).

</domain>

<decisions>
## Implementation Decisions

### Timing & persistence
- Přepnutí **platí až po Save** (ne při kliknutí checkboxu).
- `Cancel` **vrátí původní režim** a zruší runtime změny.
- Změna sandbox módu je **dirty** změna → vyžaduje `Save/Cancel`.
- Aplikovat ve **všech oknech stejného projektu**.
- **Konflikty mezi okny:** zobrazit upozornění (toast/dialog) při současných změnách.
- **Pořadí:** nejdřív persist na disk, pak runtime přepnutí.
- Pokud **persist selže**, zobrazit toast s volbou „revert / ponechat dočasně“.
- Pokud je otevřený jiný dialog v okamžiku Save, **zeptat se toastem**, zda přepnout teď nebo odložit.
- **Potvrzení** vyžadovat **jen při OFF**.
- **Inline poznámka + tooltip** mají být detailní (zmínit okamžité přepnutí a dopady).
- **Toasty**: oddělené texty pro ON/OFF, upravit pro okamžitou změnu.
- Bez Save se po restartu aplikace **vrátí původní režim**.

### Terminály
- Při přepnutí režimu **restartovat všechny terminály**.
- **Zachovat taby** (restart po tabách).
- **Běžící procesy nechat doběhnout**; přepnutí platí pro nové procesy.
- **Label** režimu měnit **až po restartu terminálu**.

### File tree a otevřené soubory
- File tree **automaticky přepnout** na root odpovídající režimu.
- Při přepnutí se **zeptat** na přemapování otevřených souborů.
- Pokud soubor v cílovém rootu **neexistuje**, tab ponechat otevřený a zobrazit stav „soubor nenalezen“.
- Pokusit se **zachovat rozbalení/selektor** podle relativních cest.

### Staged změny a sync
- Při přepnutí **OFF** a existenci staged souborů **blokovat** přepnutí a **otevřít dialog** pro vyřešení.
- „Sandbox staged“ lišta má být **viditelná i v OFF**, dokud není vyřešeno.
- Při přepnutí **ON** se **zeptat**, zda provést automatický sync projektu do sandboxu.

### Claude's Discretion
- Konkrétní wording inline poznámky, tooltipu a toastů (v rámci rozhodnuté struktury).
- UX detail „toast s volbou“ (např. akční tlačítka vs. modal).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/workspace/modal_dialogs/settings.rs`: sandbox checkbox + Save/Cancel flow.
- `src/app/ui/workspace/state/mod.rs`: `sandbox_mode_enabled`, `build_in_sandbox`, `file_tree_in_sandbox`.
- `src/app/ui/workspace/state/init.rs`: init sandbox režimu z `settings.sandbox_mode`.
- `src/app/ui/terminal/mod.rs`: `terminal_mode_label`, `terminal_working_dir`.
- `src/app/ui/terminal/bottom/build_bar.rs`: restart build terminálu při změně working dir.
- `src/app/ui/panels.rs`: přepínání `file_tree_in_sandbox` + reload tree.
- `src/app/ui/workspace/mod.rs`: sandbox staged bar + staged refresh.
- `src/app/sandbox.rs`: sync plan, staged files, promote.

### Established Patterns
- Settings modal pracuje se snapshotem a `Save/Cancel` revert logikou.
- Runtime změny settings jdou přes `settings_version` a `AppShared.settings`.
- Terminálový working dir je odvozen z `sandbox_mode_enabled`.

### Integration Points
- Sandbox mode propagace do `WorkspaceState` a UI labelů.
- Restart terminálů při změně working dir.
- File tree reload a mapping otevřených tabů.
- Staged dialog + blokace přepnutí OFF.

</code_context>

<specifics>
## Specific Ideas

- Detailní tooltip + inline poznámka o okamžitém přepnutí a dopadech.
- Potvrzení pouze při OFF.

</specifics>

<deferred>
## Deferred Ideas

- Nabídka volby režimu při otevření nového projektu (mimo scope Phase 5).

</deferred>

---

*Phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu*
*Context gathered: 2026-03-05*
