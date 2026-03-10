# Codebase Concerns

**Analysis Date:** 2026-03-04

## Tech Debt

**Pervasive use of `.expect()` for Mutex locks**
- Issue: Throughout the codebase, `lock().expect("lock")` is used for Mutex operations. This panics if a lock is poisoned, potentially crashing the entire application mid-operation.
- Files: `src/ipc.rs:326`, `src/app/startup.rs` (multiple), `src/app/registry/plugins/mod.rs`, `src/app/registry/plugins/host/fs.rs` (multiple), `src/app/registry/plugins/host/sys.rs`, `src/app/registry/plugins/security.rs`
- Impact: Lock poisoning from panics in one thread crashes the application; no graceful degradation or recovery mechanism exists.
- Fix approach: Replace with `lock().map_err(|e| {...})` patterns or use `Result` types. Implement recovery handlers for poisoned locks instead of panicking.

**Large monolithic modules with high cyclomatic complexity**
- Issue: Several modules exceed 600-800 lines of code with complex branching logic.
- Files: `src/app/mod.rs` (872 lines), `src/app/ui/editor/render_lsp.rs` (863 lines), `src/ipc.rs` (648 lines), `src/app/registry/plugins/host/fs.rs` (638 lines), `src/app/ui/background.rs` (602 lines), `src/app/ui/workspace/mod.rs` (597 lines)
- Impact: High maintenance burden; difficult to test individual code paths; increased likelihood of bugs during refactoring.
- Fix approach: Break into smaller modules with single responsibilities; extract helper functions; implement trait-based abstractions for related functionality.

**Unsafe blocks in plugin implementations**
- Issue: Ollama and Gemini plugins use `unsafe` extensively for FFI/host function calls without clear safety contracts.
- Files: `src/plugins/ollama/src/lib.rs` (multiple unsafe calls), `src/plugins/gemini/src/lib.rs` (multiple unsafe calls)
- Impact: Potential memory unsafety; FFI violations could corrupt editor state or allow arbitrary code execution.
- Fix approach: Document safety invariants; add assertions to verify preconditions; consider wrapping unsafe blocks in safe abstractions.

**Bidirectional sync between project root and sandbox has no conflict resolution**
- Issue: The sandbox sync mechanism (`src/app/sandbox.rs`) determines which direction to sync based solely on modification time. No handling for simultaneous edits in both locations or user intent.
- Files: `src/app/sandbox.rs:49-126`
- Impact: Data loss if files are edited in both locations before sync; silent overwrites without user confirmation.
- Fix approach: Implement user-facing conflict dialogs; add 3-way merge for text files; store sync metadata to detect true conflicts.

**Plugin blacklist patterns use string matching instead of proper path validation**
- Issue: Blacklist patterns in `src/app/registry/plugins/security.rs` rely on glob pattern compilation and string matching, which can have performance issues or regex DoS vulnerabilities.
- Files: `src/app/registry/plugins/security.rs`, `src/app/registry/plugins/mod.rs:46-64`
- Impact: Malicious plugins could craft filenames to bypass security checks; unexpected pattern matching behavior.
- Fix approach: Use validated path normalization; pre-compile and cache regex patterns; implement whitelist approach instead of blacklist.

## Known Bugs

**Semantic index embedding calculation blocks UI thread**
- Symptoms: UI freezes when semantic index processes large codebases; BERT model loading is synchronous.
- Files: `src/app/ui/workspace/semantic_index.rs:58-92`
- Trigger: Opening a large project triggers semantic indexing; affects responsiveness during first load.
- Workaround: Disable semantic search or open smaller projects first.
- Fix: Move embedding calculation to background thread; implement progressive indexing with cancellation support.

**File watcher event deduplication misses rapid changes**
- Symptoms: Multiple edits to the same file within milliseconds are coalesced; some changes may be lost.
- Files: `src/watcher.rs:39-80`, `src/app/ui/background.rs:48-76`
- Trigger: Rapid file modifications (e.g., from external tools writing multiple times in sequence).
- Workaround: None; users may need to manually reload files.
- Fix: Implement proper event batching with time windows; store last seen state per file.

**IPC socket cleanup on Windows incomplete**
- Symptoms: Windows IPC port file (`ipc.port`) may not be cleaned up on abnormal termination.
- Files: `src/ipc.rs:238-273`, `src/ipc.rs:259-271`
- Trigger: Application crash or forced termination; subsequent launches may fail to bind new port.
- Workaround: Manually delete `~/.config/polycredo-editor/ipc.port` and restart.
- Fix: Implement port reuse for TCP listener; add cleanup in panic hook; use advisory locking.

## Security Considerations

