# Phase 30: CLI Namespace Removal Foundation - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit vazby na `app::cli` namespace a pripravit cisty zaklad pro AI terminal-only architekturu.
Faze 30 je foundation: ciste namespace a build stabilita. Behavioral parity a dodelani runtime detailu jsou navazujici prace (phase 31/32), pokud nejsou explicitne zamknute nize.

</domain>

<decisions>
## Implementation Decisions

### Boundary and done criteria
- Faze 30 je splnena jako `no app::cli imports + builds`.
- V teto fazi je akceptovany compile-first baseline (ne plna UX parita).
- Quality gate pro fazi 30: `./check.sh` je povinny.

### Safety and slash tolerance
- Approval/security cast je dovoleno cistit uz v phase 30 (hard cleanup), ne jen drzet wire-compat.
- Docasne regrese slash/GSD jsou akceptovatelne v phase 30, pokud je jasny plan navratu v phase 31.

### Target namespace shape
- Cilovy namespace: `app::ai_core`.
- Moduly budou runtime-only split (jen to, co AI terminal realne potrebuje).
- Public API bude minimalni (jen realne pouzite exporty).
- Migrace bude `cisty rez` bez docasnych compat aliasu.

### AI terminal UI scope lock
- Z AI terminalu odstranit Ollama modely a stav spojeni.
- Zachovat cisty "Assistant" flow: combobox + spustit.
- Externi agent integrace z AI terminalu zustava mozne pouzivat, ale ne pres puvodni `app::cli` vrstvu.

### Cleanup policy
- Dead code mazat uz v phase 30, pokud neni runtime-critical.
- Docs/planning aktualizovat prubezne, ne az na konci.
- Pri riziku regrese pouzit "stop-and-fix in-phase" (ne odkladat).

### Claude's Discretion
- Presne rozhrani `app::ai_core` modulu (vnitri cleneni souboru).
- Poradi konkretni migrace importu podle nejmensiho rizika.
- Minimalni smoke/regression test set navic k `./check.sh`.

</decisions>

<specifics>
## Specific Ideas

- Cilem neni pridavat nove schopnosti; cilem je odstranit slepou ulicku `src/app/cli/*`.
- AI terminal ma zustat jednodussi: bez modelove/Ollama stavove vrstvy v UI.
- UX ma smerovat na jediny vstup "Assistant" bez provider-specific ovladacich prvku.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ui/terminal/ai_chat/*` uz obsahuje hlavni UI/render/logiku chatu.
- `src/app/ui/background.rs` obsahuje polling/tool flow navazany na `app::cli::*`.
- `src/app/ui/workspace/state/mod.rs` drzi `tool_executor` a AI runtime stav.

### Established Patterns
- Stavove vetve a async polling jsou centralizovane ve workspace/background vrstve.
- Settings a types importuji `AiExpertiseRole`/`AiReasoningDepth` z `app::cli`.
- Build/test gate v projektu je standardizovany pres `./check.sh`.

### Integration Points
- Primarni migrovane odkazy: `settings.rs`, `app/types.rs`, `app/mod.rs`, `ui/background.rs`, `ui/workspace/state/*`, `ui/terminal/ai_chat/*`.
- Root export `pub mod cli;` v `src/app/mod.rs` je centralni bod odstraneni stare vrstvy.

</code_context>

<deferred>
## Deferred Ideas

- Plna behavioral parity slash/GSD po cleanup foundation bude dodelana v navazujici fazi.
- Dalsi UX redesign AI terminalu je mimo scope phase 30.

</deferred>

---
*Phase: 30-cli-namespace-removal-foundation*
*Context gathered: 2026-03-11*
