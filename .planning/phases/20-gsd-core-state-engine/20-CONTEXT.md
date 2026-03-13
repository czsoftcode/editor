# Phase 20: GSD Core + State Engine - Context

**Gathered:** 2026-03-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Frontmatter parser, config management, path helpers a state/progress slash commands (`/gsd state`, `/gsd progress`). Tvoří základ (core) pro všechny další GSD fáze (21-23). Uživatel může z chat panelu dotazovat a aktualizovat stav GSD projektu.

Requirements: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, STATE-01, STATE-02, STATE-03, STATE-04, STATE-05

</domain>

<decisions>
## Implementation Decisions

### GSD subcommand routing
- Samostatný GSD modul v `src/app/cli/gsd/` — slash.rs předá vše za `/gsd` do GSD dispatch modulu
- `/gsd` bez argumentů = `/gsd help` (zobrazí seznam GSD subcommandů)
- V `/help` výpisu jen jeden řádek: `/gsd — GSD project management (type /gsd help for subcommands)`
- Detail subpříkazů až v `/gsd help`
- Oddělovač subcommandů: **mezera** (`/gsd state`, `/gsd progress`, ne dvojtečka ani pomlčka)

### Frontmatter parser
- **Custom parser** — žádná nová závislost (serde_yaml), v souladu s "zero new dependencies" rozhodnutím
- **Plný YAML subset**: string, integer, float, boolean, list, nested map, multi-line stringy, quoted stringy, inline listy/mapy
- **Tolerantní error handling**: parsuje co jde, ignoruje nevalidní řádky, vrátí částečný výsledek + warnings
- **Plný round-trip**: zachová komentáře, whitespace, inline formátování, pořadí klíčů — AST s pozicemi
- Soubor v `src/app/cli/gsd/frontmatter.rs`

### State/Progress výstup
- `/gsd state` zobrazí **detailní přehled**: milestone, phase, status, last activity, progress bar, velocity, blockers, tech debt
- `/gsd progress` je **samostatný příkaz** — zobrazí POUZE progress bar + tabulku fází se statusem (rychlý přehled)
- `/gsd state update` funguje **synchronně** (STATE.md je malý soubor, sub-ms zápis)
- Argumenty přes **dot-notation**: `/gsd state update status executing`, `/gsd state update progress.completed_phases 20`
- `/gsd state patch` pro batch update více polí najednou

### Config management
- Config soubor: `.planning/config.json` — vedle ostatních GSD souborů, verzovatelný v gitu
- **Graceful fallback**: když `.planning/` nebo config.json neexistuje, přátelská zpráva + config se vytvoří automaticky při prvním zápisu
- **Read + Write API**: `/gsd config get <key>` a `/gsd config set <key> <value>` — plný CRUD z chatu
- **Dot-notation hloubka**: 2 úrovně (workflow.auto_advance, progress.completed_phases)

### Claude's Discretion
- Interní architektura GSD dispatch modulu (enum vs match vs trait)
- Přesný formát progress baru (Unicode block chars, délka)
- Frontmatter AST interní reprezentace
- Path helpers API design (slug generation, phase numbering)
- Error message wording pro graceful fallback

</decisions>

<specifics>
## Specific Ideas

- `/gsd state` výstup by měl vypadat takto:
  ```
  ## GSD State
  **Milestone:** v1.2.1-dev
  **Phase:** 20/23 (GSD Core + State Engine)
  **Status:** Not started
  **Last activity:** 2026-03-07
  ### Progress
  [--------] 80%
  Plans: 45/49 | Phases: 19/23
  ### Velocity
  - v1.0.2: 17 plans (5 phases)
  - v1.2.0: 19 plans (6 phases)
  ### Blockers
  - Research: frontmatter parser
  ```
- `/gsd progress` výstup: progress bar + tabulka fází se statusem (Complete/Current/Pending)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `slash.rs`: SlashResult enum (Immediate/Async/Silent/NotACommand), SYSTEM_MSG_MARKER, dispatch pattern, Levenshtein fuzzy match
- `slash.rs`: matching_commands() pro autocomplete — GSD subcommands potřebují analogii
- `slash.rs`: COMMANDS static slice — `/gsd` se přidá jako nová položka
- `logic.rs`: send_query_to_agent() s `/` intercept na řádku 17-19
- `WorkspaceState`: root_path, slash_build_rx, slash_git_rx, slash_conversation_gen — pattern pro async výsledky

### Established Patterns
- mpsc::channel() pro async background results (viz cmd_git, cmd_build)
- SYSTEM_MSG_MARKER prefix pro system message styling
- egui_commonmark pro markdown rendering výstupu
- Synchronní I/O pro malé soubory (settings.toml read/write pattern)
- "zero new dependencies" — custom řešení místo externích crate

### Integration Points
- `slash.rs` dispatch: nový match branch `"gsd" => cmd_gsd(ws, args)` předá do GSD modulu
- `src/app/cli/gsd/mod.rs`: GSD dispatch, subcommand registry
- `src/app/cli/gsd/frontmatter.rs`: YAML-like parser s round-trip
- `src/app/cli/gsd/config.rs`: config.json management
- `src/app/cli/gsd/state.rs`: STATE.md read/write/update
- `src/app/cli/gsd/paths.rs`: path helpers, slug generation, phase numbering

</code_context>

<deferred>
## Deferred Ideas

- **GSD update systém** — kontrola nových verzí GSD z GitHubu, notifikace o dostupných aktualizacích
- **Kompilace nových verzí** — inkrementální vs. plná kompilace GSD aktualizací z editoru
- **About dialog version check** — tlačítko v About dialogu pro kontrolu nové verze GSD a nabídka kompilace
- **Kompilace jako výhoda** — vyhodnotit zda Rust kompilace GSD přímo v editoru přináší výkonnostní benefit vs. overhead

</deferred>

---

*Phase: 20-gsd-core-state-engine*
*Context gathered: 2026-03-07*
