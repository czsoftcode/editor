# Feature Landscape

**Domain:** Slash command system + GSD project management tools for AI-assisted code editor CLI
**Researched:** 2026-03-07
**Milestone:** v1.2.1-dev GSD Integration + Slash Commands

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| `/help` command | First thing any CLI user types; without it, discoverability is zero | Low | Slash dispatch infra | Static output, list all commands with one-liner descriptions |
| `/clear` command | Chat accumulates; users need a reset | Low | Slash dispatch infra, ChatState | Clear conversation vec + streaming_buffer |
| `/new` command | Start fresh conversation without clearing history | Low | Slash dispatch infra, ChatState | Reset conversation, keep prompt history |
| `/model` command | User already has model picker in settings; CLI shortcut expected | Low | Slash dispatch infra, OllamaProvider | List models / switch active model |
| Slash command parsing + dispatch | Core infrastructure everything else depends on | Med | ChatState prompt handling | Must intercept prompt starting with `/` before sending to Ollama |
| `/gsd state` | Users need to see current project state | Med | GSD state module, frontmatter parser | Parse STATE.md frontmatter + markdown sections |
| `/gsd phase` subcommands | Core GSD workflow: add/complete/list phases | High | GSD phase module, roadmap module, filesystem | 5 subcommands: next-decimal, add, insert, remove, complete |
| `/gsd roadmap` subcommands | View/analyze project roadmap | Med | GSD roadmap module, ROADMAP.md parser | get-phase, analyze, update-plan-progress |
| `/gsd commit` | Commit planning docs with standardized messages | Med | Git integration (already exists in workspace) | Must respect existing git patterns in codebase |
| `/gsd progress` | Visual progress bar/report | Low | GSD state module | Parse phase/plan counts, render as markdown in chat |
| YAML frontmatter parsing | GSD files use `---` delimited YAML frontmatter extensively | Med | None (new module) | Custom parser like JS version; do NOT use full serde_yaml -- the GSD frontmatter is a limited YAML subset |
| Markdown output in chat | GSD commands must render results as formatted markdown | Low | Already exists (pulldown-cmark + egui_commonmark) | Leverage existing chat markdown rendering |
| Error handling for missing `.planning/` | Graceful degradation when project has no GSD structure | Low | All GSD commands | Show helpful message, not crash |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Dependencies | Notes |
|---------|-------------------|------------|--------------|-------|
| `/gsd init` workflow commands | One-command context aggregation for AI workflows (execute-phase, plan-phase, new-project, etc.) | High | All GSD modules, AI model integration | The compound `init` commands aggregate state+roadmap+phase context into a single payload -- unique to GSD |
| `/gsd template fill` | Pre-filled PLAN/SUMMARY/VERIFICATION templates with phase-aware data | Med | GSD template module, frontmatter serializer | Reduces boilerplate; JS version generates frontmatter + markdown body |
| `/gsd verify` suite | Automated verification of plan structure, phase completeness, references, artifacts | High | GSD verify module, filesystem scanning | 6 verification subcommands; catches incomplete work |
| `/gsd validate` health/consistency | Integrity checking of `.planning/` directory structure | Med | GSD verify module, roadmap parser | Optionally auto-repair with `--repair` flag |
| `/gsd milestone complete` | Archive completed milestone, create MILESTONES.md entry | Med | GSD milestone module, filesystem operations | Moves phase dirs, updates roadmap |
| `/gsd scaffold` | Create phase directories and template files | Low | GSD template module | context, uat, verification, phase-dir scaffolds |
| `/gsd frontmatter` CRUD | Direct manipulation of YAML frontmatter in planning files | Med | Frontmatter parser/serializer | get/set/merge/validate subcommands |
| AI-integrated GSD commands | GSD init commands feed aggregated context to AI model for generation | High | AI provider, GSD init module | e.g., `/gsd init plan-phase 5` gathers all context, then AI generates the plan |
| `/git` shortcut | Quick git status/diff/log from chat panel | Low | Existing git_status module | Convenience wrapper |
| `/build` shortcut | Trigger cargo build from chat | Low | Existing build_runner module | Convenience wrapper |
| `/settings` shortcut | Open settings dialog from chat | Low | Existing settings modal | Simple AppAction dispatch |
| Tab completion for slash commands | Type `/g` and see `/gsd`, `/git` suggestions | Med | Slash dispatch registry | Enhances discoverability beyond `/help` |
| Command history for slash commands | Up arrow recalls previous slash commands separately from chat | Low | ChatState history | Already have history vec; filter for `/` prefix entries |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full YAML parser (serde_yaml) | GSD frontmatter is a limited YAML subset (simple key-value, arrays, 2-level nesting). Full YAML parser adds a heavy dependency for unused features. The JS version uses a custom ~80 line parser. | Port the custom frontmatter parser from JS. It handles exactly the patterns used in GSD files. |
| Auto-execute GSD commands without user seeing output | Security concern + user confusion. GSD commands modify filesystem (create dirs, write files, commit). | Always show command output in chat. Use existing approval workflow for write operations. |
| GSD web search (Brave API) | Requires API key configuration, network calls from within editor, and the editor already has Ollama as its AI backend -- web search belongs to the Claude Code workflow, not the editor. | Omit `websearch` command entirely. It is a Claude Code agent tool, not an editor feature. |
| Model profile resolution (opus/sonnet/haiku) | These are Claude API model tiers, irrelevant for Ollama-based editor. The GSD model resolution maps agent types to Claude model tiers. | Skip `resolve-model` command. The editor uses Ollama model picker directly. |
| Plugin/hook system for slash commands | Over-engineering. The editor has 10-15 slash commands total. A registry enum is sufficient. | Use a simple match-based dispatch. Add `SlashCommand` enum with `execute()` method. |
| Interactive multi-step wizards | egui immediate mode makes multi-step wizards awkward. Each frame redraws. State management for wizard steps adds complexity. | Use single-command with flags. For complex workflows, output instructions as markdown. |
| Async slash command execution | The codebase uses `ureq + std::thread` pattern (KEY DECISION from v1.2.0). Introducing async runtime contradicts this. | Use `std::thread::spawn` for long-running GSD commands (git operations, filesystem scanning). Background thread + channel pattern already proven in Ollama streaming. |

