---
status: diagnosed
trigger: "/git command output renders on a single line instead of as a multi-line code block"
created: 2026-03-07T12:00:00Z
updated: 2026-03-07T12:00:00Z
---

## Current Focus

hypothesis: render_markdown line-by-line processing strips code block fences before CommonMarkViewer sees them
test: trace how triple-backtick lines are handled in the for-loop
expecting: fences are flushed as separate blocks, breaking the code block structure
next_action: report root cause

## Symptoms

expected: Git diff output appears as multi-line code block with each file on its own line
actual: Output renders on a single line
errors: none
reproduction: run /git command in AI chat panel
started: since /git command was added

## Eliminated

(none needed -- root cause found on first hypothesis)

## Evidence

- timestamp: 2026-03-07T12:01:00Z
  checked: cmd_git output format (slash.rs:280)
  found: produces correct markdown with triple backticks wrapping multi-line git diff output
  implication: data is correct, problem is in rendering

- timestamp: 2026-03-07T12:02:00Z
  checked: conversation.rs system message path (line 150)
  found: system messages call render_markdown(ui, display_content, ...) correctly
  implication: rendering pipeline is invoked, problem is inside render_markdown

- timestamp: 2026-03-07T12:03:00Z
  checked: render_markdown in render.rs lines 46-93
  found: ROOT CAUSE -- the function iterates line-by-line and flushes accumulated blocks whenever it encounters a mode switch (monologue vs normal). The logic checks `is_mono_line` (lines starting with '>' or "Step") but has NO awareness of fenced code blocks (``` markers). Each line of the code block content is accumulated in current_block as a normal (non-mono) block and flushed to CommonMarkViewer individually or as a group, but the triple-backtick fence lines themselves cause flush boundaries. The key issue: when the function encounters the opening ``` line, it adds it to current_block. Then subsequent content lines are also added. Then the closing ``` is added. This SHOULD work if no mode switches happen. BUT the real problem is that the path_re regex in flush_block (lines 119-131) transforms file paths inside the code block into markdown links like `[path](path)`, which breaks the code block syntax. Inside a fenced code block, content should be literal -- but the regex replacement converts paths to link syntax, destroying the code block's plain-text semantics. Additionally, CommonMarkViewer may re-parse the already-mangled content.
  implication: two issues compound: (1) regex path replacement inside code blocks, and (2) potential line-by-line flushing if any line triggers mode-switch logic

## Resolution

root_cause: |
  The `render_markdown` function in `src/app/ui/widgets/ai/chat/render.rs` has two compounding issues that break fenced code blocks:

  1. **Path regex replacement inside code blocks (primary):** The `flush_block` function (line 119-131) applies a regex that converts file paths into markdown link syntax `[path](path)` on ALL text, including content inside fenced code blocks. Git diff --stat output contains file paths like `src/foo/bar.rs | 5 +++--`, which get transformed into `[src/foo/bar.rs](path) | 5 +++--`. This breaks the fenced code block because CommonMarkViewer now sees link markup inside what should be literal code content.

  2. **No code-fence awareness in line processing (secondary):** The line-by-line iteration (lines 46-93) checks for monologue markers (`>`, `Step`) but has zero awareness of fenced code block state. If any line inside a code block happens to start with `>` (which is unlikely for git diff but possible for other content), it would trigger a premature flush that splits the code block across multiple CommonMarkViewer calls, each seeing incomplete markdown.

  The primary cause for /git specifically is issue #1: the path regex mangles the code block content before CommonMarkViewer ever sees it.

fix: (not applied -- diagnose only)
verification: (not applicable)
files_changed: []
