---
phase: quick-5
plan: 01
subsystem: ai-provider
tags: [ollama, url-validation, bearer-auth, cloud-provider]
dependency_graph:
  requires: []
  provides: [relaxed-url-validation, bearer-auth-ollama]
  affects: [ollama-provider, workspace-state, background-polling]
tech_stack:
  added: []
  patterns: [conditional-bearer-auth, optional-port-url-validation]
key_files:
  created: []
  modified:
    - src/app/ai/ollama.rs
    - src/app/ai/provider.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/ui/background.rs
decisions:
  - Removed port-only restriction to support cloud Ollama endpoints on standard HTTPS ports
  - Bearer auth applied conditionally via helper method on OllamaProvider, inline closures in spawn/stream threads
metrics:
  duration: 2min
  completed: "2026-03-06T09:31:34Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Quick Task 5: Revert validate_ollama_url Port Restriction + Bearer Auth

Relaxed URL validation to accept cloud Ollama endpoints without explicit port; added optional Bearer API key authentication to all OllamaProvider HTTP calls.

## What Changed

### Task 1: Relax URL validation and add api_key to ProviderConfig (fa6284d)

- **ProviderConfig** (`provider.rs`): Added `api_key: Option<String>` with `#[serde(default)]`
- **validate_ollama_url** (`ollama.rs`): Removed `parsed.port().is_none()` rejection; URL reconstruction now conditionally includes port
- **OllamaProvider::new**: Added `api_key: Option<String>` parameter, stored in config
- **Bearer auth**: Added `apply_auth` helper method; all 4 ureq call sites (`is_available`, `available_models`, `send_chat`, `stream_chat`) and `spawn_ollama_check` now conditionally attach `Authorization: Bearer` header
- **Tests**: Renamed `rejects_web_url` to `accepts_https_no_port`; added `https_with_path`, `http_no_port`, `rejects_ftp_scheme`, `api_key_stored` tests (22 tests total, all pass)

### Task 2: Wire api_key through workspace state and background polling (3cc63a0)

- **WorkspaceState** (`state/mod.rs`): Added `ollama_api_key: Option<String>` field
- **init_workspace** (`state/init.rs`): Reads `API_KEY` from `settings.plugins["ollama"].config`
- **background.rs**: Passes `ws.ollama_api_key.clone()` to `spawn_ollama_check`

## Verification

- `cargo test -- ai::ollama`: 22 passed, 0 failed
- `cargo check`: compiles without errors (2 pre-existing warnings)

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED
