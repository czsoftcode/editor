# Phase 1: Repaint Gate - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Zastavit zbytečné překreslování render loopu v idle stavu. Editor má repaintovat pouze při:
- uživatelském vstupu (klávesnice, myš)
- příchodu dat z pozadí (AI odpověď, build výstup, file watcher event)

Vytváření nových UI funkcí ani změny v background threadech (to je Phase 2) nejsou součástí.

</domain>

<decisions>
## Implementation Decisions

### Idle repaint chování
- Čistě event-driven: v idle stavu žádný repaint — Claude rozhoduje o implementaci
- Žádný heartbeat v idle; eframe defaultně čeká na eventi pokud není volán request_repaint()

### Unfocused / minimalizované okno
- Minimalizované: zastavit repaints úplně (nebo max 1× za 5 s dle RPNT-03)
- Nezaměřené okno: throttlovat — Claude rozhoduje o konkrétním intervalu (1–5 s)
- Implementovat přes `ctx.input(|i| i.viewport().focused)` + podmíněný request_repaint_after

### FPS cap při psaní
- Omezit na rozumný cap při aktivním psaní — Claude rozhoduje (doporučeno: request_repaint_after(33ms) = ~30fps)
- Žádný bezpodmínečný request_repaint() po každém keystroke (RPNT-04)

### Accesskit
- Zakázat accesskit feature v Cargo.toml (RPNT-02)
- Ověřit při plánování, zda feature existuje — průzkum kódu neukazuje accesskit dependency

### Priorita repaintů z pozadí
- Urgentní výstupy (AI odpověď, build dokončen): request_repaint() může zůstat okamžitý
- Polling z pozadí (git status, file watcher): request_repaint_after(N) — Claude rozhoduje o intervalu
- Centrální koordinace není požadována — Claude rozhoduje o nejjednodušším přístupu

### Claude's Discretion
- Konkrétní hodnota FPS capu při psaní (doporučení: 30fps = 33ms)
- Konkrétní interval pro unfocused window throttle (doporučení: 2–5 s)
- Konkrétní interval pro background polling repainty (doporučení: 100–500ms)
- Architektura změny (patch per-site vs. centrální wrapper)
- Jak detekovat "aktivní psaní" vs. idle (poslední keystroke time delta)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `config::REPAINT_INTERVAL_MS` — konstanta existuje, použita v workspace/mod.rs pro podmíněný repaint_after; rozšíř nebo doplň podobné konstanty
- `request_repaint_after(Duration)` vzor — workspace/mod.rs:117 ukazuje správný vzor, který je třeba rozšířit
- `ctx.input(|i| i.viewport().focused)` — eframe API pro detekci focus stavu

### Established Patterns
- Podmíněný repaint_after: `if has_active_work { ctx.request_repaint_after(...) }` — vzor v workspace/mod.rs je správný; aplikovat analogicky na focus/minimize
- 32 bezpodmínečných `request_repaint()` ve vláknech na pozadí (plugins/host/fs.rs, sys.rs, background.rs, ai_chat/render.rs, dialogs/) — toto jsou hlavní kandidáti na throttling

### Integration Points
- `src/app/mod.rs:398` — `ctx.request_repaint()` v hlavní update smyčce (kritické)
- `src/app/ui/workspace/mod.rs:117` — existující podmíněný repaint_after (vzor pro rozšíření)
- `src/app/registry/plugins/host/` — fs.rs (145, 285), mod.rs (44), sys.rs (278, 342, 379, 408, 445) — vlákna volají přímý repaint
- `src/app/ui/terminal/ai_chat/` — render.rs (40), approval.rs (83, 139, 189) — AI panel repaints
- `src/app/ui/background.rs` — 6× request_repaint (140, 169, 174, 286, 310, 315)
- `src/app/ui/workspace/state/init.rs` — 4× request_repaint (319, 338, 374, 462)

</code_context>

<specifics>
## Specific Ideas

- Success criterion 1: CPU < 1–2 % po 10 s v idle (bez myší/klávesnice)
- Success criterion 2: Minimalizované okno = nula nebo max 1× za 5 s
- Success criterion 3: Accesskit odstraněn z Cargo.toml
- Success criterion 4: FPS cap při psaní (ne bezpodmínečný repaint po každém keystroke)

</specifics>

<deferred>
## Deferred Ideas

Žádné — diskuze zůstala v rozsahu fáze.

</deferred>

---

*Phase: 01-repaint-gate*
*Context gathered: 2026-03-04*
