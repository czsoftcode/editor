# T01: 19-slash-command-infrastructure 01

**Slice:** S10 — **Milestone:** M001

## Description

Create the slash command dispatch system with command registry, integrate it into the chat prompt flow, and implement synchronous built-in commands (/help, /clear, /new, /settings). Add system message rendering with distinct background color.

Purpose: This is the foundation for all slash commands. Users can interact with the editor through / prefixed commands in the chat panel without needing an AI connection.
Output: Working /help, /clear, /new, /settings commands with system message rendering.

## Must-Haves

- [ ] "User types /help and sees a markdown table of all available commands in the conversation"
- [ ] "User types /clear and conversation is emptied, token counters reset, prompt history preserved"
- [ ] "User types /new and conversation resets with PolyCredo ASCII logo displayed"
- [ ] "User types /settings and the settings modal opens"
- [ ] "Slash commands work even when Ollama is disconnected"
- [ ] "Unknown command close to a registered name shows fuzzy suggestion as system message"
- [ ] "Unrecognized /word passes through to AI as normal prompt"
- [ ] "System messages render with distinct background color"

## Files

- `src/app/ui/terminal/ai_chat/slash.rs`
- `src/app/ui/terminal/ai_chat/mod.rs`
- `src/app/ui/terminal/ai_chat/logic.rs`
- `src/app/ui/widgets/ai/chat/conversation.rs`
