# GSD Queue

## 2026-03-13: M002 — Local History

Zprovoznit local history: snapshot při uložení, split view s diff a navigací mezi verzemi, retention policy 50 verzí + 30 dní, smazat stará testovací data.

## 2026-03-13: M004 — Klávesové Zkratky a Centrální Keymap

Oprava rozbitého modifier filtrování (trojkombinace spouští dvoukombinace), vybudování centrálního keymap dispatch napojeného na command registry, implementace chybějících keyboard handlerů (Ctrl+F/H/G/P, Ctrl+Shift+F/P), sjednocení s VS Code/JetBrains konvencemi, uživatelská konfigurace přes [keybindings] v settings.toml, cross-platform Linux+macOS (Ctrl↔Cmd).

## 2026-03-13: M005 — Vylepšení Project Search

Přebudování project-wide search (Ctrl+Shift+F) z minimálního modálního dialogu na plnohodnotný vyhledávací nástroj — regex, case-sensitive/whole-word togglery, zvýraznění matchů ve výsledcích, kontextové řádky, filtrování dle typu souboru, a project-wide find & replace s preview a potvrzením per-soubor.

## 2026-03-13: M006 — Inline Search Panel + Vylepšení In-file Search

Přesun project search z modálních dialogů do inline spodního panelu pod editorem (VS Code styl) — persistentní stav, fokus management po kliknutí na výsledek, resize panelu. Sjednocení in-file search (Ctrl+F) s regex/case/whole-word engine z M005 přes sdílený build_regex().
