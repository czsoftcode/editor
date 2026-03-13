---
phase: quick-5
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app/ai/ollama.rs
  - src/app/ai/provider.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/background.rs
autonomous: true
requirements: [QUICK-5]
must_haves:
  truths:
    - "validate_ollama_url accepts https://ollama.example.com without explicit port"
    - "validate_ollama_url still accepts http://localhost:11434 with explicit port"
    - "validate_ollama_url still rejects empty strings, garbage, non-http schemes"
    - "OllamaProvider sends Authorization Bearer header when api_key is set"
    - "spawn_ollama_check sends Authorization Bearer header when api_key is provided"
    - "api_key is read from plugin settings and passed through workspace state"
  artifacts:
    - path: "src/app/ai/ollama.rs"
      provides: "Relaxed URL validation, Bearer auth in all ureq calls"
    - path: "src/app/ai/provider.rs"
      provides: "api_key field on ProviderConfig"
    - path: "src/app/ui/workspace/state/mod.rs"
      provides: "ollama_api_key field on WorkspaceState"
    - path: "src/app/ui/workspace/state/init.rs"
      provides: "api_key initialization from plugin settings"
  key_links:
    - from: "src/app/ui/workspace/state/init.rs"
      to: "settings.plugins[ollama].config[API_KEY]"
      via: "HashMap lookup"
      pattern: 'config\.get\("API_KEY"\)'
    - from: "src/app/ui/background.rs"
      to: "spawn_ollama_check"
      via: "passes api_key from workspace state"
      pattern: "spawn_ollama_check.*api_key"
---

<objective>
Revert the port-only restriction in validate_ollama_url so cloud Ollama endpoints (https://ollama.example.com without explicit port) are accepted. Add optional Bearer API key authentication to OllamaProvider for cloud/authenticated Ollama servers.

Purpose: Enable users to connect to cloud-hosted Ollama instances that use standard HTTPS ports and require API key authentication.
Output: Updated ollama.rs, provider.rs, workspace state, and background polling with Bearer auth support.
</objective>

<execution_context>
@/home/stkremen/.claude/get-shit-done/workflows/execute-plan.md
@/home/stkremen/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/app/ai/ollama.rs
@src/app/ai/provider.rs
@src/app/ui/workspace/state/mod.rs
@src/app/ui/workspace/state/init.rs
@src/app/ui/background.rs
@src/config.rs

<interfaces>
From src/app/ai/provider.rs:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub num_ctx: u64,
}
```

From src/settings.rs:
```rust
pub struct PluginSettings {
    pub enabled: bool,
    pub expertise: AiExpertiseRole,
    pub reasoning_depth: AiReasoningDepth,
    pub config: HashMap<String, String>,  // API_KEY lives here
}
```

From src/app/ai/ollama.rs:
```rust
pub fn spawn_ollama_check(base_url: String) -> mpsc::Receiver<OllamaStatus>
pub fn validate_ollama_url(url: &str) -> Option<String>
pub struct OllamaProvider { config: ProviderConfig, agent: ureq::Agent }
impl OllamaProvider { pub fn new(base_url: String, model: String) -> Self }
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Relax URL validation and add api_key to ProviderConfig</name>
  <files>src/app/ai/ollama.rs, src/app/ai/provider.rs</files>
  <action>
1. In `src/app/ai/provider.rs` — add `pub api_key: Option<String>` field to `ProviderConfig` (with `#[serde(default)]` attribute). Update the derive to keep Serialize/Deserialize working.

2. In `src/app/ai/ollama.rs` — `validate_ollama_url`:
   - REMOVE the port requirement block (lines 271-274: `if parsed.port().is_none() { return None; }`).
   - Adjust the URL reconstruction: when port is present use `scheme://host:port`, when port is absent use `scheme://host` (standard https port).
   - Keep all other validation: scheme must be http/https, host must exist, trailing slash stripped.

3. In `src/app/ai/ollama.rs` — `OllamaProvider::new`:
   - Add `api_key: Option<String>` parameter.
   - Store it in `config.api_key`.

4. In `src/app/ai/ollama.rs` — Update all ureq request calls to attach `Authorization: Bearer {key}` header when `api_key` is `Some`:
   - `is_available` — `self.agent.get(&url)` add `.set("Authorization", &format!("Bearer {key}"))` if key exists.
   - `available_models` — same pattern on the GET request.
   - `send_chat` — on the POST request.
   - `stream_chat` — on the POST request (note: config is moved into thread, api_key comes from config.api_key).

