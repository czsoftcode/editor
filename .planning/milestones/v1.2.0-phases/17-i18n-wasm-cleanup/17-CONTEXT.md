# Phase 17: i18n & WASM Cleanup - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Novy AI chat (PolyCredo CLI) je plne lokalizovany a stary WASM plugin system je kompletne odstranen. Zahrnuje: nove i18n klice pro vsechny hardcoded retezce, prejmenovani existujicich klicu (ai-chat-* -> cli-chat-*), odstraneni WASM plugin systemu (extism, PluginManager, plugin source, samples), pridani Ollama parametru do Settings.

</domain>

<decisions>
## Implementation Decisions

### Plugin Manager UI — kompletni odstraneni
- Menu polozka "Plugins" a "Plugin Manager" se KOMPLETNE odstrani — zadna nahrada
- AI nastaveni uz jsou v Settings dialogu (Phase 15), neni potreba duplicitni UI
- Vsechny command palette prikazy plugin-* se ODSTRANI (Plugin: Say Hello, Plugin: Ask Gemini, Plugin: Ask Ollama, Plugin: Ask AI Agent)
- Plugin Permissions bar (plugin-auth-bar-*) se ODSTRANI
- Plugin Error dialog (plugin-error-*) se ODSTRANI
- AI bar klice se PREJMENUJOU: ai-plugin-bar-* -> cli-bar-* (konzistentni s nazvem PolyCredo CLI)

### Settings prejmenovani
- "Plugin Blacklist (blocked files)" -> "CLI Blacklist"
- AI sekce v Settings dialogu se PREJMENUJE z "AI"/"Ollama" na "PolyCredo CLI"
- Stare plugin settings se SMAZOU (uz migrovany v Phase 15)
- PRIDAT Ollama parametry do PolyCredo CLI sekce:
  - Temperature (0.0-2.0)
  - Context window size (num_ctx)
  - Top-P / Top-K
  - + dalsi uzitecne parametry dle Ollama API (Claude's discretion)

### Lokalizace novych retezcu
- Prefix pro tool-related klice: cli-tool-* (cli-tool-approve, cli-tool-deny, cli-tool-network-warning, ...)
- Existujici ai-chat-* klice se PREJMENUJOU na cli-chat-* (15 klicu ve vsech 5 jazycich)
- NOVY soubor cli.ftl ve vsech 5 jazycich (cs, en, de, ru, sk) — vsechny CLI klice se presunou sem
- Stary ai.ftl se SMAZE (obsahuje jen 5 gemini-specific klicu)
- Hardcoded ceske retezce v approval.rs ("Odeslat", "Zrusit", "Sitovy prikaz") se nahradi i18n klici
- Hardcoded anglicke retezce v render.rs ("Stop"), inspector.rs ("Clear", "Copy") se nahradi i18n klici

### WASM odstraneni — kompletni
- Smazat src/app/registry/plugins/ (~2119 LOC): PluginManager, host functions, types
- Smazat src/plugins/ (~822 LOC): ollama/, gemini/, hello/ WASM plugin source
- Smazat docs/samples/hello-plugin/ — WASM sample
- Odstranit extism dependency z Cargo.toml
- Smazat vsechny plugin-related i18n klice z ui.ftl (~30+ klicu: plugins-*, plugin-*, command-name-plugin-*)
- Aktualizovat registry/mod.rs — odstranit WASM references, ponechat jen nativni AI logiku

### Claude's Discretion
- Ktere dalsi Ollama API parametry pridat do Settings (repeat_penalty, seed, mirostat, ...)
- Presne pojmenovani novych i18n klicu (pri dodrzeni prefixu cli-tool-* a cli-chat-*)
- Poradi odstranovani WASM kodu (dependency graph)
- Jak vycistit registry/mod.rs po odstraneni WASM pluginu

</decisions>

<specifics>
## Specific Ideas

- AI chat se JMENUJE "PolyCredo CLI" — vsechny nazvy, klice a UI musi byt konzistentni s timto nazvem
- cli-bar-* prefix pro bar, cli-chat-* pro chat UI, cli-tool-* pro tool-related retezce
- Novy cli.ftl soubor misto rozprostireni klicu do ui.ftl
- Ollama parametry (temperature, context, top-p/top-k) patri do Settings, ne do kodu

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `locales/{lang}/ui.ftl`: Stávající i18n klíče — ~30+ plugin klíčů k odstranění, ~15 ai-chat-* klíčů k přejmenování
- `locales/{lang}/ai.ftl`: 5 gemini-specific klíčů — celý soubor ke smazání
- `src/app/ui/terminal/ai_chat/approval.rs`: Hardcoded české řetězce k nahrazení i18n klíči
- `src/app/ui/terminal/ai_chat/render.rs`: Hardcoded "Stop" a model info řetězce
- `src/app/ui/terminal/ai_chat/inspector.rs`: Hardcoded "Clear", "Copy"

### Established Patterns
- fluent i18n systém: `i18n.get("key")` pattern pro lokalizované řetězce
- FTL soubory per-kategorie: dialogs.ftl, menu.ftl, ui.ftl, errors.ftl — nový cli.ftl zapadá
- Test `all_lang_keys_match_english` hlídá paritu klíčů mezi jazyky
- 5 jazyků: cs, en, de, ru, sk

### Integration Points
- `src/app/registry/mod.rs`: Hlavní registr — po WASM removal zůstane jen nativní AI logika
- `src/app/ui/workspace/menubar/mod.rs`: Menu — odstranit Plugins položku
- `src/settings.rs`: Settings struct — přidat Ollama parametry, odstranit plugin settings
- `Cargo.toml`: Odstranit extism dependency
- `src/app/mod.rs`: Plugin inicializace — odstranit WASM bootstrap

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 17-i18n-wasm-cleanup*
*Context gathered: 2026-03-06*
