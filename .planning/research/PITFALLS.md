# Domain Pitfalls

**Domain:** Node.js-to-Rust port (GSD tools), YAML-like frontmatter parser, slash command system, file-based state management, git operations in GUI
**Researched:** 2026-03-07
**Overall confidence:** HIGH (based on direct source code analysis of GSD tools 5,421 LOC and PolyCredo Editor 58,187 LOC)

---

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Mirroring JavaScript's Dynamic Object Model in Rust

**What goes wrong:** The GSD frontmatter parser (`extractFrontmatter` in frontmatter.cjs) builds nested objects on the fly -- keys are added dynamically, arrays are converted from objects mid-parse (lines 65-79), and values are untyped (string, array, or nested object). Direct translation to Rust leads to either `HashMap<String, serde_json::Value>` everywhere (losing type safety) or rigid structs that cannot handle the parser's fluid typing.

**Why it happens:** In JavaScript, `current.obj[key] = value` works regardless of whether `value` is a string, array, or object. The parser even has a "promotion" pattern where an empty `{}` is discovered to be an array when a `- item` line appears, and the parent's reference is retroactively swapped (lines 65-79). This pattern has no natural Rust equivalent.

**Consequences:**
- `unwrap()` panics on unexpected value types
- Verbose match/if-let chains that are harder to maintain than the original JavaScript
- The "object-to-array promotion" pattern requires mutating a parent through a child reference, which violates Rust's borrowing rules

**Prevention:**
- Define a `FrontmatterValue` enum: `String(String)`, `Array(Vec<FrontmatterValue>)`, `Map(IndexMap<String, FrontmatterValue>)`, `Number(i64)`, `Bool(bool)`, `Null`
- Implement `From<FrontmatterValue>` for `serde_json::Value` and vice versa for JSON output compatibility
- Handle the "object-to-array promotion" by using indices into a flat arena instead of nested references. When promotion happens, replace the value at the known index rather than chasing parent references
- Write the parser as a two-pass process: first pass identifies structure (which keys are objects vs arrays), second pass fills values. This eliminates the retroactive type-changing pattern entirely

**Detection:** More than 3 `.unwrap()` or `.expect()` calls in a single function operating on parsed frontmatter data means the type model is wrong.

**Phase relevance:** Must be solved in the first implementation phase (frontmatter parser). Everything depends on this data model.

---

### Pitfall 2: Regex-Heavy Markdown Parsing (20+ Patterns in state.cjs)

**What goes wrong:** The GSD state module uses 20+ regex patterns to extract, replace, and match fields in STATE.md. Patterns are constructed dynamically from user input (e.g., `new RegExp(\`\\*\\*${escaped}:\\*\\*\\s*(.+)\`, 'i')`). In Rust, `regex::Regex::new()` is fallible and expensive to compile. The `stateExtractField` function alone is called 15+ times in `buildStateFrontmatter` and `cmdStateSnapshot`.

**Why it happens:** JavaScript's `.match()` returns null (not error) for failed patterns, regex compilation is implicit and cached by the engine. Developers port these patterns 1:1 into Rust without considering:
1. `Regex::new()` returns `Result` and can fail on malformed patterns
2. Recompiling regex per call is expensive -- 15 compilations per snapshot operation
3. The `stateExtractField` function is defined TWICE in state.cjs (line 12 and line 184), with slight differences -- which version to port?

**Consequences:**
- Performance regression: 20+ regex compilations per state operation
- Panics on malformed field names that produce invalid regex
- Duplicated function confusion during porting

**Prevention:**
- Replace regex-based field extraction with a line-by-line parser: STATE.md has a known structure (`**Field:** value` or `Field: value`). Use `str::starts_with("**")` + `str::find(":**")` instead of regex
- For patterns that genuinely need regex (section extraction like `## Decisions`), use `OnceLock<Regex>` or `lazy_static!`
- For dynamic patterns (field name interpolation), use `regex::escape()` -- not the manual `escapeRegex` function from core.cjs
- Consolidate the two `stateExtractField` implementations into one canonical Rust function

