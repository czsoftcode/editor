# Phase 18: Phase 16 Verification & i18n Fixes - Research

**Researched:** 2026-03-06
**Domain:** Documentation gap closure, i18n bug fixes, checkpoint updates
**Confidence:** HIGH

## Summary

Phase 18 is a cleanup/verification phase that closes gaps from the v1.2.0 milestone audit. No new features are built -- this phase creates missing documentation artifacts (16-04-SUMMARY.md, 16-VERIFICATION.md), fixes an i18n bug where `cli-tool-ask-heading` is called without `$agent` parameter, removes an orphaned i18n key, localizes remaining hardcoded English strings, and updates stale checkboxes in ROADMAP.md and REQUIREMENTS.md.

All 7 success criteria are well-defined, concrete, and verifiable with grep/file-existence checks. The codebase is in good shape -- Phase 16 work is complete (8 commits for plan 04), but the summary and verification documents were never written. The i18n issues are minor: one missing parameter, one orphaned key, and ~10 hardcoded English strings in render.rs/background.rs/logic.rs.

**Primary recommendation:** Structure as 2-3 small plans: (1) documentation artifacts + checkbox updates, (2) i18n fixes. All tasks are straightforward file edits with automated verification.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TOOL-01 | Automaticky editor kontext | Already implemented (16-02); needs verification doc to confirm SATISFIED |
| TOOL-02 | File read tool with approval | Already implemented (16-03); needs verification doc |
| TOOL-03 | File write/replace tool with approval | Already implemented (16-03); needs verification doc |
| TOOL-04 | Command execution tool with approval | Already implemented (16-03); needs verification doc |
| TOOL-05 | Approval UI -- Approve/Deny/Always workflow | Already implemented (16-04); needs verification doc + REQUIREMENTS.md checkbox |
| TOOL-06 | Ask-user tool | Already implemented (16-03/16-04); needs verification doc + i18n fix for `$agent` param |
| CLEN-03 | i18n aktualizace | Partially done (17-01/17-03); orphaned key + hardcoded strings remain |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| fluent / fluent-bundle | existing | i18n localization | Already used throughout project |
| cargo test | built-in | Verification | `all_lang_keys_match_english` test validates i18n parity |

No new dependencies needed. This phase only modifies existing files.

## Architecture Patterns

### i18n Pattern in This Project
```rust
// Simple key (no parameters):
i18n.get("cli-tool-approve")

// Key with parameters (FluentArgs):
let mut args = FluentArgs::new();
args.set("agent", "AI");
i18n.get_args("cli-tool-ask-heading", &args)
```

**Fluent FTL syntax for parameters:**
```ftl
cli-tool-ask-heading = Agent '{ $agent }' is asking:
```

When a key uses `{ $variable }` syntax in .ftl files, it MUST be called via `i18n.get_args()` with a `FluentArgs` containing that variable. Using `i18n.get()` will silently omit the variable, producing broken output like "Agent '{$agent}' is asking:".

### Verification Document Pattern
Previous phases used VALIDATION.md (test strategy). Phase 18 SC2 requires a 16-VERIFICATION.md that confirms each TOOL-* requirement is SATISFIED. This is a different artifact -- a post-hoc verification report, not a test plan.

### SUMMARY.md Pattern
Each completed plan gets a SUMMARY.md documenting commits, changes made, and completion status. The 16-04 plan had 8 commits (7a8e5e1..e48601f) but no SUMMARY.md was written.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| i18n key parity check | Manual checking across 5 locales | `cargo test all_lang_keys` | Existing test catches missing/extra keys |
| Finding hardcoded strings | Manual grep | Grep for `"[A-Z][a-z]` patterns in target files | Systematic, repeatable |

## Common Pitfalls

### Pitfall 1: Fluent Args Type Mismatch
**What goes wrong:** `FluentArgs::set()` accepts `Into<FluentValue>`. Passing wrong type silently produces `{$var}` literal in output.
**How to avoid:** Always use `FluentValue::from("string")` or pass `&str` directly. Test by switching language and verifying output.

