# Phase 31 Research: AI Terminal Runtime Migration

Datum: 2026-03-11  
Faze: 31-ai-terminal-runtime-migration  
Scope: TERM-01, TERM-02, TERM-03, SAFE-01, SAFE-02, SAFE-03  
Stav: ready-for-planning

## Research Summary
Phase 31 je migracni faze s pozadavkem na behavior parity: AI terminal musi fungovat bez puvodni PolyCredo CLI runtime vrstvy, ale bez uvolneni safety kontraktu. V kodu uz existuje cilovy namespace `app::ai_core`, ai chat tok (`ui/terminal/ai_chat/*`) i approval/security/audit komponenty. Plan faze ma byt o kontrolovanem presmerovani runtime vazeb, odstraneni legacy semantiky (hlavne CLI branding/chovani), a dukazech parity na TERM/SAFE pozadavcich.

Nejvyssi riziko neni "jestli to zkompiluje", ale regresni chovani v event loopu (`ui/background.rs`) a v approval/tool-call resume cyklu. Plan proto musi byt rezany podle runtime toku (send -> stream -> tool call -> approval -> resume), ne jen podle souboru.

## Source Inputs
- `.planning/phases/31-ai-terminal-runtime-migration/31-CONTEXT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `./CLAUDE.md`
- `src/app/ai_core/*`
- `src/app/ui/terminal/ai_chat/*`
- `src/app/ui/background.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/ui/terminal/right/ai_bar.rs`
- `src/app/ui/ai_panel.rs`

## Scope Lock (Phase 31)
- In-scope:
  - Runtime migrace provider/executor/tooling tak, aby AI terminal bezel bez puvodni CLI vrstvy.
  - Udrzeni funkcniho prompt->stream toku a slash/GSD workflow.
  - Udrzeni approval/security/audit kontraktu beze zmen UX semantiky schvalovani.
- Out-of-scope:
  - Novy provider stack nebo redesign UX panelu.
  - Uvolneni guardu nebo zmena policy SAFE kontraktu.
  - Velke cross-module refaktory mimo AI terminal runtime trasu.

## Current Runtime Topology (co je dulezite pro plan)
- Runtime jadro je v `src/app/ai_core`:
  - `provider.rs` (trait + stream event kontrakt)
  - `ollama.rs` (runtime provider implementace)
  - `executor.rs` (tool dispatch + approval + audit + path sandbox)
  - `security.rs` (PathSandbox, FileBlacklist, CommandBlacklist, RateLimiter, SecretsFilter)
  - `audit.rs` (file audit log)
- UI/flow body:
  - `ai_chat/logic.rs` startuje chat request, sklada zpravy, inicializuje `ToolExecutor`, startuje stream.
  - `ui/background.rs` je centralni event pumpa: stream tokeny, ToolCall eventy, approval response, slash async vysledky.
  - `ai_chat/slash.rs` drzi slash dispatch vcetne `/gsd` a async `/build` `/git`.
  - `workspace/state/mod.rs` drzi runtime state pro approval/slash asynchronii.

## Requirement Coverage Strategy
- TERM-01 (otevrit panel + odeslat prompt):
  - Drzet beze zmen otevreni panelu a `send_query_to_agent()` vstup.
  - Pri migraci nesmi vzniknout path, kde prompt projde, ale stream se nespusti.
- TERM-02 (streaming bez zamrznuti UI):
  - Zachovat non-blocking model (`stream_chat` + `try_recv` polling v `background.rs`).
  - Zakazat blokujici operace v render a input ceste.
- TERM-03 (model picker + slash/GSD funkcni):
  - Udrzet selected model tok (`ws.ai.ollama.selected_model`) a slash dispatch (`/gsd` v `slash.rs`).
  - Otestovat /build, /git placeholder -> final update v conversation.
- SAFE-01 (approval flow):
  - Zachovat `ToolResult::NeedsApproval` -> `PendingToolApproval` -> `process_approval_response` -> resume.
- SAFE-02 (security filtry/rate limit/path sandbox):
  - Migrace nesmi obchazet `ToolExecutor` security check body.
  - Zachovat `PathSandbox`, `FileBlacklist`, `CommandBlacklist`, `RateLimiter` v runtime ceste.
- SAFE-03 (audit/error handling):
  - Zachovat audit log zapis pro tool call i security eventy.
  - Chyby nesmi utichnout; musi jit do UI (toast nebo zretelna chat zprava).

## Standard Stack
- Jazyk/UI runtime:
  - Rust + `eframe/egui` (stávající event-loop architecture)
- AI runtime vrstva:
  - `app::ai_core::provider` (trait boundary)
  - `app::ai_core::runtime_provider::OllamaProvider` (aktualni provider implementace)
  - `app::ai_core::executor::ToolExecutor` (native tooling orchestrace)
- Bezpecnostni vrstva:
  - `PathSandbox`, `FileBlacklist`, `CommandBlacklist`, `RateLimiter`, `SecretsFilter`
- Async primitives:
  - `std::sync::mpsc` + background threads + `try_recv` polling
- Planning rozhodnuti:
  - Pro phase 31 nepridavat nove crates; migrace ma stavet na existujicich modulech.

## Architecture Patterns
1. Runtime boundary pattern (`ui` -> `ai_core`)
- UI vrstva nema implementovat bezpecnost ani tool policy; pouze predava eventy/stav.
- Ve phase planu kontrolovat, ze vsechny tool call pathy jdou pres `ToolExecutor`.

2. Stream state machine pattern
- Kriticke stavy: `idle -> streaming -> tool_call_pending -> approval_pending -> resume_stream -> done`.
- Plan ma explicitne mapovat, kde se stav meni (`logic.rs`, `background.rs`, `workspace/state`).

3. Async placeholder replacement pattern (slash)
- `/build` a `/git` pouzivaji placeholder zpravy a pozdejsi replacement.
- Nutne drzet generation guard (`slash_conversation_gen` / `slash_build_gen`) proti stale update po `/clear` nebo `/new`.

4. Safety-preserving adapter pattern
- Kde je jeste legacy CLI semantika (branding/texty/kontrakty), migrovat adapterove: minimalni patch, stejny vstup/vystup, bez zmeny guard behavior.

## Don't Hand-Roll
- Nepsat novy tool executor ani vlastni security vrstvu mimo `ai_core/executor.rs` a `ai_core/security.rs`.
- Nepsat novy audit logger; pouzit `ai_core/audit.rs`.
- Nezavadet druhy paralelni stream handling mimo `ui/background.rs`.
- Neobchazet slash dispatch lokálními if-vetvemi v random UI modulech; centralni bod je `ai_chat/slash.rs`.
- Nezavadet custom approval UX mimo `ai_chat/approval.rs` + `PendingToolApproval` state.

## Common Pitfalls
- Coupling na legacy CLI slovnik/branding v user-facing textech:
  - Napr. `AiManager::get_logo()` stale vraci ASCII logo s `CLI` suffixem; to je semanticky konflikt s terminal-only boundary.
- Tichy rozpad SAFE toku:
  - Snadne je zachovat happy path, ale rozbit `NeedsApproval` nebo `Deny` branch pri refactoru `background.rs`.
- Stream freeze regrese:
  - Jakykoli blokujici call v frame loopu (`process_background_events`) zhodi TERM-02.
- Slash strictness drift:
  - V `slash.rs` je dispatch pres `to_lowercase()`, coz implicitne povoluje velka pismena; pokud produktove pravidlo chce strict lowercase, plan musi rozhodnout a otestovat.
- Error visibility gap:
  - Cast chyb jde do conversation, cast do toastu; pri migraci muze cast chyb zmizet bez user feedbacku.

## Code Examples
```rust
// src/app/ui/terminal/ai_chat/logic.rs
// Kriticky vstupni bod: slash se musi vyresit pred provider connectivity check.
if ws.ai.chat.prompt.starts_with('/') {
    super::slash::dispatch(ws, shared);
    return;
}
```

```rust
// src/app/ui/background.rs
// Kriticky SAFE-01 tok: tool call -> approval -> resume.
if let Some(ref mut executor) = ws.tool_executor {
    let result = executor.execute(&name, &arguments);
    match result {
        ToolResult::NeedsApproval { .. } => { /* pending approval UI */ }
        ToolResult::Success(output) => { /* resume_after_tool_call */ }
        ToolResult::Error(msg) => { /* visible error + resume */ }
        _ => {}
    }
}
```

```rust
// src/app/ai_core/security.rs
// SAFE-02 jadro: path validace proti traversal utokum.
pub fn validate_path(&self, relative: &str) -> Result<PathBuf, String> {
    // absolute path guard + canonical root check + nonexistent parent validation
}
```

## Planning Slices (doporučené baliky pro PLAN)
1. Runtime parity audit slice
- Cíl: sepsat a opravit vsechny body, kde je jeste legacy CLI semantika v AI terminal runtime toku.
- DoD: zadna funkcni vazba na puvodni CLI runtime; jen terminal-only chovani.

2. TERM flow hardening slice
- Cíl: stabilizovat prompt->stream->done a slash async update path.
- DoD: TERM-01/02/03 prokazane manualnim smoke matrixem + testy kde realne mozne.

3. SAFE flow hardening slice
- Cíl: potvrdit approval/security/audit invariants behem migrace.
- DoD: SAFE-01/02/03 pokryte testy + explicitni overeni error visibility.

4. Cleanup + evidence slice
- Cíl: odstranit mrtve runtime vazby a zanechat dukazy pro acceptance.
- DoD: cisty grep/build gate + plan artefakty s mapou requirement -> dukaz.

## Validation Architecture
Validace musi byt dvouurovnova: (A) prubezna po kazdem baliku zmen, (B) finalni acceptance gate pro TERM/SAFE.

A. Prubezna validace (po kazdem patchi)
- `cargo check`.
- Cileny smoke run AI terminalu:
  - otevrit panel, odeslat prompt, overit ze se streamuje token output.
  - spustit `/help`, `/gsd help`, `/git`, `/build`.
- Pri failu stop-and-fix v tom samem baliku, nepokracovat.

B. Requirement-based acceptance matrix
- TERM-01:
  - otevreni panelu + odeslani promptu + vznik nove conversation polozky.
- TERM-02:
  - behem streamu UI reaguje (scroll, input, repaint bez freezu).
- TERM-03:
  - model selection zustava platna, `/gsd` dispatch funguje, `/build` a `/git` async update probiha.
- SAFE-01:
  - vynutit `NeedsApproval` tool, overit approve i deny cestu.
- SAFE-02:
  - test path traversal attempt, blocked file pattern, command blacklist/rate limit.
- SAFE-03:
  - overit zapis do `.polycredo/ai-audit.log` a user-visible chybu pri selhani.

C. Nyquist gate test inventory (minimum)
- Unit/integration:
  - `ai_core/executor.rs` approval/security testy (existujici + doplnit chybejici okraje migrace).
  - `ai_chat/slash.rs` dispatch strictness + async placeholder update chovani.
- Manual E2E smoke:
  - prompt streaming, tool approval modal, deny/approve retry, slash/GSD command chain.

D. Final gate
- Povinne: `cargo check` + `./check.sh`.
- Povinne: requirement checklist s explicitnimi dukazy pro TERM-01..03, SAFE-01..03.
- Povinne: grep/audit sanity pass, ze nezustaly legacy CLI runtime vazby v AI terminal toku.

## Open Decisions For Planning (co rozhodnout pred PLAN)
- Presna policy pro slash strict lowercase (soucasny kod je case-insensitive dispatch).
- Jak nalozit s CLI branding stringy (napr. logo text) bez UX regressions.
- Jestli pri SAFE-03 sjednotit error surfacing strategii (toast vs system message) nebo jen dokumentovat stavajici behavior.

## Ready-to-Plan Checklist
- [x] Scope lock proti phase 31 boundary.
- [x] Pokryti TERM-01/02/03 a SAFE-01/02/03.
- [x] Definovana validační architektura pro Nyquist gate.
- [x] Identifikovane kriticke rizikove body v runtime event loopu.
- [x] Definovane doporucene implementacni baliky pro PLAN.

## RESEARCH COMPLETE