**Plugin execution with full file system access**
- Risk: WASM plugins can read/write any file within sandbox_root via host functions; no fine-grained permission model.
- Files: `src/app/registry/plugins/host/fs.rs`, `src/app/registry/plugins/host/mod.rs`
- Current mitigation: Blacklist/whitelist path patterns; user approval dialog for write operations.
- Recommendations: Implement capability-based security (grant specific file paths or directories); audit plugins before loading; add runtime permission requests per operation; implement sandbox escape detection.

**Mutex poisoning could expose state corruption**
- Risk: If a thread panics while holding a lock, subsequent operations may read/write corrupted state.
- Files: `src/ipc.rs:326`, `src/app/startup.rs`, `src/app/registry/plugins/mod.rs`
- Current mitigation: None; panics will crash the application.
- Recommendations: Implement lock guards that prevent poisoning; use atomic operations for critical state; add backup state recovery.

**Unsafe FFI in plugins without bounds checking**
- Risk: Ollama/Gemini plugins call host functions with user-controlled input; no validation of arguments or return values.
- Files: `src/plugins/ollama/src/lib.rs:159`, `src/plugins/gemini/src/lib.rs:197-207`
- Current mitigation: Some JSON validation in host handlers.
- Recommendations: Implement strict input validation before unsafe calls; use `Result` types for all FFI boundaries; add fuzz testing for plugin inputs.

**IPC message parsing has no length limits**
- Risk: Malicious IPC clients could send arbitrarily large messages causing DoS or memory exhaustion.
- Files: `src/ipc.rs:275-282`, `src/ipc.rs:320-380`
- Current mitigation: Read timeouts; buffer-based line reading.
- Recommendations: Implement maximum message size limits; add rate limiting; validate message format before processing.

## Performance Bottlenecks

**Semantic index requires full file parsing for every modification**
- Problem: When a file changes, the semantic index recalculates embeddings for all snippets in that file.
- Files: `src/app/ui/workspace/semantic_index.rs`, `src/app/ui/background.rs:85-105`
- Cause: BERT model inference runs on CPU; no incremental indexing strategy.
- Improvement path: Implement snippet diffing; cache unchanged snippets; run indexing in background thread pool; use GPU acceleration if available.

**File watcher broadcasts all events to main thread**
- Problem: Every file system change (even in ignored directories) is processed; 500-event buffer can still saturate.
- Files: `src/watcher.rs:69-80`
- Cause: Flat event processing; no prioritization or batching by significance.
- Improvement path: Implement event filtering at watcher level; separate high-priority events (open files) from low-priority (index); use debouncing windows.

**IPC server spawns one thread per connection**
- Problem: With many instances/requests, thread count grows unbounded.
- Files: `src/ipc.rs:15`, `src/ipc.rs:402-460`
- Cause: No thread pool; each connection spawns a new thread.
- Improvement path: Use thread pool with bounded workers (currently `IPC_MAX_WORKERS = 16` but not enforced); implement work queue.

**Syncing sandbox to project walks entire directory tree**
- Problem: `Sandbox::get_sync_plan()` uses `WalkDir` for both directions; hashing every file on each sync cycle.
- Files: `src/app/sandbox.rs:49-126`
- Cause: No incremental sync tracking; no skip for excluded directories at walk time.
- Improvement path: Maintain inode/mtime map of last sync state; implement incremental tree walk; add gitignore parsing to skip excluded dirs early.

## Fragile Areas

**Multi-viewport state management through Arc<Mutex>**
- Files: `src/app/mod.rs:40-80`, `src/app/types.rs`
- Why fragile: State is shared across multiple window contexts; mutations require locking entire `AppShared`; deadlock risk if locks acquired in wrong order.
- Safe modification: Always acquire locks in consistent order; minimize lock scope; use read-write locks for frequently-read state.
- Test coverage: No explicit tests for multi-window synchronization; race conditions would only manifest under load.

**File watcher event ordering assumptions**
- Files: `src/watcher.rs`, `src/app/ui/background.rs:78-180`
- Why fragile: Code assumes file operations complete in order (create → write → close); rapid external modifications can violate this.
- Safe modification: Treat all file events as concurrent; use file locking or version numbers to detect out-of-order updates.
- Test coverage: No tests for high-frequency file modifications or concurrent external edits.

**Plugin host function callbacks to UI**
- Files: `src/app/registry/plugins/host/mod.rs`, `src/app/registry/plugins/mod.rs`
- Why fragile: Plugins can request UI operations (ask_user, announce_completion) that must be marshaled back to main thread.
- Safe modification: Implement message passing channel; add timeouts for UI waits; validate all plugin responses.
- Test coverage: No integration tests for plugin → UI → plugin cycles.

**Sandbox directory structure assumptions**
- Files: `src/app/sandbox.rs:24-33`, `src/app/ui/workspace/state/init.rs`
- Why fragile: Code assumes `.polycredo/sandbox` always exists and is writable; if creation fails, subsequent operations may fail silently.
- Safe modification: Validate sandbox existence at startup; handle creation failures explicitly; add error recovery.
- Test coverage: No tests for permission errors or full-disk scenarios.

