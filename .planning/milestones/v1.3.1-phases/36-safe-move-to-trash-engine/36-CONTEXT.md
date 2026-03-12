# Phase 36: Safe Move-to-Trash Engine - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Faze 36 meni delete tok na bezpecny move-to-trash pro soubory i adresare a doplnuje fail-safe chovani pri chybach bez ztraty dat. Scope je uvnitr delete workflow (TRASH-01, TRASH-02, TRASH-04, RELIAB-02) bez pridavani restore UI nebo dalsich novych capability.

</domain>

<decisions>
## Implementation Decisions

### Kolizni policy v `.polycredo/trash`
- Pri kolizi nazvu/cesty se pouzije automaticky unikatni suffix (nedestruktivni default).
- Format suffixu: timestamp + counter (citelný a trasovatelny).
- Zachovava se puvodni relativni struktura slozek (nezplostovat do rootu).
- Pri kolizi uvnitr presouvaneho adresare je povoleny kompletni prubeh operace: konflikty dostanou suffix, operace nema failnout cela.

### Delete adresaru
- Pri delete adresare se presouva cely strom vcetne skrytych souboru.
- Metadata vzdy drzi puvodni relativni cestu korene adresare (priprava pro restore fazi).
- Pokud selze cast presunu adresare z duvodu I/O/permission, delete failne jako celek (fail-safe proti ztrate dat).
- Pokus o smazani `.polycredo/trash` se blokuje a surfacuje toastem.

### UX chyb v delete toku
- Error toast je strucny, ale obsahuje konkretní pricinu selhani.
- Toast obsahuje kratke doporuceni dalsi akce (napr. opravnni/zamceny soubor/zkusit znovu).
- Pri vice selhanich v sekvenci preferovat souhrnny toast (ne spam jednoho toastu na kazdou polozku).
- Delete confirm modal se po chybe zavira; bez auto-retry (navazuje na Phase 35).

### Carried Forward z predchozich fazi
- `.polycredo/trash` se vytvari on-demand a je fixni interni cesta.
- Delete tok je bez hard-delete fallbacku.
- Chyby se reportuji toast-first pipeline a I/O beha mimo UI vlakno.

### Claude's Discretion
- Presny text tostu a i18n wording v ramci rozhodnute struktury.
- Presna serializace metadat pro kolizni suffix (nazvy poli), pokud zachova rozhodnute behavior kontrakty.
- Jak presne agregovat detail v souhrnnem tostu (format poctu/struchny detail).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/trash.rs`: foundation helpery z Phase 35 (`ensure_trash_dir`, `move_path_to_trash`) pro rozsireny move engine.
- `src/app/project_config.rs`: centralni helpery cest `.polycredo`.
- `src/app/ui/file_tree/dialogs.rs`: delete entrypoint + toast surfacing.
- `src/app/ui/file_tree/mod.rs` + `pending_error`: existujici kanal pro error -> toast.
- `src/app/ui/background.rs::spawn_task`: zavedeny async pattern mimo UI vlakno.

### Established Patterns
- Toast-first UX pro file tree chyby.
- Fail-closed semantika zavedena ve fazi 35 (bez hard-delete fallbacku).
- `.polycredo` namespace je interni a ma specialni zachazeni ve watcher/file tree vrstvach.

### Integration Points
- Primarni: `src/app/ui/file_tree/dialogs.rs` a `src/app/trash.rs`.
- Podporne: `src/app/ui/file_tree/mod.rs`, `src/app/project_config.rs`, `src/watcher.rs`.
- Testy: `tests/phase35_*` jako zaklad pro phase36 regression rozsireni.

</code_context>

<specifics>
## Specific Ideas

- Kolizni nazvy maji zustat citelne a dohledatelne (timestamp + counter).
- U vice selhani je preferovany agregovany signal misto toast spamu.
- Bezpecnost dat ma prioritu nad "all-or-nothing" striktnosti u kolizi (kolize se resi suffixem).

</specifics>

<deferred>
## Deferred Ideas

- Restore flow/UI a conflict policy pri restore zustavaji ve fazi 37.
- Pokrocily retention/cleanup policy a bulk trash operace zustavaji v pozdejsich fazich.

</deferred>

---

*Phase: 36-safe-move-to-trash-engine*
*Context gathered: 2026-03-12*
