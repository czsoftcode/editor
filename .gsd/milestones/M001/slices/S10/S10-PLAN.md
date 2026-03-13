# S10: Slash Command Infrastructure

**Goal:** Create the slash command dispatch system with command registry, integrate it into the chat prompt flow, and implement synchronous built-in commands (/help, /clear, /new, /settings).
**Demo:** Create the slash command dispatch system with command registry, integrate it into the chat prompt flow, and implement synchronous built-in commands (/help, /clear, /new, /settings).

## Must-Haves


## Tasks

- [x] **T01: 19-slash-command-infrastructure 01** `est:3min`
  - Create the slash command dispatch system with command registry, integrate it into the chat prompt flow, and implement synchronous built-in commands (/help, /clear, /new, /settings). Add system message rendering with distinct background color.

Purpose: This is the foundation for all slash commands. Users can interact with the editor through / prefixed commands in the chat panel without needing an AI connection.
Output: Working /help, /clear, /new, /settings commands with system message rendering.
- [x] **T02: 19-slash-command-infrastructure 02** `est:8min`
  - Implement the three async/stateful slash commands: /model (list/switch), /git (background diff --stat), and /build (background cargo build with in-place message update).

Purpose: These commands require either state access (model list) or background thread execution (git, build) and build on the dispatch infrastructure from Plan 01.
Output: All 7 built-in slash commands fully functional.
- [x] **T03: 19-slash-command-infrastructure 03** `est:1min`
  - Fix /git command output rendering so fenced code blocks display correctly as multi-line content.

Purpose: The path regex in flush_block replaces file paths inside fenced code blocks with markdown link syntax, breaking the code block. Adding code-fence awareness prevents regex replacement inside ``` blocks.
Output: render.rs with code-fence state tracking in flush_block
- [x] **T04: 19-slash-command-infrastructure 04** `est:5min`
  - Add slash command autocomplete popup that appears when typing `/` in the chat prompt.

Purpose: Users expect command suggestions as they type, not just after pressing Enter. The popup provides discoverability and efficient command entry.
Output: Working autocomplete with filtered suggestions, keyboard navigation, and Tab/Enter selection.

## Files Likely Touched

- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/terminal/ai_chat/mod.rs`
- `src/app/ui/terminal/ai_chat/logic.rs`
- `src/app/ui/widgets/ai/chat/conversation.rs`
- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/background.rs`
- `src/app/ui/widgets/ai/chat/render.rs`
- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/widgets/ai/chat/input.rs`
- `src/app/ui/terminal/ai_chat/render.rs`
