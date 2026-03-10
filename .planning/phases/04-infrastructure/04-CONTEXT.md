# Phase 4: Infrastructure - Context

**Gathered:** 2026-03-05
**Updated:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Základní přepínač sandbox režimu v Settings > Projekt a jeho napojení na terminály a init projektu. OFF znamená práce v root projektu a terminály v rootu; změna se projeví po znovuotevření projektu.

</domain>

<decisions>
## Implementation Decisions

### Chování přepínače
- Změny se projeví až po znovuotevření projektu a všechno najednou (žádné dílčí přepínání v běžící instanci).
- Přepínač respektuje Save/Cancel v Settings (uloží se až po Save).
- Inline poznámka pod přepínačem vysvětlí, že změna se projeví po znovuotevření projektu.

### Texty v Settings
- Label: "Režim sandboxu" (CZ) / "Sandbox mode" (EN).
- Tooltip popíše OFF režim jen v rozsahu Phase 4: práce v rootu, terminály v rootu, změna se projeví po reopen.

### Terminály
- Po přepnutí režimu a znovuotevření projektu zachovat terminálové taby (spustit nové PTY v novém rootu/sandboxu).
- Sandbox ON: label "Sandbox".
- Sandbox OFF: label pouze "Terminál" (bez cesty).
- Krátká poznámka, že při změně režimu se terminálové procesy po znovuotevření znovu spustí.

### Informování uživatele
- Informace o OFF režimu primárně v tooltipu u přepínače (bez zmínek o UI/sync).
- Jednorázový toast při prvním vypnutí sandboxu.
- Krátký toast při znovuzapnutí sandboxu.

### Claude's Discretion
- Přesné znění tooltipu, inline poznámky a toastů.
- Drobné typografické doladění labelu "Terminál" (bez cesty).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- src/app/ui/workspace/modal_dialogs/settings.rs: Settings UI (Save/Cancel pattern, lokalizované texty).
- src/settings.rs: persistování settings.toml, napojení na změny nastavení.
- src/app/ui/workspace/state/init.rs: init workspace (build_in_sandbox / file_tree_in_sandbox).

### Established Patterns
- Settings modal používá snapshot Save/Cancel semantiku.
- Lokalizace přes locales/cs/ui.ftl a locales/en/ui.ftl.

### Integration Points
- src/app/ui/terminal/bottom/build_bar.rs: sandbox ovládací prvky a labely.
- src/app/ui/terminal/mod.rs a src/app/ui/terminal/instance/*: terminálový label a pracovní adresář.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

- Rozšířit tooltip o zmínku skrytí sandbox UI prvků a vypnutí syncu v Phase 5.
- Okamžité aplikování změny režimu sandboxu už při přepnutí checkboxu (mimo scope Phase 4).

</deferred>

---

*Phase: 04-infrastructure*
*Context gathered: 2026-03-05*
*Context updated: 2026-03-05*
