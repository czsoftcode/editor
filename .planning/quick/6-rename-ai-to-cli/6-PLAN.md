---
phase: quick-6
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app/mod.rs
  - src/app/ai/ -> src/app/cli/ (directory rename)
  - src/settings.rs
  - src/app/types.rs
  - src/app/ui/ai_panel.rs
  - src/app/ui/background.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/modal_dialogs/settings.rs
  - src/app/ui/terminal/right/ai_bar.rs
  - src/app/ui/terminal/ai_chat/mod.rs
  - src/app/ui/terminal/ai_chat/render.rs
  - src/app/ui/terminal/ai_chat/logic.rs
  - src/app/ui/widgets/ai/chat/mod.rs
  - src/app/ui/widgets/ai/chat/settings.rs
autonomous: true
requirements: [QUICK-6]
must_haves:
  truths:
    - "Module crate::app::cli existuje a exportuje vše co dříve crate::app::ai"
    - "Žádný soubor neobsahuje app::ai::"
    - "cargo build --release projde bez chyb"
  artifacts:
    - path: "src/app/cli/mod.rs"
      provides: "AI manager module (přejmenovaný z ai)"
    - path: "src/app/cli/state.rs"
      provides: "AiState a související typy"
  key_links:
    - from: "src/app/mod.rs"
      to: "src/app/cli/mod.rs"
      via: "pub mod cli"
      pattern: "pub mod cli"
---

<objective>
Přejmenovat adresář src/app/ai na src/app/cli a aktualizovat všechny mod deklarace a use cesty.

Purpose: Sjednotit pojmenování modulu s již přejmenovanými i18n klíči (cli-chat-*, cli-bar-*) z fáze 17-01.
Output: Kompilující kód s app::cli místo app::ai.
</objective>

<execution_context>
@/home/stkremen/.claude/get-shit-done/workflows/execute-plan.md
@/home/stkremen/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.planning/quick/6-rename-ai-to-cli/6-CONTEXT.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Přesunout adresář ai → cli a aktualizovat mod deklaraci</name>
  <files>src/app/ai/ -> src/app/cli/, src/app/mod.rs</files>
  <action>
1. Přejmenovat adresář: `git mv src/app/ai src/app/cli`
2. V src/app/mod.rs změnit `pub mod ai;` na `pub mod cli;`

Poznámka: Rust typy (AiManager, AiState atd.) se NEPŘEJMENOVÁVAJÍ — per user decision.
  </action>
  <verify>
    <automated>ls src/app/cli/mod.rs && ! ls src/app/ai/ 2>/dev/null && grep "pub mod cli" src/app/mod.rs</automated>
  </verify>
  <done>Adresář src/app/cli existuje se všemi soubory, src/app/ai neexistuje, mod.rs deklaruje pub mod cli</done>
</task>

<task type="auto">
  <name>Task 2: Nahradit všechny app::ai:: za app::cli:: v 13 souborech</name>
  <files>
src/settings.rs
src/app/types.rs
src/app/ui/ai_panel.rs
src/app/ui/background.rs
src/app/ui/workspace/state/init.rs
src/app/ui/workspace/state/mod.rs
src/app/ui/workspace/modal_dialogs/settings.rs
src/app/ui/terminal/right/ai_bar.rs
src/app/ui/terminal/ai_chat/mod.rs
src/app/ui/terminal/ai_chat/render.rs
src/app/ui/terminal/ai_chat/logic.rs
src/app/ui/widgets/ai/chat/mod.rs
src/app/ui/widgets/ai/chat/settings.rs
  </files>
  <action>
V každém z 13 souborů nahradit všechny výskyty `app::ai::` za `app::cli::` (celkem 48 výskytů).

Použít sed: `find src/ -name '*.rs' -exec sed -i 's/app::ai::/app::cli::/g' {} +`

Po nahrazení ověřit, že žádný `app::ai::` nezůstal: `grep -r "app::ai::" src/`

NEPŘEJMENOVÁVAT nic jiného — žádné Ai prefixy typů, žádné ai_ adresáře widgetů/terminálů.
  </action>
  <verify>
    <automated>! grep -r "app::ai::" src/ && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>Žádný soubor v src/ neobsahuje "app::ai::", cargo check projde bez chyb</done>
</task>

</tasks>

<verification>
- `grep -r "app::ai::" src/` vrátí 0 výsledků
- `cargo check` projde bez chyb
- `cargo test` projde (zejména all_lang_keys_match_english)
</verification>

<success_criteria>
- Adresář src/app/cli/ existuje se všemi 9 soubory (audit.rs, executor.rs, mod.rs, ollama.rs, provider.rs, security.rs, state.rs, tools.rs, types.rs)
- Adresář src/app/ai/ neexistuje
- Všech 48 výskytů app::ai:: nahrazeno za app::cli::
- cargo check + cargo test projdou bez chyb
</success_criteria>

<output>
After completion, create `.planning/quick/6-rename-ai-to-cli/6-SUMMARY.md`
</output>