**Detection:** `grep -c "Regex::new" src/gsd/` returning more than 5 is a red flag. Most field extraction should be string operations.

**Phase relevance:** Core infrastructure phase. Must be resolved before state operations are built.

---

### Pitfall 3: Slash Command Parser Conflicts with Chat Input

**What goes wrong:** User input starting with `/` could be a slash command (`/gsd state`), a file path (`/home/user/file.txt`), or natural language ("use /dev/null for testing"). The existing `ChatState` has a single `prompt: String` field with a direct path to `send_chat()` via the `AiProvider` trait. There is no routing layer between user input and the AI provider.

**Why it happens:** The existing chat flow is: user types prompt -> prompt sent to AI provider -> response streamed back. Adding slash commands requires intercepting input before it reaches the AI, but the interception must be precise enough to avoid eating legitimate messages.

**Consequences:**
- Users type `/home/user/path` and get "unknown command" errors
- `/gsd` prefix conflicts if user discusses GSD in conversation
- State inconsistency: command partially executes, error leaves chat with `loading = true` stuck

**Prevention:**
- Strict command syntax: `/command` must be at START of input AND followed by a space or end-of-string. `/gsd state` is a command; `/home/user` is not (no registered `home` command)
- Implement `InputRouter` enum: `Command { name: String, args: Vec<String> }` | `ChatMessage { text: String }` with deterministic parsing
- Register commands in a `HashMap<String, Box<dyn SlashCommand>>` -- unknown `/word` falls through to AI
- **Critical:** Command execution must be atomic with respect to chat UI state. Before executing: set `loading = true`, after completion (success or error): set `loading = false` and append output. Never leave UI in intermediate state
- Unknown `/word` should produce a non-blocking toast with "did you mean?" suggestions, then send to AI

**Detection:** If the slash command parser and the AI send path share the same code path without an explicit routing decision, this pitfall is active.

**Phase relevance:** Must be designed in architecture phase, implemented before any specific commands.

---

### Pitfall 4: File-Based State Concurrency in GUI Context

**What goes wrong:** GSD tools read and write `.planning/STATE.md`, `.planning/config.json`, and phase directory files. The PolyCredo editor uses `notify` for file watching. When a GSD command writes STATE.md, the file watcher fires, the UI tries to reload, and if another command is mid-write, the file is truncated or half-written. The JavaScript `writeStateMd` function (state.cjs:679) calls `syncStateFrontmatter` which re-parses and rewrites the ENTIRE file on every single field update.

**Why it happens:** The original GSD tools are CLI processes that run sequentially: read, modify, write, exit. No concurrent access. In a GUI app, multiple systems are alive simultaneously:
1. GSD command executing in a background thread
2. File watcher monitoring `.planning/`
3. UI thread reading state for display
4. User potentially running another command before the first finishes

**Consequences:**
- Torn reads: UI reads STATE.md while a command is writing (partial content, parse failure)
- Watcher storms: writing STATE.md triggers watcher, which triggers reload, which triggers re-render
- Lost updates: two concurrent commands both read STATE.md, modify different fields, second write overwrites first

**Prevention:**
- **Single writer:** All GSD state writes go through a `GsdStateManager` that serializes access via `mpsc::channel`. Commands send `StateAction` messages; the manager applies them sequentially
- **Debounce watcher events** for `.planning/` directory with 200ms+ delay
- **Never read state files in the UI render loop.** Cache state in memory (`GsdStateCache`), update only when the state manager signals a change
- **Atomic writes:** Write to a tempfile in the same directory, then `std::fs::rename()` to the target path. Rename is atomic on same filesystem (POSIX guarantee)
- **Exclude `.planning/` from the editor's file watcher** or add a dedicated handler that understands GSD state files

**Detection:** If `std::fs::read_to_string(".planning/STATE.md")` appears in any function called from the egui `update()` loop, this pitfall is active.

