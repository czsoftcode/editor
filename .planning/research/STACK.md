# Technology Stack

**Project:** PolyCredo Editor v1.2.1 -- GSD Integration + Slash Commands
**Researched:** 2026-03-07
**Scope:** NEW dependencies only. Existing stack (eframe/egui, syntect, egui_term, ureq, serde_json, fluent, globset, regex, walkdir) is validated and NOT re-evaluated.

## Recommended Stack Additions

### Zero New Dependencies Required

The critical finding of this research: **no new crate dependencies are needed** for the GSD port. Every GSD capability maps cleanly to existing dependencies or the Rust standard library.

| Capability | Node.js Original | Rust Solution | New Dep? |
|------------|-----------------|---------------|----------|
| YAML-like frontmatter | Hand-rolled parser | Hand-rolled parser (same approach) | NO |
| JSON config CRUD | `fs` + `JSON.parse` | `serde_json` (already in Cargo.toml) | NO |
| Markdown section extraction | Regex-based | `regex` (already in Cargo.toml) | NO |
| Git operations (add, commit, check-ignore) | `child_process.execSync` | `std::process::Command` (already used in background.rs) | NO |
| File system operations | `fs` module | `std::fs` + `walkdir` (already in Cargo.toml) | NO |
| Path manipulation | `path` module | `std::path` | NO |
| Glob pattern matching | N/A | `globset` (already in Cargo.toml) | NO |
| Date/time formatting | `new Date().toISOString()` | `std::time::SystemTime` + manual ISO format | NO |
| Slug generation | Hand-rolled regex | `regex` (already in Cargo.toml) | NO |
| Template interpolation | String replace | `str::replace` chains | NO |
| Slash command dispatch | N/A (new feature) | Trait + enum dispatch | NO |
| AI model integration | N/A (new feature) | Existing `AiProvider` trait + `ureq` | NO |

### Rationale: Why NOT Add Dependencies

#### YAML Parser (serde_yml, yaml-rust2) -- NOT NEEDED

The GSD frontmatter is NOT arbitrary YAML. It's a constrained subset:
- Simple `key: value` pairs
- Inline arrays `[a, b, c]`
- Block arrays with `- item`
- Max 3 levels of nesting
- No anchors, aliases, tags, or multi-document streams

The Node.js code hand-parses this with 84 lines of code (`extractFrontmatter()` in frontmatter.cjs). A Rust port will be ~100-120 lines using string splitting and `regex`. Adding `serde_yml` (successor to deprecated `serde_yaml`) or `yaml-rust2` would:
1. Add unnecessary compile time (~5-8s incremental)
2. Pull in transitive dependencies
3. Be overkill -- we need parse/reconstruct for a fixed format, not arbitrary YAML
4. Violate the project constraint: "Bez externich zavislosti: Nechceme pridavat nove heavy dependencies"

**Confidence:** HIGH -- verified by reading all 299 lines of frontmatter.cjs.

#### git2 Crate -- NOT NEEDED

The GSD git operations are trivial (from `core.cjs::execGit()` and `commands.cjs::cmdCommit()`):
- `git add .planning/` (stage files)
- `git commit -m "message"` (commit)
- `git check-ignore --no-index -- path` (check gitignore)
- `git rev-parse --short HEAD` (get hash)

The project already uses `std::process::Command::new("git")` in `background.rs` lines 701, 762 for git status. The `git2` crate (v0.20.x, wraps libgit2) would add:
1. Native C library dependency (libgit2 1.9+)
2. ~15-20s compile time
3. Complex cross-compilation for packaging targets
4. Overkill for 4 simple shell commands

**Confidence:** HIGH -- verified by reading commands.cjs `cmdCommit()` and core.cjs `execGit()`.

#### chrono Crate -- NOT NEEDED

GSD uses dates in exactly two formats:
- ISO date: `YYYY-MM-DD` (for `today` fields in STATE.md, frontmatter)
- ISO datetime: `YYYY-MM-DDTHH:MM:SS.sssZ` (for timestamps)