## Feature Dependencies

```
Slash Command Dispatch (infra)
  |
  +-- /help, /clear, /new, /model, /settings, /build, /git (basic commands)
  |
  +-- YAML Frontmatter Parser (new module)
  |     |
  |     +-- GSD State Module (reads/writes STATE.md)
  |     |     |
  |     |     +-- /gsd state (subcommands)
  |     |     +-- /gsd progress
  |     |
  |     +-- GSD Roadmap Module (reads/writes ROADMAP.md)
  |     |     |
  |     |     +-- /gsd roadmap (subcommands)
  |     |
  |     +-- GSD Phase Module (directory operations + roadmap updates)
  |     |     |
  |     |     +-- /gsd phase (subcommands)
  |     |     +-- GSD Template Module
  |     |     |     |
  |     |     |     +-- /gsd template fill
  |     |     |     +-- /gsd scaffold
  |     |     |
  |     |     +-- GSD Milestone Module
  |     |           |
  |     |           +-- /gsd milestone complete
  |     |
  |     +-- GSD Verify Module
  |     |     |
  |     |     +-- /gsd verify (subcommands)
  |     |     +-- /gsd validate (subcommands)
  |     |
  |     +-- GSD Frontmatter CRUD
  |           |
  |           +-- /gsd frontmatter (subcommands)
  |
  +-- GSD Config Module (reads .planning/config.json)
  |     |
  |     +-- Used by state, phase, roadmap for configuration
  |
  +-- GSD Init Module (compound workflow aggregation)
        |
        +-- /gsd init (subcommands) -- depends on ALL above modules
        +-- AI model integration (feeds aggregated context to Ollama)
```

## MVP Recommendation

Prioritize:

