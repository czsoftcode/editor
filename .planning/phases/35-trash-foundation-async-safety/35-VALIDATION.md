---
phase: 35
slug: trash-foundation-async-safety
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 35 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none |
| **Quick run command** | `cargo test phase35 -- --nocapture` |
| **Full suite command** | `cargo check && ./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test phase35 -- --nocapture`
- **After every plan wave:** Run `cargo check && ./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 35-01-01 | 01 | 1 | TRASH-03 | unit | `cargo test phase35_trash_path -- --nocapture` | ❌ W0 | ⬜ pending |
| 35-01-02 | 01 | 1 | RELIAB-01 | unit | `cargo test phase35_async_delete -- --nocapture` | ❌ W0 | ⬜ pending |
| 35-02-01 | 02 | 2 | TRASH-03 | integration | `cargo test phase35_delete_foundation -- --nocapture` | ❌ W0 | ⬜ pending |
| 35-02-02 | 02 | 2 | RELIAB-01 | gate | `cargo check && ./check.sh` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase35_trash_path.rs` — stubs pro TRASH-03
- [ ] `tests/phase35_async_delete.rs` — non-blocking flow checks pro RELIAB-01
- [ ] `tests/phase35_delete_foundation.rs` — foundation delete behavior (create/fail-closed)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Delete modal UX po failu create trash | RELIAB-01 | egui modal/toast timing je UI-driven | Potvrdit delete na read-only projektu, ověřit: modal se zavře, zobrazí se error toast, nic se nesmaže |
| Tichý úspěch create trash bez info toastu | TRASH-03 | UX preference z contextu | Smazat soubor při chybějícím `.polycredo/trash`; ověřit vznik adresáře bez info toastu |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
