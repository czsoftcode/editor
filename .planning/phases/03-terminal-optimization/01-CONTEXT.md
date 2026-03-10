# Phase 3: Terminal Optimization - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Optimalizace výkonu a robustnosti terminálu:
- Detekce cest (regex) a její dopad na CPU při pohybu myši.
- Správa životního cyklu PTY (cleanup process group).
- Vizuální indikace změn v neaktivních tabech.

</domain>

<decisions>
## Implementation Decisions

### Path Detection (RPNT-05)
- Regex na cesty spouštět pouze nad řádkem, na kterém je myš, ne nad celou obrazovkou (pokud to tak již není).
- Cacheovat výsledky parsování pro jednotlivé řádky mřížky.

### PTY Robustness
- Při zavření tabu nebo `drop` terminálu poslat SIGTERM celé process group (nejen hlavnímu shellu), aby se ukončily i běžící procesy (např. `cargo watch`).

### Activity Indicator
- Přidat `has_unread_output: bool` do `Terminal` a resetovat jej při focusu.

</decisions>

<code_context>
## Existing Code Insights

### Path Parsing
- `src/app/ui/terminal/instance/mod.rs:207` — `path_regex` se používá k detekci cest.
- Aktuálně se parsuje pouze řádek pod kurzorem (`grid_line_idx`), což je dobré, ale regex se kompiluje v `new` a mohl by být efektivnější.

### Process Cleanup
- `src/app/ui/terminal/instance/mod.rs:434` — `kill_process_group` už existuje, ale volá se jen v `drop`. Je třeba zajistit jeho volání i při ručním zavření tabu.

</code_context>

<specifics>
## Specific Ideas

- Success criterion 1: Žádné zombie procesy po zavření editoru (ověřeno přes `ps aux | grep cargo`).
- Success criterion 2: Vizuální indikace (tečka) u tabu terminálu, pokud v něm na pozadí přibyl text.

</specifics>