5. In `src/app/ai/ollama.rs` — `spawn_ollama_check`:
   - Change signature to `pub fn spawn_ollama_check(base_url: String, api_key: Option<String>) -> mpsc::Receiver<OllamaStatus>`.
   - Inside the thread, attach Bearer header on the GET `/api/tags` call when api_key is Some.

6. Update tests:
   - Fix `validate_ollama_url_rejects_web_url` test — it should now ACCEPT `https://ollama.com` (returns `Some("https://ollama.com")`). Rename to `validate_ollama_url_accepts_https_no_port`.
   - Add test `validate_ollama_url_https_with_path` for `https://my-proxy.com/ollama/` -> `Some("https://my-proxy.com/ollama")`.
   - Add test `validate_ollama_url_http_no_port` for `http://ollama.local` -> `Some("http://ollama.local")`.
   - Fix `OllamaProvider::new` calls in tests to pass `None` for api_key.
   - All existing passing tests must continue to pass.
  </action>
  <verify>
    <automated>cd /home/stkremen/MyProject/Rust/polycredo_editor && cargo test --lib -- ai::ollama 2>&1 | tail -20</automated>
  </verify>
  <done>validate_ollama_url accepts URLs without explicit port. ProviderConfig has api_key field. OllamaProvider and spawn_ollama_check support Bearer auth. All ollama tests pass.</done>
</task>

<task type="auto">
  <name>Task 2: Wire api_key through workspace state and background polling</name>
  <files>src/app/ui/workspace/state/mod.rs, src/app/ui/workspace/state/init.rs, src/app/ui/background.rs</files>
  <action>
1. In `src/app/ui/workspace/state/mod.rs` — add `pub ollama_api_key: Option<String>` field to `WorkspaceState` struct (next to `ollama_base_url`).

2. In `src/app/ui/workspace/state/init.rs` — in `init_workspace` function:
   - Read API_KEY from plugin settings: `settings.plugins.get("ollama").and_then(|p| p.config.get("API_KEY")).cloned()`.
   - Store in the new `ollama_api_key` field of the returned WorkspaceState.
   - Also pass the api_key to `ProviderConfig` where `OllamaProvider::new` is called, if it's called here (it may be called elsewhere — grep first). If OllamaProvider is not constructed in init.rs, just store the field.

3. In `src/app/ui/background.rs` — in the Ollama polling section (line ~201):
   - Change `spawn_ollama_check(ws.ollama_base_url.clone())` to `spawn_ollama_check(ws.ollama_base_url.clone(), ws.ollama_api_key.clone())`.

4. Grep for any other call sites of `spawn_ollama_check` or `OllamaProvider::new` and update them to pass the api_key parameter. Common locations: any place that creates OllamaProvider for chat sending.
  </action>
  <verify>
    <automated>cd /home/stkremen/MyProject/Rust/polycredo_editor && cargo check 2>&1 | tail -20</automated>
  </verify>
  <done>api_key is read from settings.plugins["ollama"].config["API_KEY"], stored in WorkspaceState, and passed to spawn_ollama_check and OllamaProvider. Full project compiles without errors.</done>
</task>

</tasks>

<verification>
1. `cargo test --lib -- ai::ollama` — all ollama tests pass including new URL validation tests.
2. `cargo check` — full project compiles.
3. Manual: set `API_URL = "https://some-cloud.example.com"` and `API_KEY = "sk-test"` in ollama plugin config — verify app accepts the URL and sends Bearer header (observable in logs or network).
</verification>

<success_criteria>
- validate_ollama_url("https://ollama.example.com") returns Some("https://ollama.example.com")
- validate_ollama_url("http://localhost:11434") still returns Some (backward compat)
- validate_ollama_url("") and validate_ollama_url("not-a-url") still return None
- ProviderConfig.api_key field exists and is Optional
- All ureq calls conditionally send Authorization: Bearer header
- spawn_ollama_check accepts optional api_key
- WorkspaceState.ollama_api_key populated from plugin settings
- cargo check passes, cargo test passes
</success_criteria>

<output>
After completion, create `.planning/quick/5-revert-validate-ollama-url-port-restrict/5-SUMMARY.md`
</output>
