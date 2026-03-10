# Testing Patterns

**Analysis Date:** 2026-03-04

## Test Framework

**Runner:**
- `cargo test` (Rust's built-in test framework)
- No separate test runner (e.g., pytest, Jest) needed
- Config: Standard Rust inline tests (no separate Cargo.toml test config)

**Assertion Library:**
- Standard Rust assertions: `assert!()`, `assert_eq!()`, `assert!((a - b).abs() < epsilon)`
- No external assertion library (e.g., proptest, quickcheck) detected

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test --lib       # Run library tests only
cargo test -- --nocapture  # Show output during tests
cargo test -- --test-threads=1  # Single-threaded execution
cargo test <pattern>   # Run tests matching pattern
```

## Test File Organization

**Location:**
- Inline tests in source files — not separate test directory
- Test modules declared with `#[cfg(test)]` and `mod tests { ... }`
- 5 test modules detected in codebase:
  - `src/app/registry/plugins/mod.rs`
  - `src/app/ui/editor/render_lsp.rs`
  - `src/app/ui/editor/render/mod.rs`
  - `src/app/build_runner.rs`
  - `src/i18n.rs`

**Naming:**
- Test functions prefixed with `test_`: `fn test_hello_plugin()`, `fn test_parse_json_compiler_message_error()`
- Test modules named `tests`: `mod tests { ... }`
- Multiple assertion tests in single module: 3+ assertions per test module average

**Structure:**
```
src/module/mod.rs      # Main code
  #[cfg(test)]
  mod tests {          # Test module at end of file
    #[test]
    fn test_something() { ... }
  }
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;  // Import all parent scope items

    #[test]
    fn test_name() {
        // Setup (if needed)
        let input = ...;

        // Execute
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
```

**Patterns from codebase:**

*Assertion with floating-point tolerance (from `src/app/ui/editor/render/mod.rs`):**
```rust
#[test]
fn goto_scroll_centers_when_possible() {
    let offset = goto_centered_scroll_offset(50, 200, 20.0, 200.0);
    assert!((offset - 890.0).abs() < f32::EPSILON);
}
```

*Simple setup-execute-assert (from `src/app/registry/plugins/mod.rs`).*
```rust
#[test]
fn test_hello_plugin() {
    let manager = PluginManager::new(PathBuf::from("/tmp"));
    let plugins_dir = crate::ipc::plugins_dir();
    // ... setup continues ...
}
```

*JSON parsing test (from `src/app/build_runner.rs`).*
```rust
#[test]
fn parse_json_compiler_message_error() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error",...}}"#;
    // Parse input through function
    let errors = parse_build_messages_json(input);
    // Assert on result
    assert!(!errors.is_empty());
}
```

**Setup and Teardown:**
- No explicit setup/teardown functions (no `#[before_each]` style)
- Setup done inline in test function
- No shared fixtures detected
- PathBuf creation for temp paths: `PathBuf::from("/tmp")`

## Mocking

**Framework:** None explicitly used

**Patterns:**
- Direct function calls on deterministic inputs
- No mock/stub objects detected
- No Mockito or other mock library in dependencies
- Plugin tests use real filesystem (`crate::ipc::plugins_dir()`)
- Test data passed as literal values:
  ```rust
  let input = r#"{"reason":"compiler-message","message":{"level":"error",...}}"#;
  let errors = parse_build_messages_json(input);
  ```

**What to Mock:**
- Filesystem operations would benefit from mocking (but not currently done)
- Network calls (if added) should be mocked
- Plugin execution could use mock WASM modules

**What NOT to Mock:**
- Pure functions (like `parse_build_messages_json()`)
- Simple data structures
- Utility functions (math calculations)
- String/JSON parsing

## Fixtures and Factories

**Test Data:**
- Hardcoded literals in test functions:
  ```rust
  let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `x` in this scope","spans":[{"file_name":"src/main.rs","line_start":3,"column_start":5,"is_primary":true}]}}"#;
  ```
- No factory pattern or builder for test objects
- Simple initialization: `let manager = PluginManager::new(PathBuf::from("/tmp"));`

**Location:**
- Test data defined inline in test functions
- No separate fixtures directory or file
- Constants for repeated values not used across multiple tests

## Coverage

**Requirements:** Not enforced

**View Coverage:**
- Run with: `cargo tarpaulin` (if installed)
- Or: `cargo llvm-cov` (requires cargo-llvm-cov tool)
- No CI/CD coverage checks configured
- Estimated coverage: Low (~5% — only 5 test modules for 229 files)

## Test Types

**Unit Tests:**
- Scope: Pure functions and parsing logic
- Approach: Input-output validation
- Example: `test_goto_scroll_centers_when_possible()` tests scroll offset calculation
- Example: `test_parse_json_compiler_message_error()` tests JSON parsing
- Example: `test_hello_plugin()` tests plugin system initialization

**Integration Tests:**
- Scope: None explicitly separated
- Could be added via `tests/` directory at project root
- Currently untested: LSP client interactions, file system I/O, terminal rendering

**E2E Tests:**
- Framework: None
- Status: Not implemented
- Would require UI automation for egui interactions (difficult/not standard)

## Common Patterns

**Assertion Patterns:**

*Equality:*
```rust
assert_eq!(result.unwrap(), expected);
```

*Boolean conditions:*
```rust
assert!(!errors.is_empty());
assert!(result.is_ok());
```

*Range/approximate:*
```rust
assert!((offset - 890.0).abs() < f32::EPSILON);
```

**Async Testing:**
- Not used (Rust std test framework does not support async by default)
- Would need tokio test harness if added: `#[tokio::test]`
- Currently no async tests detected

**Error Testing:**
```rust
#[test]
fn parse_json_ignores_non_compiler_messages() {
    let input = r#"{"reason":"build-finished","success":false}"#;
    // This should parse without error, but produce empty results
    let errors = parse_build_messages_json(input);
    assert!(errors.is_empty());
}
```

## Languages Detection Test

**From `src/i18n.rs`:**
```rust
#[test]
fn supported_langs_load_without_panic() {
    for lang in SUPPORTED_LANGS {
        let bundle = build_bundle(lang);
        // Implicitly passes if no panic
    }
}
```

## Test Execution

**Inline vs. Separate:**
- All tests are inline (`#[cfg(test)] mod tests { ... }`)
- Tests run with main binary: `cargo test --lib`
- No separate test binary crate

**Visibility:**
- Test modules use `#[cfg(test)]` to compile only during testing
- Test functions must be public: `pub fn test_name()` or just `fn test_name()` (implicit pub in test module)
- Test module imports parent scope with `use super::*;`

## Gaps and Recommendations

**Untested Code:**
- UI rendering logic (egui interactions)
- File system operations (save, load, watch)
- Terminal emulation
- IPC communication
- Plugin system (partial coverage)
- Async I/O (tokio tasks)

**Potential Additions:**
1. Integration tests for file operations in `tests/` directory
2. Mock filesystem for plugin loading tests
3. Snapshot testing for parsed error output
4. Property-based testing for parsing logic (proptest crate)
5. Async test support for tokio operations

---

*Testing analysis: 2026-03-04*
