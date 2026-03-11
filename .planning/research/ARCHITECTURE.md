# Milestone v1.3.0 Research - Architecture

## Current coupling
- `src/app/ui/*` a `src/settings.rs` importuji `crate::app::cli::*` typy.
- AI terminal logika (`src/app/ui/terminal/ai_chat/*`) pouziva provider/tools/executor casti z `app::cli`.

## Target shape
- AI terminal zustane v `src/app/ui/terminal/ai_chat/*`.
- Potrebne runtime casti se presunou pod terminal-focused namespace (napr. `ui/terminal/ai_core` nebo ekvivalent).
- Workspace state a settings budou odkazovat na novy namespace, ne na `app::cli`.

## Suggested build order
1. Vytvorit novy namespace/moduly pro AI terminal core a prepojit importy.
2. Migrovat provider/executor/types pouzite AI terminalem.
3. Odebrat `src/app/cli/*` a dojet cleanup testu/docs.