## Scaling Limits

**Single event queue for all file system changes**
- Current capacity: 500-event buffer in `FileWatcher::try_recv()`
- Limit: Large projects (>10k files) can exceed buffer during builds or bulk operations.
- Scaling path: Implement tiered event queues (high/medium/low priority); implement event compression (multiple changes to same file → one event); use parallel watchers per directory.

**Plugin manager loads all plugins into memory at startup**
- Current capacity: Limited only by available RAM; no unloading.
- Limit: Large number of plugins (>50) may cause memory bloat; startup time grows linearly.
- Scaling path: Implement lazy loading; add plugin lifecycle management (unload unused); implement plugin memory limits.

**Semantic index embeddings stored in memory**
- Current capacity: BERT model ~500MB; embeddings ~500B per snippet; ~1000 snippets per GB.
- Limit: Projects with >100k files exhaust memory.
- Scaling path: Implement disk-based index with mmap; implement vector quantization; add eviction policy for old snippets.

**IPC recent projects list stored in JSON**
- Current capacity: Unbounded; stored in `~/.config/polycredo-editor/recent.json`.
- Limit: If user opens 10k projects, file becomes large and parsing is slow.
- Scaling path: Cap recent list at reasonable size (100); implement SQLite backend for metadata; implement LRU eviction.

## Dependencies at Risk

**Extism WASM runtime not actively maintained**
- Risk: `extism = "1.5"` locked at old version; no updates; potential security issues in WASM VM.
- Impact: Plugins may have unchecked vulnerabilities; new WASM features unavailable.
- Migration plan: Monitor extism updates; consider wasmtime or wasmer as alternatives; add security audit of plugin runtime.

**Candle ML model loading blocks main thread**
- Risk: `candle_core`, `candle_transformers`, `candle_nn` versions 0.9 may have memory leak or performance regressions.
- Impact: Semantic indexing freezes UI indefinitely on large files.
- Migration plan: Implement background loading; add timeout for model initialization; consider using onnxruntime instead.

**Notify file watcher implementation varies by platform**
- Risk: `notify = "7"` backend selection (inotify/FSEvents/ReadDirectoryChangesW) may have bugs or limitations per platform.
- Impact: File changes missed on some platforms; spurious watcher errors.
- Migration plan: Add comprehensive watcher tests per platform; implement fallback polling mechanism; add watcher health checks.

## Missing Critical Features

**No atomic transaction support for multi-file edits**
- Problem: Plugin writes to multiple files can be interrupted; no rollback on failure.
- Blocks: Complex refactorings; safe plugin operations.
- Fix: Implement write staging area; only commit after all writes succeed; add rollback on partial failure.

**No permission-aware file locking for concurrent edits**
- Problem: Two instances can edit same file without conflict detection.
- Blocks: Multi-instance safety; collaborative editing.
- Fix: Implement file-level locking (fcntl/CreateFileA); add advisory lock checks before writes.

**No incremental compilation support**
- Problem: Build status parsing doesn't distinguish between incremental and full rebuilds.
- Blocks: Optimization of build feedback; incremental semantic indexing.
- Fix: Parse cargo metadata for incremental compilation status; use build profile information.

## Test Coverage Gaps

**No tests for multi-viewport state synchronization**
- What's not tested: Concurrent edits across multiple windows; viewport cleanup on close.
- Files: `src/app/mod.rs`, `src/app/types.rs`
- Risk: Race conditions or deadlocks in multi-window scenarios would only manifest under load.
- Priority: High

**No integration tests for plugin execution**
- What's not tested: Plugin loading, execution, host function calls, error handling.
- Files: `src/app/registry/plugins/mod.rs`, `src/app/registry/plugins/host/*`
- Risk: Security vulnerabilities or stability issues in plugin sandbox would go undetected.
- Priority: High

**No tests for file system watcher edge cases**
- What's not tested: Rapid file modifications; symlink handling; directory deletion; permission errors.
- Files: `src/watcher.rs`, `src/app/ui/background.rs`
- Risk: Silent data loss or inconsistency in edge cases.
- Priority: Medium

**No tests for IPC message parsing and error handling**
- What's not tested: Malformed messages; oversized payloads; connection drops; concurrent requests.
- Files: `src/ipc.rs`
- Risk: Crashes or data corruption from bad IPC messages.
- Priority: Medium

**No tests for sandbox sync conflict detection**
- What's not tested: Simultaneous edits in project and sandbox; sync ordering; recovery from sync failure.
- Files: `src/app/sandbox.rs`
- Risk: Data loss if sync fails midway or encounters conflicts.
- Priority: High

---

*Concerns audit: 2026-03-04*
