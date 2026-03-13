# T04: 03-light-varianty-settings-ui 04

**Slice:** S05 — **Milestone:** M001

## Description

Doplnit lock na jemne prizpusobeni terminalu a git barev podle zvolene light varianty, bez rozsirovani scope mimo existujici render pipeline.

Purpose: Dokoncit light-variant polish tak, aby se tonalita light variant propsala i do casti dorucenych ve Phase 2.
Output: Variant-aware tone adaptace v terminal a file tree git barvach s regresnimi testy citelnosti.

## Must-Haves

- [ ] "V light mode nejsou terminal ani git barvy staticke; jemne se prizpusobuji aktivni light variante pres tonalitu z `ui.visuals()` (WarmIvory/CoolGray/Sepia)."
- [ ] "Prizpusobeni je zamerne lehke: statusy i ANSI barvy zustavaji citelne a semanticky odlisitelne, pouze se jemne posune ton."
- [ ] "Dark mode beha beze zmen (zadny regres do hotoveho TERM/TREE chovani z Phase 2)."
- [ ] "Variant-aware chovani je testovatelne: dve ruzne light varianty produkuji odlisne vystupy pro terminal/git barvy."

## Files

- `src/app/ui/terminal/instance/theme.rs`
- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/git_status.rs`
- `src/app/ui/file_tree/render.rs`