A helper function using `std::time::SystemTime` + `UNIX_EPOCH` arithmetic produces both formats in ~20 lines. The project already uses `std::time` throughout (Instant, Duration). No `chrono` anywhere in existing codebase.

**Confidence:** HIGH.

## Integration Points with Existing Stack

### 1. Slash Command System -- Pure Rust, No Dependencies

```
SlashCommand trait
  -> name() -> &str
  -> aliases() -> Vec<&str>
  -> execute(args: &str, ctx: &mut CommandContext) -> CommandResult
  -> help() -> &str
  -> completions(partial: &str) -> Vec<String>

SlashRegistry (HashMap<String, Box<dyn SlashCommand>>)
  -> register(handler)
  -> dispatch(input: &str) -> Option<CommandResult>
```

Integration: Called from the existing CLI chat input handler in `src/app/cli/mod.rs`. When user types `/gsd state`, the registry dispatches before sending to AI provider. Output renders as markdown in the existing chat UI via `egui_commonmark`.

**CommandResult** enum:
- `Output(String)` -- markdown text to display in chat
- `Silent` -- command executed, no visible output
- `Error(String)` -- error message to display
- `AiRequest { system_prompt, user_prompt }` -- delegate to AI provider

### 2. GSD Tools Port -- serde_json + regex + std::fs

All 11 Node.js modules (5,421 LOC total) map to a single `src/app/gsd/` module tree:

| Node.js Module | LOC | Rust Module | Key Dependencies Used |
|----------------|-----|-------------|----------------------|
| core.cjs | 492 | `gsd/core.rs` | `regex`, `std::fs`, `std::process::Command`, `serde_json` |
| config.cjs | 169 | `gsd/config.rs` | `serde_json`, `serde` (derive), `std::fs` |
| frontmatter.cjs | 299 | `gsd/frontmatter.rs` | `regex`, `serde_json::Value` |
| state.cjs | 721 | `gsd/state.rs` | `regex`, `std::fs`, `serde_json` |
| phase.cjs | 901 | `gsd/phase.rs` | `regex`, `std::fs`, `walkdir` |
| roadmap.cjs | 298 | `gsd/roadmap.rs` | `regex`, `std::fs` |
| commands.cjs | 548 | `gsd/commands.rs` | `std::process::Command`, `serde_json`, `walkdir` |
| verify.cjs | 820 | `gsd/verify.rs` | `regex`, `std::fs`, `walkdir` |
| init.cjs | 710 | `gsd/init.rs` | `std::fs`, `serde_json` |
| milestone.cjs | 241 | `gsd/milestone.rs` | `std::fs`, `regex` |
| template.cjs | 222 | `gsd/template.rs` | `regex`, `std::fs` |
| gsd-tools.cjs | 592 | `gsd/mod.rs` (dispatch) | Enum-based command dispatch |

**Expected Rust LOC:** ~4,000-5,000 (Rust is more verbose in error handling but more concise in pattern matching).

### 3. AI Model Integration for GSD -- Existing AiProvider Trait

GSD workflow commands that need AI (e.g., research, roadmap generation) use the existing infrastructure:

- `AiProvider` trait (src/app/cli/provider.rs) -- already supports streaming + tools
- `OllamaProvider` (src/app/cli/ollama.rs) -- already implements NDJSON streaming
- `ureq` HTTP client -- already used for Ollama API calls
- `std::thread` + `std::sync::mpsc` -- existing threading model

The integration pattern:
1. GSD slash command returns `CommandResult::AiRequest { system_prompt, user_prompt }`
2. CLI chat handler builds messages with the system prompt
3. Calls existing `provider.chat_stream()` via `ProviderConfig`
4. Collects streamed response through existing `StreamEvent` channel
5. AI response displayed in chat, optionally parsed and written to `.planning/` files

**Key decision:** No async runtime changes needed. The `ureq` + `std::thread` model (established in v1.2.0, replacing the proposed reqwest approach) works perfectly for GSD. The AI calls are fire-and-forget from the main thread -- the background thread handles blocking HTTP.