**Phase relevance:** Must be solved in the state management phase, before any commands that write state.

---

### Pitfall 5: Blocking Git Operations Freeze the GUI

**What goes wrong:** The GSD `cmdCommit` function (commands.cjs:216) calls `execGit` which uses `execSync` -- fully blocking. It stages files, runs commit (which may trigger pre-commit hooks), and reads the commit hash. This can take 1-30+ seconds. Direct Rust port using `std::process::Command::output()` blocks the calling thread.

**Why it happens:** The existing codebase already has async git patterns (`fetch_git_branch`, `fetch_git_status` in `background.rs` using `spawn_task` + `mpsc::channel`). But developers porting GSD commands may not discover these patterns and instead write synchronous `Command::new("git")...output()` in the command handler, especially since the GSD JavaScript is synchronous.

**Consequences:**
- Editor freezes for 1-30 seconds during git commit
- Pre-commit hooks that hang make the editor permanently unresponsive
- User cannot cancel the operation
- Contradicts project value: "Editor nesmi zahrivat notebook v klidovem stavu"

**Prevention:**
- **All git operations MUST use the existing `spawn_task` pattern** from `background.rs`
- Return a `mpsc::Receiver<GitResult>` and poll it in the next frame with `try_recv()`
- Add a cancellation token (`Arc<AtomicBool>`) consistent with the existing `cancellation_token` in `AiState`
- Show a spinner/loading indicator in the chat during git operations
- Implement a timeout (30 seconds default) with user-visible "operation timed out" message
- **Never call `Command::new("git")...output()` on the main thread or in a synchronous command handler**
- Create a reusable `GitExecutor` that wraps the async pattern, so each GSD command doesn't reinvent it

**Detection:** `grep "Command::new.*git.*output()" src/gsd/` should return zero matches.

**Phase relevance:** Must be enforced from the first command that touches git. Create the async git wrapper before implementing `cmdCommit`.

---

## Moderate Pitfalls

### Pitfall 6: YAML Frontmatter Encoding Edge Cases

**What goes wrong:** The GSD frontmatter parser assumes clean UTF-8 and uses simple string splitting. Real-world edge cases that silently break:
- BOM (Byte Order Mark) at file start: `\xEF\xBB\xBF---\n` does not match `^---\n`
- CRLF line endings (Windows): `\r\n` vs `\n` -- `split('\n')` leaves trailing `\r` in values
- Values containing colons: `title: My: Complex: Title` -- must split on FIRST `: ` only
- Values containing `---`: `description: Use --- as separator` triggers false frontmatter end
- Quoted values: inconsistent quoting in GSD (`"value"` and `value` both valid)
- Empty files, files with only frontmatter (no body after `---`), files without frontmatter

**Prevention:**
- Strip BOM: `content.strip_prefix('\u{FEFF}').unwrap_or(content)`
- Normalize line endings: `content.replace("\r\n", "\n")` before parsing
- Use `splitn(2, ": ")` for key-value splitting (the JavaScript already handles this correctly)
- The frontmatter end `---` regex is non-greedy (`[\s\S]+?`) which handles embedded `---` -- verify this in Rust's regex crate
- Write a test suite with these 6+ edge cases BEFORE implementing the parser

**Phase relevance:** Frontmatter parser implementation phase. Tests first, then parser.

---

### Pitfall 7: Error Handling Philosophy Mismatch (process.exit vs Result)

**What goes wrong:** GSD JavaScript uses `process.exit(1)` for errors and `process.exit(0)` for success. Every `error()` call in core.cjs terminates the process. In a GUI app, there is no process to exit -- errors must be propagated to the UI as messages, toasts, or chat output. The JavaScript also has many silent `catch {}` blocks (e.g., state.cjs:29, phase.cjs:414) that swallow errors entirely.

**Consequences:**
- If ported literally, `process.exit` equivalent would be `std::process::exit` -- killing the entire editor
- Silent `catch {}` blocks hide bugs that would surface as panics in Rust (e.g., file permission errors, disk full)

