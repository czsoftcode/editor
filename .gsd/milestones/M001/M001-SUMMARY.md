---
id: M001
provides:
  - Theme systém s dark/light mode a 3 light variantami (WarmIvory, CoolGray, Sepia)
  - Theme-aware terminal rendering bez restartu PTY backendu
  - Settings TOML persistence s legacy JSON migrací
  - Sandbox režim s plným runtime lifecycle (persist → apply → restart → remap)
  - Slash command infrastruktura se 7 built-in příkazy
  - GSD core state engine s YAML-like frontmatter parserem
key_decisions:
  - "LightVariant enum se 3 variantami centralizovaný v Settings"
  - "Terminal theme resolver napojený na visuals — set_theme() per-frame bez restartu"
  - "Canonical storage settings.toml; settings.json legacy smazán po migraci"
  - "Sandbox apply přes pending_sandbox_apply po persistu, ne okamžitě"
  - "Slash dispatch match-based se static slice registry"
  - "Custom YAML-like parser bez externích závislostí s round-trip fidelitou"
patterns_established:
  - "Theme fingerprint pro detekci reálné změny — zbytečné repainty eliminovány"
  - "Snapshot-based cancel/discard flow pro Settings modal"
  - "SlashResult::Async + generation counter pro background příkazy"
  - "FmDocument parse → get/set → to_string_content round-trip vzor"
  - "spawn_task pro asynchronní operace bez blokace UI vlákna"
  - "TempConfigDir pro izolované persistence testy"
observability_surfaces:
  - "/gsd state — milestone, phase, status, progress bar, velocity, blockers"
  - "/gsd progress — kompaktní progress bar s Unicode bloky"
requirement_outcomes: []
duration: ~4 dny (2026-03-04 až 2026-03-07)
verification_result: passed
completed_at: 2026-03-07
---

# M001: Migration

**Multiplatformní textový editor obohacen o dark/light theme systém se 3 variantami, sandbox runtime lifecycle, slash command infrastrukturu a GSD state engine.**

## What Happened

M001 byl migrovaný milestone, který pokryl 7 slicí v logické posloupnosti:

**S02 (Základ)** položil datový model — LightVariant enum, centralizovaná theme logika v Settings s `to_egui_visuals()` delegací a `syntect_theme_name()` pro syntax highlighting.

**S04 (Terminal Git Barvy)** napojil terminálový rendering na aktivní visuals — background, scrollbar, kurzor i git barvy reagují na light/dark mode za běhu bez restartu PTY backendu. Řešení centralizováno v StandardTerminalWindow pro automatickou propagaci do AI i build terminálu.

**S05 (Light Varianty Settings UI)** dodala plný lifecycle light variant: explicitní palety pro WarmIvory/CoolGray/Sepia, picker UI jako clickable karty, live preview s fingerprint-based invalidací, TOML persistence s legacy JSON migrací, snapshot-based Save/Cancel flow, variant-aware terminal paletu (panel_fill blending 0.55) a tonálně adaptované git barvy. 55+ unit testů.

**S07 (Infrastructure)** zavedla sandbox režim do settings.toml s UI přepínačem, tooltipem a apply-on-reopen sémantikou. Terminály a build bar respektují režim pro cwd i label.

**S08 (Sandbox Runtime Apply)** vybudovala plný runtime workflow: Save→persist→apply s multi-window propagací, graceful restart terminálů, tab remap s varováním u chybějících souborů, blokace OFF při staged souborech a sync dialog při ON. Kompletní i18n pro 5 jazyků.

**S10 (Slash Command Infrastructure)** dodala command registry se 7 příkazy (/help, /clear, /new, /settings, /model, /git, /build), async vzor s generation counter, Levenshtein fuzzy matching, code-fence aware rendering a autocomplete popup s keyboard navigací.

**S11 (GSD Core State Engine)** implementovala custom YAML-like frontmatter parser s plnou round-trip fidelitou, GSD subcommand dispatch, config management s dot-notation a /gsd state/progress příkazy. 75 GSD testů.

