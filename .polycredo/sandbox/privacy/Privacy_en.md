# PolyCredo Editor — Privacy Policy

**Last updated:** 2026-02-20

---

## What PolyCredo Editor Is

PolyCredo Editor is a desktop code editor. It is a graphical interface for working with files on your machine and for running AI CLI tools (such as Claude Code or Gemini CLI) in an integrated terminal.

---

## Data Collection

**PolyCredo Editor does not collect any data.**

Specifically, the editor:

- does not send your source code, file contents, or file paths to any server
- does not contain telemetry, analytics, or crash reporting
- does not communicate with any external API on its own
- does not require an account or registration
- does not store any data outside your machine

All state the editor saves (open sessions, recent projects, settings) is stored locally in `~/.config/polycredo-editor/`.

---

## AI Tools

PolyCredo Editor integrates with third-party AI CLI tools such as **Claude Code** (Anthropic) and **Gemini CLI** (Google). These tools run as separate processes in the editor's integrated terminal — the same way they would run if you launched them in any terminal application.

**The editor does not control, intercept, or forward any data these tools send or receive.**

When you use an AI tool inside PolyCredo Editor:

- the tool communicates directly with its provider's servers (Anthropic, Google, etc.)
- the provider's own privacy policy and terms of service apply
- you are responsible for what you share with the AI tool, exactly as you would be if you ran it in a standalone terminal

PolyCredo Editor's role is equivalent to that of a terminal emulator — it displays output, it does not process or forward the content.

### Offline / Local AI

If you use a locally running model (e.g. via Ollama + Aider), no data leaves your machine. PolyCredo Editor supports any AI tool that runs as a CLI process, including fully local solutions.

---

## Responsibility

| Who | Responsibility |
|---|---|
| **PolyCredo Editor author** | Ensuring the editor itself sends no data without explicit user action |
| **User** | Choosing which AI tools to use and accepting their terms |
| **AI tool provider** (Anthropic, Google, …) | Data handling by their tool under their privacy policy |

---

## Cloud Features

Any future cloud features (settings sync, snippet sharing, etc.) will be:

- **opt-in** — disabled by default
- clearly documented as to what data they send and where
- fully optional — the editor works completely without them

---

## Contact

Issues and questions: [https://github.com/czsoftcode/editor/issues](https://github.com/czsoftcode/editor/issues)
