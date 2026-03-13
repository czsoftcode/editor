# T04: 19-slash-command-infrastructure 04

**Slice:** S10 — **Milestone:** M001

## Description

Add slash command autocomplete popup that appears when typing `/` in the chat prompt.

Purpose: Users expect command suggestions as they type, not just after pressing Enter. The popup provides discoverability and efficient command entry.
Output: Working autocomplete with filtered suggestions, keyboard navigation, and Tab/Enter selection.

## Must-Haves

- [ ] "Typing / in the chat prompt opens an autocomplete popup listing all commands"
- [ ] "Typing more characters after / filters the command list"
- [ ] "Arrow keys navigate the popup, Enter/Tab selects, Escape dismisses"
- [ ] "Selecting a command replaces the prompt text with the command"

## Files

- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/widgets/ai/chat/input.rs`
- `src/app/ui/terminal/ai_chat/render.rs`
