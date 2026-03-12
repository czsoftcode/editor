# Phase 35: Trash Foundation + Async Safety - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Faze 35 doda zaklad pro interni trash infrastrukturu a non-blocking vykonavani I/O tak, aby vznikl deterministicky `.polycredo/trash` adresar bez hard-delete fallbacku. Scope je omezen na foundation + async safety; UI pro prohlizeni trash a restore workflow patri do nasledujicich fazi.

</domain>

<decisions>
## Implementation Decisions

### Trash creation timing and placement
- `.polycredo/trash` se vytvari on-demand pri prvni relevantni operaci, ne pri otevreni projektu.
- Umisteni je fixni: `.polycredo/trash` (bez konfigurovatelne cesty ve fazi 35).
- Pokud chybi `.polycredo`, vytvari se cela potrebna vetev automaticky.
- Pro fazi 35 se napojeni drzi jen na file-tree delete tok (minimalni patch v boundary faze).

### Trash structure and collision behavior
- V trash se zachovava relativni struktura projektu.
- Pri opakovanem mazani polozek se stejnym projektovym nazvem ma kazda trash polozka vlastni unikatni trash nazev.
- Metadata maji obsahovat vazbu mezi trash nazvem a puvodnim projektovym nazvem/cestou (priprava pro restore faze).
- Pokud je na ceste `.polycredo/trash` soubor misto adresare, operace se zastavi s chybou (zadny auto-fix).

### Async execution and error handling UX
- Vytvareni trash/adresarove pripravy bezi vzdy mimo UI vlakno (konzistentne pres background task pattern).
- Pri chybe create trash se operace zastavi fail-closed: zadny hard-delete fallback.
- Chyby se surfacuji jako error toast s konkretni pricinou; u konfliktu cesty i s doporucenou akci.
- Delete confirm modal se po failu zavre a uzivatel dostane vysledek pres toast; dalsi pokus az pri nove uzivatelske akci (bez auto-retry).

### Claude's Discretion
- Presna podoba interniho metadata formatu (jednotlive nazvy poli) pri zachovani uzamcenych vazeb trash_name <-> puvodni cesta.
- Jemne doladeni textu toast zpravy v ramci existujici i18n konvence.

</decisions>

<specifics>
## Specific Ideas

- Uzivatel chce zachovat relativni strukturu slozek projektu v trash.
- Pokud je polozka smazana opakovane pod stejnym projektnim nazvem, kazda instance ma mit vlastni trash identitu, aby se neztratila moznost obnovy starsi verze.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/background.rs::spawn_task`: existujici pattern pro spousteni I/O mimo UI vlakno.
- `src/app/project_config.rs`: centralni konvence pro `.polycredo` cesty, vhodna pro helper `trash_path`.
- `src/app/ui/file_tree/mod.rs` + `pending_error`: existujici kanal pro surfacing file-tree chyb do toastu.

### Established Patterns
- File-tree dialogy (`src/app/ui/file_tree/dialogs.rs`) dnes delaji sync `remove_file/remove_dir_all`; phase 35 ma prepnout tok na bezpecny foundation krok bez UI blokace.
- Error feedback je uz ve workspace stacku toast-first; success toasty se pouzivaji stridme.
- `.polycredo` je interni projektovy namespace (uz skryvany ve tree a ignorovany v watcheru).

### Integration Points
- Primarni integration point: `src/app/ui/file_tree/dialogs.rs` (delete confirm tok).
- Podporne integration pointy: `src/app/ui/background.rs` (async execution), `src/app/project_config.rs` (path helpery), `src/watcher.rs` (respekt k `.polycredo` ignore chovani).

</code_context>

<deferred>
## Deferred Ideas

- Nahled obsahu trash v UI + restore flow (phase 37).
- Pokrocile retention/cleanup policy a bulk operace (phase 38+/v2).

</deferred>

---

*Phase: 35-trash-foundation-async-safety*
*Context gathered: 2026-03-11*