### Pitfall 2: Orphaned Key Removal Breaking Tests
**What goes wrong:** Removing a key from one locale but not all 5 causes `all_lang_keys_match_english` test to fail.
**How to avoid:** Always remove/add keys in ALL 5 locales (cs, en, de, ru, sk) simultaneously. Run `cargo test all_lang_keys` after every i18n change.

### Pitfall 3: Checkbox Formatting in Markdown
**What goes wrong:** Using `[X]` instead of `[x]` or adding extra spaces breaks GSD tooling parsing.
**How to avoid:** Always use lowercase `[x]` with no extra spaces: `- [x]`.

### Pitfall 4: Forgetting Traceability Table
**What goes wrong:** Updating the requirement checkbox but not the Traceability table status at the bottom of REQUIREMENTS.md.
**How to avoid:** Always update BOTH the checkbox AND the Traceability table status (change "Pending" to "Complete").

## Current Gap Analysis

### SC1: 16-04-SUMMARY.md
- **Status:** MISSING
- **Action:** Create summary documenting 8 commits (7a8e5e1..e48601f)
- **Commits:** test(16-04), feat(16-04) x5, fix(16-04) x2

### SC2: 16-VERIFICATION.md
- **Status:** MISSING (16-VALIDATION.md exists but is different -- it's a test strategy, not verification report)
- **Action:** Create verification report confirming TOOL-01 through TOOL-06 are SATISFIED
- **Evidence needed:** For each requirement, cite the implementing files/commits and verification method

### SC3: cli-tool-ask-heading with $agent parameter
- **Status:** BUG -- `approval.rs:134` uses `i18n.get("cli-tool-ask-heading")` but the key expects `{ $agent }` parameter
- **Action:** Change to `i18n.get_args("cli-tool-ask-heading", &args)` where args contains "agent" value
- **File:** `src/app/ui/terminal/ai_chat/approval.rs` line 134

### SC4: Hardcoded strings in background.rs, logic.rs, render.rs
- **Status:** Multiple hardcoded English strings found:
  - `render.rs:47` -- "Family: {}" (tooltip label)
  - `render.rs:50` -- "Parameters: {}" (tooltip label)
  - `render.rs:53` -- "Quantization: {}" (tooltip label)
  - `render.rs:56` -- "Context: {ctx}" (tooltip label)
  - `render.rs:112` -- "In: {} | Out: {}" (token counter)
  - `render.rs:495` -- "Generating..." (loading label)
  - `render.rs:509` -- "In: {} | Out: {}" (token counter, duplicate)
  - `background.rs:372,493` -- "Unexpected result" (error fallback)
  - `logic.rs:18` -- "Ollama is not connected." (toast error)
- **Action:** Create i18n keys for each, add translations to all 5 locales, replace hardcoded strings

### SC5: Orphaned cli-tool-approval-heading key
- **Status:** Key exists in all 5 locale cli.ftl files but is NOT referenced anywhere in source code
- **Note:** `cli-tool-tool-approval-heading` (line 46 of en/cli.ftl) is a separate key that IS used or may be needed. Only remove `cli-tool-approval-heading` (line 36).
- **Action:** Remove `cli-tool-approval-heading` from all 5 locale files

### SC6: ROADMAP plan checkboxes
- **Status:** 10 plan checkboxes are `[ ]` but should be `[x]`:
  - `13-01-PLAN.md`, `13-02-PLAN.md`, `13-03-PLAN.md`
  - `15-00-PLAN.md`, `15-02-PLAN.md`, `15-03-PLAN.md`
  - `16-01-PLAN.md`, `16-02-PLAN.md`, `16-03-PLAN.md`, `16-04-PLAN.md`
- **Action:** Change `[ ]` to `[x]` for each, add completion date

### SC7: REQUIREMENTS.md checkboxes
- **Status:** 4 requirement checkboxes are `[ ]` but should be `[x]`:
  - `PROV-01`, `PROV-02`, `CHAT-01`, `CHAT-06`
  - Also: `TOOL-05` is `[ ]` but should be `[x]` (implemented in 16-04)
- **Action:** Change `[ ]` to `[x]`, update Traceability table status to "Complete"

## Code Examples

### Fixing cli-tool-ask-heading call (SC3)
```rust
// BEFORE (broken -- $agent not passed):
egui::RichText::new(i18n.get("cli-tool-ask-heading"))

// AFTER (correct):
let mut args = fluent::FluentArgs::new();
args.set("agent", "AI");  // or whatever agent name is appropriate
egui::RichText::new(i18n.get_args("cli-tool-ask-heading", &args))
```

### Adding i18n key for hardcoded string (SC4)
```ftl
# In locales/en/cli.ftl:
cli-chat-generating = Generating...
cli-chat-model-family = Family: { $value }
cli-chat-model-params = Parameters: { $value }
cli-chat-model-quant = Quantization: { $value }
cli-chat-model-context = Context: { $value }
cli-chat-token-counter = In: { $input } | Out: { $output }
cli-chat-unexpected-result = Unexpected result
cli-chat-ollama-disconnected = Ollama is not connected.
```

### 16-VERIFICATION.md Structure
```markdown
# Phase 16: Tool Execution - Verification

| Requirement | Status | Evidence |
|-------------|--------|----------|
| TOOL-01 | SATISFIED | build_system_message() in logic.rs, AiContextPayload in types.rs |
| TOOL-02 | SATISFIED | read_file handler in executor.rs, approval flow in background.rs |
| TOOL-03 | SATISFIED | write_file/replace handlers in executor.rs, diff preview in approval.rs |
| TOOL-04 | SATISFIED | exec handler in executor.rs with timeout, approval in background.rs |
| TOOL-05 | SATISFIED | render_tool_approval_ui in approval.rs, Approve/Deny/Always buttons |
| TOOL-06 | SATISFIED | ask_user handler in executor.rs, render_tool_ask_ui in approval.rs |
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TOOL-01..06 | Verification doc confirms SATISFIED | doc review | `test -f .planning/phases/16-tool-execution/16-VERIFICATION.md` | Wave 0 |
| CLEN-03 (i18n) | All keys in all locales | unit | `cargo test all_lang_keys -x` | Exists |
| CLEN-03 (orphan) | No orphaned keys in source | grep | `grep -r cli-tool-approval-heading src/` should return nothing | N/A |
| CLEN-03 (args) | $agent parameter passed | grep | `grep get_args.*cli-tool-ask-heading src/` should match | N/A |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before verify

### Wave 0 Gaps
None -- existing test infrastructure covers all phase requirements. The `all_lang_keys_match_english` test validates i18n key parity across all 5 locales.

## Open Questions

1. **What agent name for $agent parameter?**
   - What we know: The FTL key uses `{ $agent }`, the approval.rs context likely has access to model name or "AI"
   - Recommendation: Use the model name from AiSettings or a generic "AI" string. Check what context approval.rs has access to.

2. **Should cli-tool-tool-approval-heading also be checked?**
   - What we know: Line 46 of en/cli.ftl has `cli-tool-tool-approval-heading` -- a separate key from the orphaned `cli-tool-approval-heading`
   - Recommendation: Verify `cli-tool-tool-approval-heading` IS used in source code. If not, it should also be removed.

## Sources

### Primary (HIGH confidence)
- Project source code -- direct grep/read of all relevant files
- `.planning/ROADMAP.md` -- current checkbox state verified
- `.planning/REQUIREMENTS.md` -- current checkbox state verified
- `git log --oneline` -- 8 commits for 16-04 confirmed
- `locales/en/cli.ftl` -- key definitions verified
- `src/i18n.rs` -- `get()` vs `get_args()` API confirmed

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, existing patterns only
- Architecture: HIGH - direct code inspection of all affected files
- Pitfalls: HIGH - i18n test already exists, patterns well-established

**Research date:** 2026-03-06
**Valid until:** 2026-03-13 (stable -- no external dependencies)
