# Report opravy: Nekonečné notifikace v Safe Mode

## Problém
Při editaci souboru v **Safe Mode** (read-only) mimo sandbox docházelo k efektu "vodopádu" červených chybových notifikací.
Příčinou byla smyčka **automatického ukládání (autosave)**:
1. Uživatel provedl změnu -> soubor byl označen jako `modified`.
2. Po 500 ms se spustil `try_autosave`.
3. Pokus o uložení selhal kvůli Safe Mode omezení a vrátil chybu.
4. Editor zobrazil notifikaci.
5. Protože uložení selhalo, soubor zůstal ve stavu `modified`.
6. V dalším cyklu se autosave pokusil uložit znovu, což vedlo k nekonečnému proudu notifikací.

## Řešení
Upravil jsem funkci `try_autosave` v souboru `src/app/ui/editor/files.rs`.

```rust
pub fn try_autosave(..., read_only: bool) -> Option<String> {
    // ...
    if should_save {
        // NOVÉ: Pokud jsme v Safe Mode a mimo sandbox, autosave tiše ignorujeme.
        if read_only {
            if let Some(tab) = self.active() {
                let path_str = tab.path.to_string_lossy();
                if !path_str.contains(".polycredo/sandbox") {
                    return None; // Žádná chyba, žádná akce
                }
            }
        }
        self.save(...)
    }
}
```

## Výsledek
- **Autosave**: V Safe Mode tiše selže (neprovede se), uživatel není rušen. Indikátor změny (tečka u tabu) zůstává aktivní.
- **Manuální uložení (Ctrl+S)**: Stále vyvolá chybovou hlášku ("Cannot edit in safe mode"), což je správné chování pro explicitní akci uživatele.
- **Efekt vodopádu**: Odstraněn.