**Prevention:**
- Define a `GsdError` enum: `FileNotFound(PathBuf)`, `ParseError(String)`, `GitError(String)`, `ValidationError(String)`, `ConfigError(String)`, `StateNotInitialized`
- All GSD functions return `Result<GsdOutput, GsdError>`
- Map `GsdError` to user-visible messages via the existing Toast system (`AppAction::ShowToast`) or chat output
- **Never use `unwrap()` in GSD code.** Use `?` operator exclusively
- Create a `GsdOutput` enum: `Json(serde_json::Value)`, `Text(String)`, `Markdown(String)`, `Table { headers: Vec<String>, rows: Vec<Vec<String>> }` for typed output
- For JavaScript's silent `catch {}` blocks: explicitly decide per case whether to log warning, return default, or propagate error

**Phase relevance:** First phase -- define error and output types before implementing any commands.

---

### Pitfall 8: `Object.assign` / JSON.parse Patterns Without Direct Rust Equivalents

**What goes wrong:** GSD pervasively uses:
- `Object.assign(fm, mergeData)` for shallow merge (frontmatter.cjs:270)
- `try { JSON.parse(value) } catch { value }` for "parse if JSON, otherwise use as string" (frontmatter.cjs:255)
- Dynamic property access: `fm[field]` where `field` is a runtime string
- `Set` for deduplication (commands.cjs:101): `digest.tech_stack = new Set()`

These patterns appear 30+ times across the codebase.

**Prevention:**
- For `Object.assign`: Implement `merge(&mut self, other: &FrontmatterValue)` on the Map variant that overwrites matching keys
- For "parse if JSON": Create `fn parse_loose(s: &str) -> FrontmatterValue` that tries `serde_json::from_str` first, falls back to string
- For dynamic property access: Use `HashMap<String, FrontmatterValue>` with `.get()` and `.insert()`. This is the natural Rust equivalent
- For `Set`: Use `HashSet<String>` or `IndexSet<String>` (preserves insertion order like JavaScript `Set`)
- **Key insight:** Don't try to make Rust code "look like" the JavaScript. Identify the INTENT (merge, parse loosely, deduplicate) and use idiomatic Rust patterns for each

**Phase relevance:** Throughout implementation. Establish these helper functions in the core module first.

---

### Pitfall 9: Command Registration and Dispatch Complexity

**What goes wrong:** GSD JavaScript has a flat dispatch in `gsd-tools.cjs` (592 lines of if/else on command names with 40+ subcommands). Porting this as a single `match` statement in Rust creates a massive function. Adding a new command requires modifying the central dispatch.

**Prevention:**
- Use a trait-based command registry:
  ```
  trait SlashCommand: Send + Sync {
      fn name(&self) -> &str;
      fn aliases(&self) -> &[&str];
      fn description(&self, i18n: &I18n) -> String;
      fn execute(&self, args: &[&str], ctx: &mut GsdContext) -> Result<GsdOutput, GsdError>;
  }
  ```