1. **Slash command dispatch infrastructure** -- everything depends on it. Simple `match` on first token after `/`. Returns `SlashCommandResult` enum (Output/Error/ClearChat/SwitchModel/etc.).

2. **Basic slash commands** (`/help`, `/clear`, `/new`, `/model`) -- immediate user value, validates the dispatch infra works.

3. **YAML frontmatter parser** -- required by all GSD modules. Port the JS `extractFrontmatter()` + `reconstructFrontmatter()` + `spliceFrontmatter()` functions. ~150 lines of Rust.

4. **GSD core module** (config loader, slug generator, phase number comparator, path utilities) -- shared infrastructure for all GSD commands.

5. **GSD state commands** (`/gsd state`, `/gsd progress`) -- most frequently used GSD commands, validates the entire pipeline from slash dispatch through frontmatter parsing to markdown output.

6. **GSD phase + roadmap commands** -- the operational core of GSD workflow management.

7. **GSD verify/validate commands** -- quality assurance layer.

8. **GSD template + scaffold commands** -- file generation layer.

9. **GSD milestone commands** -- lifecycle management.

10. **GSD init compound commands** -- the most complex feature, aggregating all modules, with optional AI integration.

Defer:
- **Tab completion**: Nice to have but not blocking. Can be added in a polish pass.
- **AI-integrated init commands**: The init commands work first as context aggregators (output markdown). AI generation on top is a separate enhancement.
- **`/gsd frontmatter` direct CRUD**: Power-user feature. Lower priority than workflow commands.

## Scope Sizing

The JS GSD tools total **5,421 lines** across 11 modules. Rust port will be approximately:

| JS Module | JS Lines | Est. Rust Lines | Priority | Notes |
|-----------|----------|-----------------|----------|-------|
| core.cjs | 492 | ~400 | P0 | Config, path utils, phase utils, git helpers |
| frontmatter.cjs | 299 | ~250 | P0 | Parser, serializer, CRUD commands |
| state.cjs | 721 | ~600 | P1 | STATE.md operations, progression engine |
| commands.cjs | 548 | ~450 | P1 | Slug, timestamp, todos, scaffold, progress |
| phase.cjs | 901 | ~750 | P1 | Phase CRUD, decimal numbering, completion |
| roadmap.cjs | 298 | ~250 | P1 | Roadmap parsing, phase extraction |
| config.cjs | 169 | ~130 | P0 | Config JSON read/write |
| template.cjs | 222 | ~200 | P2 | Template selection and fill |
| verify.cjs | 820 | ~700 | P2 | Plan structure, completeness, references |
| milestone.cjs | 241 | ~200 | P2 | Milestone completion, archiving |
| init.cjs | 710 | ~600 | P3 | Compound workflow init commands |
| **Total** | **5,421** | **~4,530** | | Plus ~200 for slash dispatch infra |

Estimated total new Rust code: **~4,700 lines** (excluding tests).

## Commands Not Ported (With Rationale)

| JS Command | Reason for Exclusion |
|------------|---------------------|
| `websearch` | Brave API integration -- belongs to Claude Code agent, not editor |
| `resolve-model` | Claude model tier resolution -- irrelevant for Ollama backend |
| `state record-session` | Session continuity tracking -- designed for Claude Code's agent spawning, not editor sessions |
| `summary-extract --fields` | Niche extraction tool -- covered by `frontmatter get` |

## Sources

- GSD tools source: `~/.claude/get-shit-done/bin/gsd-tools.cjs` (592 lines, CLI router)
- GSD lib modules: `~/.claude/get-shit-done/bin/lib/*.cjs` (5,421 lines total, 11 modules)
- GSD templates: `~/.claude/get-shit-done/templates/` (27 template files)
- GSD workflows: `~/.claude/get-shit-done/workflows/` (34 workflow definitions)
- Existing PolyCredo CLI: `src/app/cli/` (9 Rust modules -- executor, tools, state, security, etc.)
- Project context: `.planning/PROJECT.md`, `.planning/STATE.md`
