# Phase 29: Syntect Theme Mapping - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Nastavit mapování syntax highlighting témat (syntect) tak, aby každá light varianta i každá dark varianta měla správně odlišný a konzistentní vizuální výstup.

Scope je pouze mapování a jeho verifikace (SYNTAX-01, SYNTAX-02), bez přidávání nových UI capability.

</domain>

<decisions>
## Implementation Decisions

### Unikátnost mapování
- Každá varianta musí mít unikátní syntect mapování.
- Týká se všech 4 light variant a 2 dark variant.

### Vizuální charakter mapování
- Rozdíly mezi variantami mají být jemné a konzistentní (ne agresivní stylové skoky).
- WarmIvory je baseline pro neutrální light vzhled.
- WarmTan má mít teplejší a měkčí charakter syntaxe.
- U dark páru: `Default` zůstává neutrální, `Midnight` má být chladnější (modřejší tón), ale stále čitelný.

### Výběr syntect témat
- Použít pouze osvědčená built-in témata z `ThemeSet` (bez custom definic).

### Fallback chování
- Pokud mapované syntect téma není dostupné, použít bezpečný fallback.
- Fallback nesmí způsobit pád aplikace.
- Chování musí zanechat warning log (ne tiché selhání).

### Testovací kontrakt
- Zavést pevný unit gate, který kontroluje:
  - unikátnost mapování všech 6 variant,
  - validitu/funkčnost fallback chování.

### Checkpoint rozhodnutí (WarmTan)
- Požadavek 4/4 unikátních light mapování zůstává tvrdě závazný.
- V rámci fáze 29 se má vybrat konkrétní 4. built-in light kandidát z `ThemeSet::load_defaults()`.
- Pokud takový kandidát nebude dostupný, je to blocker k vyřešení v rámci fáze (ne povolená výjimka).

### Claude's Discretion
- Konkrétní názvy built-in syntect témat pro jednotlivé varianty (v rámci výše uvedených omezení).
- Přesné rozložení testů mezi jednotkové testy a případné smoke/regression testy.

</decisions>

<specifics>
## Specific Ideas

- Cíl je sjednotit syntax highlighting s paletou variant bez rušivých skoků mezi variantami.
- Preferována je stabilita a čitelnost před experimentálními barevnými styly.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/settings.rs::syntect_theme_name()` je centrální integrační bod mapování (aktuálně příliš hrubé dark/light větvení).
- `src/settings.rs::LightVariant` a `DarkVariant` už obsahují varianty potřebné pro Phase 29 (`WarmIvory`, `CoolGray`, `Sepia`, `WarmTan`, `Default`, `Midnight`).
- `src/highlighter.rs` používá `theme_name` jako součást cache klíče a zpracování highlightu.

### Established Patterns
- Theme rozhodnutí se dělá přes `Settings` a propaguje se do UI/editoru centrálně.
- Editor render path už používá `settings.syntect_theme_name()` (např. `src/app/mod.rs`, `src/app/ui/editor/ui.rs`).
- Projekt preferuje konzervativní, test-first úpravy bez přidávání nových heavy závislostí.

### Integration Points
- `src/settings.rs` (primární změna mapovací logiky a testů).
- `src/app/mod.rs` a editor render (`src/app/ui/editor/ui.rs`) jako spotřebitelé výstupu `syntect_theme_name()`.
- `src/highlighter.rs` jako runtime místo, kde se mapované téma projeví v highlightingu.

</code_context>

<deferred>
## Deferred Ideas

- Vlastní editor témat (custom authoring) je mimo scope této fáze.
- Per-file override syntax tématu je mimo scope (budoucí THEME-EXT).

</deferred>

---

*Phase: 29-syntect-theme-mapping*
*Context gathered: 2026-03-10*
