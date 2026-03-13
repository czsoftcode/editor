---
status: diagnosed
trigger: "Terminal — adresar se zmeni, ale okno se vymaze"
created: 2026-03-05T12:00:00Z
updated: 2026-03-05T12:00:00Z
---

## Current Focus

hypothesis: Terminal content is wiped because apply_sandbox_mode_change() destroys the old Terminal instance and creates a brand new one with a fresh PTY — there is no mechanism to preserve scrollback or session content.
test: Code review of apply_sandbox_mode_change() and Terminal::new()
expecting: Confirm that old terminal is retired (graceful exit) and new terminal starts with empty buffer
next_action: Report root cause

## Symptoms

expected: Terminal restarts gracefully with new working dir, previous content preserved or clean new session shown properly
actual: Terminal content is wiped clean when switching sandbox mode
errors: None (not a crash, just UX issue)
reproduction: Toggle sandbox mode ON or OFF while terminal panel is visible
started: By design — the retire+recreate pattern always produces this

## Eliminated

(none needed — root cause found on first hypothesis)

## Evidence

- timestamp: 2026-03-05T12:00:00Z
  checked: apply_sandbox_mode_change() in state/mod.rs (lines 217-271)
  found: Method takes all claude_tabs via std::mem::take(), retires each old terminal (graceful exit), then creates brand new Terminal::new() instances with the target_root as working_dir. Same pattern for build_terminal.
  implication: Every sandbox switch destroys all terminal state — scrollback, running processes, shell history — and replaces with a blank new PTY session.

- timestamp: 2026-03-05T12:01:00Z
  checked: Terminal::new() in instance/mod.rs (lines 50-90)
  found: Creates a completely fresh TerminalBackend with new PTY process. No mechanism to transfer content from old terminal. The new terminal starts with an empty grid buffer.
  implication: "Wipe" is inherent — new PTY = new empty terminal.

- timestamp: 2026-03-05T12:02:00Z
  checked: retire_terminal() in state/mod.rs (lines 212-215)
  found: Calls request_graceful_exit() which sends "exit\n" to old PTY, then pushes to retired_terminals vec. Old terminal is ticked in background until Exit event, then dropped.
  implication: Old terminal's content is never read or transferred anywhere before destruction.

- timestamp: 2026-03-05T12:03:00Z
  checked: Terminal::create_backend() in instance/backend.rs (lines 6-43)
  found: Creates new TerminalBackend with BackendSettings containing working_directory. This spawns a new shell process. No history or scrollback injection.
  implication: The new terminal is a completely fresh shell session — expected behavior for PTY replacement, but the user sees it as "content wiped".

## Resolution

root_cause: The apply_sandbox_mode_change() method in state/mod.rs (line 243-256) uses a full destroy-and-recreate pattern for terminal instances. When sandbox mode toggles, each terminal tab is retired (old PTY receives "exit\n" and is moved to retired_terminals) and a brand new Terminal::new() is created with the target working directory. Since Terminal::new() spawns a fresh PTY/shell process via TerminalBackend, the new terminal starts with a completely empty scrollback buffer. There is no mechanism to preserve, transfer, or display any content from the old terminal session. This is inherent to PTY-based terminals — you cannot change the working directory of a running shell from outside, so the destroy+recreate approach is the correct architectural choice, but the UX impact (blank screen) is the reported issue.

fix: (not applied — diagnosis only)
verification: (not applied — diagnosis only)
files_changed: []
