# Phase 12: I18n Cleanup & Integrity Verification - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit ~215 sandbox i18n klíčů ze všech 5 jazyků (cs, en, de, ru, sk), opravit compile warnings, implementovat runtime cleanup .polycredo/sandbox/ adresářů a ověřit kompletní funkčnost editoru bez sandbox režimu. Editor musí kompilovat bez warningů a všechny testy musí procházet.

</domain>

<decisions>
## Implementation Decisions

### I18n čištění
- Smazat všech ~215 sandbox referencí ze všech 5 jazyků najednou (ne po jednom)
- Čistý řez — smazat klíče + osiřelé komentáře/sekce které sloužily jen sandbox klíčům
- Před smazáním grep src/ na všechny sandbox klíče — dvojitá kontrola že žádný kód neodkazuje na smazaný klíč
- Po smazání spustit test all_lang_keys_match_english pro ověření parity mezi jazyky
- Soubory: locales/{cs,en,de,ru,sk}/ui.ftl a locales/{cs,en,de,ru,sk}/ai.ftl

### Runtime cleanup .polycredo/sandbox/
- Při startu editoru detekovat .polycredo/sandbox/ a smazat celý adresář (rm -rf)
- Implementovat jako background task — nebude blokovat startup
- Po smazání zobrazit info toast ("Sandbox adresář vyčištěn" nebo obdobně)
- Adresář .polycredo/ samotný ponechat (může obsahovat jiné věci)

### Warning cleanup
- Smazat mrtvý kód (ne podtržítko prefix): unused import restore_runtime_settings_from_snapshot, unused id_salt, unused egui_ctx
- Zjistit proč id_salt a egui_ctx nejsou použity — smazat nebo opravit použití
- Po každém kroku spustit cargo check a opravovat nové warnings průběžně
- Cíl: 0 warnings při cargo build

### Verifikace kompletnosti
- cargo test — všech 57+ testů musí projít
- cargo build bez warningů
- grep -r 'sandbox' src/ locales/ = 0 výsledků (kompletní odstranění)
- Sandbox komentáře v src/ smazat — git historie je dostatečná dokumentace

### Claude's Discretion
- Pořadí kroků (i18n první vs warnings první)
- Přesná formulace info toastu pro sandbox cleanup
- Technická implementace background cleanup tasku

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Test `all_lang_keys_match_english` — existující test ověřuje paritu i18n klíčů mezi jazyky
- Background task pattern v background.rs — existující vzor pro asynchronní operace při startu

### Established Patterns
- Phase 9/10/11 vzor: agresivní mazání + fix kompilace
- I18n soubory používají Fluent format (.ftl) — klíče jsou řádek po řádku
- Toast systém přes AppAction::ShowToast — existující mechanismus pro info toasty

### Integration Points
- `locales/{cs,en,de,ru,sk}/ui.ftl` — 44-47 sandbox referencí per jazyk
- `locales/{cs,en,de,ru,sk}/ai.ftl` — 1 sandbox reference per jazyk
- `src/app/ui/workspace/modal_dialogs.rs:15` — unused import k odstranění
- `src/app/ui/workspace/modal_dialogs/settings.rs:130` — unused id_salt
- `src/app/ui/background.rs:33` — unused egui_ctx
- `src/app/mod.rs` nebo background.rs — místo pro runtime sandbox cleanup

</code_context>

<specifics>
## Specific Ideas

- Sandbox cleanup má být tichý background task s info toastem — uživatel se o sandbox nemá starat
- Grep kontrola nulových sandbox referencí je finální verifikace celého v1.1.0 milestonu
- I18n klíče smazat ze všech jazyků najednou — synchronizace je kritická

</specifics>

<deferred>
## Deferred Ideas

None — toto je poslední fáze sandbox removal milestonu

</deferred>

---

*Phase: 12-i18n-cleanup-integrity-verification*
*Context gathered: 2026-03-05*
