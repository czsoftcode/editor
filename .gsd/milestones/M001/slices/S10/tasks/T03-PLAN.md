# T03: 19-slash-command-infrastructure 03

**Slice:** S10 — **Milestone:** M001

## Description

Fix /git command output rendering so fenced code blocks display correctly as multi-line content.

Purpose: The path regex in flush_block replaces file paths inside fenced code blocks with markdown link syntax, breaking the code block. Adding code-fence awareness prevents regex replacement inside ``` blocks.
Output: render.rs with code-fence state tracking in flush_block

## Must-Haves

- [ ] "Git diff output from /git renders as a multi-line code block with each file on its own line"
- [ ] "Path regex replacement does NOT alter content inside fenced code blocks"
- [ ] "Other markdown content outside code blocks still gets path highlighting"

## Files

- `src/app/ui/widgets/ai/chat/render.rs`
