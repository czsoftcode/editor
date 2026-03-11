# Phase 31: AI Terminal Runtime Migration - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Odstranit vazby na puvodni PolyCredo CLI runtime vrstvu v kodu i UI a zachovat funkcni AI terminal workflow.
Tato faze neresi nove capability; jen migraci runtime/provider/executor/tooling casti na terminal-only smer.

</domain>

<decisions>
## Implementation Decisions

### Product boundary (locked)
- PolyCredo CLI jde pryc z kodu i UI.
- Uzivatelsky zustava pouze AI terminal.
- Neni dovolene nechavat fallback UI prvky navazane na puvodni PolyCredo CLI.

### Slash command policy
- V AI terminalu zustavaji slash prikazy externich asistentu (napr. Claude Code, Codex).
- Interne PolyCredo slash prikazy patri puvodni CLI vrstve a maji byt odstraneny.
- Chovani terminalu ma byt po migraci konzistentni s externim asistentem, ne s legacy PolyCredo slash infrastrukturou.

### Failure UX (runtime)
- Pri docasnem selhani asistenta zobrazit kratkou srozumitelnou hlasku.
- Nabidnout jednoduchou moznost zkusit akci znovu (Retry).
- Nezobrazovat uzivateli zbytecne technicke detaily, pokud nejsou potreba.

### Safety / approval policy
- Approval a security guardy zustavaji funkcne stejne jako dosud.
- V phase 31 se guardy neuvolnuji ani nepritvrzuji.
- SAFE-01/SAFE-02/SAFE-03 se maji splnit migraci bez zmeny UX kontraktu schvalovani.

### Claude's Discretion
- Presne mapovani internich modulu a poradi migrace runtime casti (`provider`, `executor`, `tools`, `security`, `audit`).
- Konretni podoba interniho cleanupu dead-code po CLI odstraneni.
- Test matrix nad `TERM-*`/`SAFE-*` pro prokazani parity.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/app/ai_core/*`: cilova runtime vrstva (provider/executor/tools/security/audit/state).
- `src/app/ui/terminal/ai_chat/*`: stavajici AI terminal workflow (logic/render/approval/slash/gsd).
- `src/app/ui/terminal/right/ai_bar.rs`: vstupni ovladani AI terminalu po assistant-only locku z phase 30.

### Established Patterns
- Build/quality gate: `cargo check` + `./check.sh`.
- Security a audit body jsou centralizovane v `ai_core/security.rs` a `ai_core/audit.rs`.
- Chat akce a stavove toky jsou vedeny pres `ui/terminal/ai_chat/mod.rs` a navazne moduly.

### Integration Points
- Runtime migrace se opira o `ai_core` a navazne call-sites v `ui/terminal/ai_chat/*` a `ui/ai_panel.rs`.
- Approval flow je integrovany pres `ui/terminal/ai_chat/approval.rs`.
- Slash/GSD body jsou v `ui/terminal/ai_chat/slash.rs` a `ui/terminal/ai_chat/gsd/*` a musi respektovat novu boundary (bez legacy PolyCredo slash).

</code_context>

<specifics>
## Specific Ideas

- "AI chat je PolyCredo CLI a to ma jit pryc, ma zustat jenom AI terminal."
- "V AI terminalu bezi jenom externi asistenti (Claude Code, Codex...) a maji svoje slash prikazy."
- "Vlastni slash byly jenom pro PolyCredo CLI."

</specifics>

<deferred>
## Deferred Ideas

- Pripadny novy jednotny slash abstraction layer nad externimi asistenty (pokud bude potreba) je samostatna budouci faze.

</deferred>

---
*Phase: 31-ai-terminal-runtime-migration*
*Context gathered: 2026-03-11*