## Cross-Slice Verification

Roadmap neobsahoval žádná formální success criteria (sekce byla prázdná). Verifikace provedena na úrovni definice done:

- **Slices:** 7/7 dokončených (`[x]`) — S02, S04, S05, S07, S08, S10, S11
- **Slice summaries:** 7/7 existují
- **Build:** `cargo check` čistý
- **Testy:** Všechny testy prochází kromě 1 předexistujícího (`phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols`) — hledá soubor z v1.3.1 milestone, který byl odstraněn. Tento test je známý tech debt mimo scope M001.
- **Cross-slice integrace:** Theme systém (S02→S04→S05) je koherentní — Settings model se propisuje do terminálu a git barev. Sandbox (S07→S08) je kompletní lifecycle. Slash/GSD (S10→S11) sdílejí dispatch pattern.

## Requirement Changes

M001 byl migrovaný milestone bez formálních requirements. Žádné requirement transitions neproběhly. Existující aktivní požadavky z AGENTS.md backlogu (V-1, V-2, K-1, S-3, N-5, S-1, S-4, V-3) nebyly scope tohoto milestone a zůstávají nezměněné.

## Forward Intelligence

### What the next milestone should know
- Theme systém je kompletní a stabilní — další varianty lze přidat rozšířením LightVariant enum a match armu v to_egui_visuals()
- Sandbox runtime lifecycle je plný — nové sandbox-aware funkce mohou navazovat na stávající pending_sandbox_apply vzor
- Slash command systém je rozšiřitelný — nový příkaz = nový arm v dispatch + záznam v COMMANDS static array
- GSD FmDocument API je základ pro všechny příkazy pracující s .planning/ markdown soubory

### What's fragile
- warm_ivory_bg() threshold (r-b > 10) — budoucí light varianty mohou být na hraně detekce
- pending_tab_remap cleanup závisí na toast retain() pořadí — změna toast lifecycle může rozbít
- Generation counter (slash_conversation_gen) musí se vždy bumpnout při /clear — jinak stale výsledky proniknou
- Dot-notation ve FmDocument omezena na 2 úrovně zanoření

### Authoritative diagnostics
- `settings.rs` testy — theme mapování, persistence round-trip, migrace
- `theme.rs` testy — luminance, kontrast, variant distinctness
- `frontmatter.rs` testy — 36 testů parser edge cases
- `state.rs` testy — 18 testů state/progress příkazy
- `slash.rs` testy — dispatch, fuzzy matching
- `/gsd state` a `/gsd progress` — runtime diagnostika ve editoru

### What assumptions changed
- Restart PTY pro theme změnu zbytečný — set_theme() per-frame stačí
- serde_yaml nepotřebný — custom parser je jednodušší a bez závislosti
- run_build_check pro /build reuse — nová implementace zbytečná
- Sandbox apply-on-reopen nahrazen runtime apply v S08 — lepší UX

## Files Created/Modified

- `src/settings.rs` — LightVariant, theme metody, sandbox_mode, TOML persistence
- `src/app/ui/terminal/instance/theme.rs` — terminal theme resolver, tone_light_palette
- `src/app/ui/git_status.rs` — GitVisualStatus, variant-aware barvy
- `src/app/ui/workspace/modal_dialogs/settings.rs` — variant picker, sandbox toggle, Save/Cancel
- `src/app/ui/workspace/state/mod.rs` — sandbox pending struktury, runtime apply
- `src/app/ui/workspace/mod.rs` — process_pending, sync confirmation
- `src/app/ui/terminal/ai_chat/slash.rs` — command registry, dispatch, autocomplete
- `src/app/ui/terminal/ai_chat/gsd/` — frontmatter parser, dispatch, config, paths, state
- `src/app/ui/widgets/ai/chat/` — system message rendering, input autocomplete
- `src/app/ui/background.rs` — async slash result polling
- `locales/*/ui.ftl` — i18n pro sandbox a slash příkazy