**Confidence:** HIGH -- this is exactly the pattern already working in the shipped v1.2.0 AI chat.

### 4. Date/Time Utility -- std::time Only

```rust
use std::time::{SystemTime, UNIX_EPOCH};

pub fn iso_date_today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = secs / 86400;
    // Calculate year/month/day from days since epoch
    // ~15 lines of civil calendar arithmetic
    format!("{:04}-{:02}-{:02}", year, month, day)
}

pub fn iso_datetime_now() -> String {
    // Same + hours/minutes/seconds
    format!("{}T{:02}:{:02}:{:02}Z", iso_date_today(), h, m, s)
}
```

This matches the existing codebase pattern (no chrono anywhere).

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Frontmatter parsing | Hand-rolled | `serde_yml` 0.0.12 | Overkill, heavy dep, GSD format is fixed subset |
| Frontmatter parsing | Hand-rolled | `yaml-rust2` 0.9 | Better than serde_yml but still unnecessary |
| Git operations | `std::process::Command` | `git2` 0.20 | Requires libgit2 native lib, overkill for 4 commands |
| Date formatting | `std::time` manual | `chrono` 0.4 | Heavy for two format strings |
| CLI arg parsing (slash) | `str::split` + match | `clap` 4.x | Slash commands are simple prefix match, not CLI |
| Markdown generation | `format!()` strings | Template engine (tera, handlebars) | Template filling is simple `{var}` replacement |
| Config management | `serde_json` | `toml` (already present) | GSD config is JSON, not TOML -- matching upstream format |
| Slash dispatch | Trait + HashMap | Macro-based registration | Over-engineering for ~15 commands |

## What NOT to Add

| Crate | Reason to Avoid |
|-------|----------------|
| `serde_yaml` / `serde_yml` | Deprecated / overkill for constrained frontmatter format |
| `git2` | Native C dependency, complex cross-compile, overkill |
| `chrono` | Only need 2 date formats, std::time suffices |
| `clap` | Slash commands are simple prefix dispatch, not CLI parsing |
| `reqwest` | Project uses `ureq` (sync) + std::thread -- don't change working model |
| `async-trait` | GSD operations are synchronous; AI uses existing sync threading |
| `handlebars` / `tera` | Template interpolation is simple `str::replace()` |
| `colored` / `termcolor` | Output goes to egui chat UI, not terminal |
| `ollama-rs` | Too specific, hides API behind abstraction the code already handles |

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| No YAML dep needed | HIGH | Read all 299 lines of frontmatter.cjs, format is constrained |
| No git2 needed | HIGH | Read cmdCommit/execGit, only 4 simple git commands |
| No chrono needed | HIGH | Only 2 date formats, verified no chrono in existing codebase |
| Slash dispatch pattern | HIGH | Standard Rust trait dispatch, no external dep |
| AI integration via existing AiProvider | HIGH | Provider trait + OllamaProvider already shipping in v1.2.0 |
| serde_json for config | HIGH | GSD config.json maps directly to serde_json Value |
| Estimated port LOC | MEDIUM | Based on module-by-module analysis, Rust verbosity varies |

## Sources

- [serde_yaml deprecation discussion](https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868) -- confirms serde_yaml is deprecated
- [yaml-rust2 GitHub](https://github.com/Ethiraric/yaml-rust2) -- pure Rust YAML, stable API
- [serde_yml on crates.io](https://crates.io/crates/serde_yml) -- maintained fork of serde_yaml
- [git2 on crates.io](https://crates.io/crates/git2) -- v0.20.x, requires libgit2 1.9+
- GSD source: `~/.claude/get-shit-done/bin/lib/*.cjs` (11 modules, 5,421 LOC) -- primary source for port analysis
- Project `Cargo.toml` -- verified all existing dependencies
- Project source `src/app/cli/` -- verified existing AI provider, tool executor, security infrastructure
- Project source `src/app/ui/background.rs` -- verified existing `std::process::Command` git usage
