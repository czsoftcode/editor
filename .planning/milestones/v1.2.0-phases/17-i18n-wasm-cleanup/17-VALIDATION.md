---
phase: 17
slug: i18n-wasm-cleanup
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-06
validated: 2026-03-06
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test all_lang_keys_match_english` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~1.5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check && cargo test all_lang_keys_match_english`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 17-01-01 | 01 | 1 | CLEN-03 | unit | `cargo test all_lang_keys_match_english` | Exists | ✅ green |
| 17-01-02 | 01 | 1 | CLEN-03 | manual | `grep -rn '[áčďéěíňóřšťúůýž]' src/app/ui/terminal/ai_chat/approval.rs` | N/A | ✅ green |
| 17-02-01 | 02 | 2 | CLEN-02 | smoke | `cargo check` | N/A | ✅ green |
| 17-02-02 | 02 | 2 | CLEN-02 | unit | `cargo test` | Exists | ✅ green |
| 17-03-01 | 03 | 1 | CLEN-03 | unit | `cargo test all_lang_keys_match_english` | Exists | ✅ green |
| 17-03-02 | 03 | 1 | CLEN-02 | smoke | `cargo check 2>&1 \| grep "warning.*src/app/ai/"` | N/A | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

- `all_lang_keys_match_english` test in `src/i18n.rs` validates i18n key parity across all 5 languages
- `cargo check` validates compilation after WASM removal
- Full `cargo test` suite (182 tests) validates no regressions

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No hardcoded Czech strings in approval.rs | CLEN-03 | String content, not structure | grep for Czech diacritics in approval.rs after i18n migration |
| Settings UI shows Ollama params | CLEN-03 | Visual verification | Open Settings, check AI section has temperature, top_p, top_k, etc. |
| Editor runs without WASM runtime | CLEN-02 | Runtime behavior | Launch editor, open AI chat, send message — no WASM errors |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-03-06

---

## Validation Audit 2026-03-06

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total tasks verified | 6 |
| Tests passing | 182 |

### Verification Results

- `cargo test` — 182 passed, 0 failed
- `cargo test all_lang_keys_match_english` — PASS
- `cargo check` — clean, no warnings
- grep Czech diacritics in approval.rs — 0 hits
- grep `extism|PluginManager|PluginSettings` in src/ — 0 hits
- grep `ai-chat-|ai-plugin-bar-` in src/ — 0 hits
