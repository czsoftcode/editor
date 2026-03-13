---
status: diagnosed
trigger: ".polycredo/ je v .gitignore, takze git add na soubory v sandboxu nefunguje — blokace sandbox OFF pri staged souborech se nemuze nikdy aktivovat"
created: 2026-03-05T00:00:00Z
updated: 2026-03-05T00:00:00Z
---

## Current Focus

hypothesis: UAT report is INCORRECT — the feature does NOT use git at all
test: Read all code involved in staged file detection
expecting: Confirm that "staged" means sandbox-vs-project diff, not git staging
next_action: Report diagnosis

## Symptoms

expected: should_block_sandbox_off_due_to_staged prevents turning sandbox OFF when there are staged files
actual: UAT claims .polycredo/ in .gitignore blocks this feature
errors: none
reproduction: n/a — this is a design analysis issue
started: UAT Phase 05, Test 6

## Eliminated

- hypothesis: The feature uses git commands (git diff --cached, git status) to detect staged files
  evidence: Sandbox::get_staged_files() does filesystem comparison (xxh3 hash), zero git commands
  timestamp: 2026-03-05

## Evidence

- timestamp: 2026-03-05
  checked: .gitignore contents
  found: Line 4 has ".polycredo/sandbox/" — sandbox IS gitignored
  implication: git add on sandbox files would indeed fail, BUT this is irrelevant

- timestamp: 2026-03-05
  checked: Sandbox::get_staged_files() in src/app/sandbox.rs lines 257-304
  found: Method walks sandbox directory with WalkDir, compares each file to project_root counterpart using file size + xxh3 hash. Returns files that differ or are new. Zero git interaction.
  implication: "staged" in this context means "sandbox files that differ from project" — NOT git staged files

- timestamp: 2026-03-05
  checked: trigger_sandbox_staged_refresh() in src/app/ui/workspace/mod.rs lines 32-46
  found: Creates Sandbox::new_with_roots() and calls sb.get_staged_files(). Result stored in ws.sandbox_staged_files.
  implication: The staged files cache is populated by filesystem diff, not git

- timestamp: 2026-03-05
  checked: should_block_sandbox_off_due_to_staged() in settings.rs lines 41-46
  found: Simply checks `change == Disabled && !staged_files.is_empty()`. staged_files comes from ws.sandbox_staged_files.
  implication: Guard works correctly — blocks sandbox OFF when sandbox has modified/new files vs project

- timestamp: 2026-03-05
  checked: render_sandbox_staged_bar() in mod.rs lines 626-655
  found: Shows warning bar with review/promote-all buttons when sandbox_staged_files is non-empty. "Promote" copies files from sandbox to project.
  implication: The entire "staged" concept is sandbox-internal, not git-related

## Resolution

root_cause: FALSE POSITIVE — The UAT report is incorrect. The term "staged" in this codebase does NOT refer to git staging (`git add`). It refers to files in the sandbox that differ from their project-root counterparts, detected via filesystem hash comparison (xxh3). The `.polycredo/sandbox/` entry in `.gitignore` is completely irrelevant to this feature because the feature never interacts with git at all.

fix: No code fix needed. The UAT test was based on a misunderstanding of the term "staged" in this context.

verification: Code audit confirms Sandbox::get_staged_files() uses WalkDir + xxh3 hash comparison, not git commands.

files_changed: []