- Register commands at startup in a `Vec<Box<dyn SlashCommand>>`
- Group GSD commands under `/gsd` prefix with sub-dispatch (matching the JavaScript's two-level routing: `gsd-tools.cjs` -> module function)
- Each module (state, phase, frontmatter, etc.) provides a `register_commands(registry: &mut CommandRegistry)` function
- This scales cleanly: adding a command = adding a struct + calling `registry.register(Box::new(MyCommand))`

**Phase relevance:** Architecture phase. Must be decided before any commands are implemented.

---

### Pitfall 10: Chat Conversation Model Not Designed for System Output

**What goes wrong:** GSD commands produce structured output (JSON, tables, progress bars). The existing chat conversation is `Vec<(String, String)>` (user_input, ai_response pairs) in `ChatState`. GSD command output is neither user input nor AI response -- it is system/tool output. Trying to force it into the existing model creates rendering artifacts and breaks AI context.

**Why it happens:** The conversation model was designed for two-party dialogue. GSD commands introduce a third party (the system) that speaks in structured formats (tables, JSON, progress bars) rather than natural language.

**Consequences:**
- GSD output either appears as "AI response" (confusing) or is lost entirely
- When conversation is sent to AI provider, GSD output pollutes the context with tables and JSON
- No visual distinction between AI responses and command output

**Prevention:**
- Extend conversation to `Vec<ChatEntry>`:
  ```
  enum ChatEntry {
      UserMessage(String),
      AiResponse { content: String, model: String },
      CommandOutput { command: String, output: GsdOutput, timestamp: SystemTime },
      SystemError { message: String },
  }
  ```
- Render `CommandOutput` with different styling (monospace, different background, collapsible)
- When building AI context: filter out `CommandOutput` entries or include only a one-line summary
- This is a **breaking change** to the chat data model -- must be done before implementing visible commands

**Phase relevance:** Must be addressed in the first phase, as a prerequisite for any command that produces output.

---

### Pitfall 11: `writeStateMd` Round-Trip Complexity

**What goes wrong:** Every STATE.md write in JavaScript goes through `writeStateMd()` -> `syncStateFrontmatter()` which:
1. Strips existing frontmatter
2. Re-parses ALL fields from the markdown body (15+ regex calls)
3. Rebuilds YAML frontmatter
4. Writes the complete file

This happens on every single field update (including `cmdStateAdvancePlan`, `cmdStatePatch`, `cmdStateRecordSession`). In Rust, this is expensive string processing repeated unnecessarily.

**Prevention:**
- Cache the parsed state in a `GsdState` struct. Modify the struct in memory, serialize only when flushing to disk
- Batch multiple field updates into a single write (`cmdStatePatch` already supports multi-field updates -- extend this pattern)
- Consider whether frontmatter sync is needed at all in the embedded version. If the GUI has state in memory, the YAML frontmatter in STATE.md is redundant. Write only the markdown body; compute frontmatter on demand for external tool compatibility
- If frontmatter sync is kept, do it on a timer (every 5 seconds) not on every mutation

**Phase relevance:** State management implementation phase. Decide caching strategy early.

---

## Minor Pitfalls

### Pitfall 12: i18n Key Explosion for GSD Commands

**What goes wrong:** GSD commands add 50+ new i18n keys (command names, descriptions, help text, error messages) across 5 languages. The existing `all_lang_keys_match_english` test will fail repeatedly during development as keys are added incrementally.

**Prevention:**
- Add all i18n keys in English first, then batch-translate
- Use prefix convention: `gsd-cmd-*` for commands, `gsd-err-*` for errors, `gsd-help-*` for help
- Consider whether GSD structured output (tables, progress bars) should be localized -- the underlying data is in English markdown files, localizing it may cause confusion
- Run the i18n parity test only in CI, not on every local build, to avoid blocking development

**Phase relevance:** Each phase that adds commands needs its i18n pass.

---

### Pitfall 13: Path Handling Differences (Node.js path vs Rust PathBuf)

**What goes wrong:** JavaScript `path.join` normalizes separators. Rust `PathBuf::join` does not resolve `..` (appends literally). GSD code uses `path.isAbsolute(filePath) ? filePath : path.join(cwd, filePath)` which in Rust needs explicit handling.

**Prevention:**
- Use `std::fs::canonicalize()` after joining (only for existing paths)
- For new files: `parent.canonicalize()?.join(filename)`
- Implement `toPosixPath` using `Path::components()` and rebuilding with `/`
- `PathBuf::join` with an absolute path replaces the entire path (unlike JavaScript) -- use conditional logic explicitly
- Create a `GsdPaths` helper with `resolve(cwd: &Path, file: &str) -> PathBuf` that encapsulates the pattern

**Phase relevance:** Core utilities phase. Create early.

---

### Pitfall 14: `process.stdout.write` to Chat Output Mapping

**What goes wrong:** GSD JavaScript uses `process.stdout.write(JSON.stringify(result))` as its output mechanism (core.cjs:35-51). In Rust, there is no stdout to write to -- output must be pushed into the chat conversation. If the mapping is not clean, some commands silently produce no visible output.

**Prevention:**
- Each command returns `Result<GsdOutput, GsdError>` -- the slash command dispatch is responsible for rendering output into the chat
- The `raw` vs JSON output mode distinction from JavaScript disappears -- in the GUI, always use structured output
- The "large payload to tmpfile" pattern (core.cjs:43-48) is irrelevant in the embedded version -- output goes to memory, not stdout

**Phase relevance:** Implicit in the GsdOutput design (Pitfall 7).

---

### Pitfall 15: Directory Scanning Performance for Phase Operations

**What goes wrong:** GSD phase operations (`cmdPhasesList`, `cmdProgressRender`, `cmdStateUpdateProgress`) scan the `.planning/phases/` directory, read all files, and extract frontmatter from each. The JavaScript version does this synchronously. In a GUI context, doing this on every `/gsd progress` command could cause noticeable delay on projects with many phases.

**Prevention:**
- Cache directory scan results in `GsdStateCache` with a TTL (5 seconds)
- Invalidate cache when any file in `.planning/phases/` changes (via the file watcher)
- For the initial implementation, synchronous scanning is acceptable -- profile and optimize only if measured latency exceeds 100ms

**Phase relevance:** Phase/roadmap operations implementation. Not blocking, but keep in mind.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Core infrastructure (GsdError, GsdOutput, GsdPaths) | Pitfall 7, 13, 14 | Define error types, output types, and path helpers first. Everything depends on these. |
| Frontmatter parser | Pitfall 1, 6, 8 | Build as state machine, not JavaScript port. Test edge cases first. Two-pass approach eliminates object-to-array promotion. |
| Chat model extension | Pitfall 10 | Extend ChatEntry enum BEFORE implementing any visible commands. Breaking change to conversation model. |
| Slash command dispatch | Pitfall 3, 9 | Trait-based registry with prefix routing. Solve input routing before adding commands. |
| State operations (state.cjs port) | Pitfall 2, 4, 8, 11 | Single-writer pattern, in-memory cache, atomic writes. Replace regex with line-by-line parsing. Heaviest integration work. |
| Git operations (commit, branch) | Pitfall 5 | Async wrapper using existing `spawn_task` pattern. Non-negotiable. Create GitExecutor before any git commands. |
| Phase/Roadmap operations | Pitfall 15, 8 | Directory scanning with caching. Profile before optimizing. |
| i18n pass | Pitfall 12 | Batch after each command group. Use `gsd-*` prefix convention. |

---

## Sources

- GSD tools source: `~/.claude/get-shit-done/bin/lib/*.cjs` (5,421 LOC, 11 modules) -- direct analysis, HIGH confidence
- GSD entry point: `~/.claude/get-shit-done/bin/gsd-tools.cjs` (592 LOC) -- direct analysis, HIGH confidence
- PolyCredo Editor: `src/app/cli/executor.rs` (tool execution patterns), `src/app/cli/state.rs` (AiState/ChatState), `src/app/ui/background.rs` (async git patterns) -- direct analysis, HIGH confidence
- Rust `regex` crate compilation cost: well-documented in crate docs, recommend `OnceLock` for static patterns -- HIGH confidence
- POSIX `rename()` atomicity: established guarantee on same-filesystem operations (Linux, macOS) -- HIGH confidence
- egui immediate-mode rendering model: verified from existing codebase patterns and egui documentation -- HIGH confidence
- Previous milestone pitfalls (v1.2.0): `.planning/research/PITFALLS.md` from 2026-03-06 -- patterns from Pitfall 1 (UI blocking) and Pitfall 5 (repaint storms) remain relevant for GSD git and state operations
