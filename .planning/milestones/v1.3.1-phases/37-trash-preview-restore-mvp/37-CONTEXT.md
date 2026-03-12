# Phase 37: Trash Preview + Restore MVP - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Faze 37 doda uzivatelsky nahled obsahu `.polycredo/trash` a obnovu jedne polozky zpet na puvodni cestu, vcetne nedestruktivniho konfliktniho chovani a konzistentniho UI refresh po restore. Scope je omezen na TRASHUI-01 + RESTORE-01/02/03 (bez bulk restore a bez retention cleanup).

</domain>

<decisions>
## Implementation Decisions

### Trash Preview entrypoint a forma
- Primarni vstup do trash preview bude jen pres menu/command (ne pres file tree uzel).
- Trash preview bude v MVP jako modal dialog (ne samostatny panel).
- Preview nabidne restore + detail polozky (minimalne puvodni cesta), ne pouze slepy restore.
- Restore v MVP je pouze pro jednu polozku najednou.

### Obsah trash seznamu
- Vychozi razeni: nejnovejsi smazane polozky nahore.
- Minimalni informace v radku: nazev, typ, cas smazani, puvodni cesta.
- MVP obsahuje jednoduchy textovy filtr (nazev/cesta), bez pokrocilych filtru.
- Pokud je puvodni cesta neplatna/necitliva, polozka zustane v seznamu a bude oznacena varovnym stavem.

### Restore konflikt policy (RESTORE-02)
- Pri existujici cilove ceste se pouzije nedestruktivni default: nabidnout "Obnovit jako kopii" (se suffixem), nikdy tichy overwrite.
- Konfliktni modal nabidne jen: "Obnovit jako kopii" a "Zrusit".
- Pokud chybi parent adresare puvodni cesty, restore je automaticky vytvori.
- Pri restore failu se pouzije toast s duvodem + doporucenym dalsim krokem (stejny UX pattern jako delete flow).

### UI konzistence po restore (RESTORE-03)
- Po uspesnem restore probehne reload file tree a zvyrazneni obnovene polozky na puvodni ceste.
- Obnoveny soubor se nebude automaticky otevirat do tabu.
- Uspesny restore vraci success toast se jmenem/cestou obnovene polozky.
- Refresh bude okamzity lokalni update + watcher dorovnani (ne cekani pouze na watcher).

### Carried Forward z predchozich fazi
- Bez hard-delete fallbacku, fail-closed priorita datove bezpecnosti.
- I/O chyby se surfacuji do toast pipeline.
- Async execution mimo UI vlakno zustava povinna.

### Claude's Discretion
- Presne wordingy preview/restore toastu v i18n, pokud zachovaji dohodnutou informacni strukturu.
- Vizualni hustota tabulky/listu v modalu (spacing, ikonky, drobne UI detaily).
- Presny format suffixu pro "obnovit jako kopii" (pri zachovani nedestruktivni policy).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/trash.rs`: existujici metadata (`TrashEntryMeta`) a delete move engine je pripraveny jako zaklad pro restore mapovani.
- `src/app/ui/file_tree/dialogs.rs`: zavedeny modal pattern a toast error mapping pro file operace.
- `src/app/ui/file_tree/mod.rs`: async polling pattern + pending_error kanal pro toast surfacing.
- `src/app/ui/panels.rs`: centralni misto, kde se file tree vysledky promitaji do editor stavu (napr. close tabs pri delete).
- `src/app/ui/editor/tabs.rs`: utility pro tab synchronizaci po file operacich.

### Established Patterns
- File operace pouzivaji modal confirm + background task + toast result signalizaci.
- `.polycredo` je interni namespace skryvany/ignorovany ve watcheru a tree render flow.
- Toast-first UX pro chyby i/o v file tree je uz etablovany.

### Integration Points
- Trash Preview modal: `src/app/ui/file_tree/dialogs.rs` (nebo navazujici workspace modal orchestrator).
- Restore engine + conflict handling: rozsireni `src/app/trash.rs`.
- UI refresh po restore: `src/app/ui/file_tree/mod.rs`, `src/app/ui/panels.rs`, `src/app/ui/editor/tabs.rs`.
- Event konzistence: `src/watcher.rs` + `src/app/ui/background.rs` v kombinaci s okamzitym lokalnim refresh.

</code_context>

<specifics>
## Specific Ideas

- Preview ma byt rychly a kontextovy: uzivatel musi hned videt, odkud polozka byla smazana.
- Konflikt flow ma byt explicitne nedestruktivni: bez prepisu, s jasnou volbou kopie.
- Po restore ma byt jasne videt, co se stalo (highlight + success toast), ale bez agresivniho auto-open tabu.

</specifics>

<deferred>
## Deferred Ideas

- Bulk restore vice polozek (TRASH-05) patri do dalsi faze.
- Retention/cleanup policy trash (TRASH-06) patri do dalsi faze.
- Full trash timeline/preview/diff mimo MVP scope.

</deferred>

---

*Phase: 37-trash-preview-restore-mvp*
*Context gathered: 2026-03-12*
