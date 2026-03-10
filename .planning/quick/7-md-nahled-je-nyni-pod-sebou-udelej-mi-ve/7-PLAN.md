# Quick Plan: Markdown náhled režimy zobrazení

## Cíl
Přidat cyklické toggle tlačítko vedle akce „Otevřít v externím prohlížeči“ pro markdown preview s režimy:
`Pod sebou | Vedle sebe | Jenom kód | Jenom náhled`.

## Tasky

1. Zavést stav režimu markdown layoutu
- Přidat enum pro 4 režimy (`PodSebou`, `VedleSebe`, `JenomKod`, `JenomNahled`) do existujícího workspace/editor stavu.
- Nastavit výchozí hodnotu na `Pod sebou` (aktuální chování).
- Změna je izolovaná na datový model stavu bez zásahu do renderu.

2. Přidat cyklické toggle tlačítko do toolbaru markdown preview
- Vedle „Otevřít v externím prohlížeči“ vložit tlačítko, které po kliknutí přepíná režimy v pořadí:
  `Pod sebou -> Vedle sebe -> Jenom kód -> Jenom náhled -> Pod sebou`.
- Label tlačítka vždy zobrazuje aktuální režim.
- Akce je atomická: pouze update stavu + UI event handler.

3. Upravit render markdown editoru podle režimu
- `Pod sebou`: kód nahoře, náhled dole.
- `Vedle sebe`: kód vlevo, náhled vpravo.
- `Jenom kód`: renderovat jen editor.
- `Jenom náhled`: renderovat jen preview.
- Zachovat existující logiku „Otevřít v externím prohlížeči“ beze změny.
