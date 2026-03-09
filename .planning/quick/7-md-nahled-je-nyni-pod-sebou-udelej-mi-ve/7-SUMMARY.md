# Quick Task 7 Summary

## Cíl
Implementovat cyklické přepínání režimu markdown náhledu a renderovat split podle režimu:
`Pod sebou | Vedle sebe | Jenom kód | Jenom náhled`.

## Co bylo provedeno

### Task 1: Stav layout režimu
- Přidán enum `MarkdownLayoutMode` do editor stavu.
- Přidána položka `md_layout_mode` do `Editor` se defaultem `MarkdownLayoutMode::PodSebou`.

### Task 2: Cyklický toggle v toolbaru
- Vedle tlačítka `md-open-external` přidáno toggle tlačítko.
- Toggle cyklí pořadí přesně dle plánu:
  `PodSebou -> VedleSebe -> JenomKod -> JenomNahled -> PodSebou`.
- Přidány i18n klíče pro label režimu do všech jazykových souborů (`cs`, `en`, `de`, `ru`, `sk`).

### Task 3: Render splitu podle režimu
- `Pod sebou`: editor nahoře, preview dole (s vertikálním resize handle).
- `Vedle sebe`: editor vlevo, preview vpravo (s horizontálním resize handle).
- `Jenom kód`: renderuje se pouze editor.
- `Jenom náhled`: renderuje se pouze preview.
- Zachována existující logika tlačítka `md-open-external`.

## Validace
- `cargo check`: **OK**
- `./check.sh`: **FAIL (unrelated)**
  - Fail je na `cargo fmt --check` kvůli již existujícím formátovacím rozdílům v mnoha souborech mimo scope této quick task změny.
  - Nejde o regresi v této implementaci markdown layout režimů.

## Commity
1. `8d9faa7` — `feat(markdown): add layout mode state enum`
2. `25c4aff` — `feat(markdown): add layout mode toggle button`
3. `ff30f03` — `feat(markdown): render preview layouts by selected mode`
