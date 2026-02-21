# Obecné prvky uživatelského rozhraní

## Panely
panel-files = Soubory
panel-runners = Spouštěče
panel-build = Build
panel-build-errors =
    { $count ->
        [one] Chyba (1)
        [few] Chyby ({ $count })
       *[other] Chyb ({ $count })
    }

## Build tlačítka
btn-build = ▶ Build
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Vytvořit .deb
btn-run-profile = ▶ Spustit...
btn-edit-profiles = ⚙ Upravit
runner-none = Nejsou definovány žádné profily.

## Status bar
statusbar-line-col = Řádek { $line }, Sloupec { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Neuloženo
statusbar-saving = Ukládání…
statusbar-saved = Uloženo
statusbar-filetype-plain = Prostý text

## Záložky editoru
tab-unsaved-indicator = ●
tab-deleted-indicator = ⚠

## Hledání a nahrazování
search-label = Hledat:
replace-label = Nahradit:
search-replace-expand = Nahradit…
search-placeholder = Hledat…
replace-placeholder = Nahradit…
search-prev = ▲
search-next = ▼
search-replace-one = Nahradit
search-replace-all = Nahradit vše
search-results =
    { $count ->
        [one] 1 výsledek
        [few] { $count } výsledky
       *[other] { $count } výsledků
    }
search-no-results = Nenalezeno

## Editor
editor-empty-hint = Otevřete soubor z adresářového stromu vlevo
editor-preview-label = Náhled

# LSP / rust-analyzer
lsp-missing-title = Chybí rust-analyzer
lsp-missing-msg = Pro chytré funkce (doplňování, chyby) je potřeba rust-analyzer. Chcete jej nainstalovat?
lsp-install-btn = Nainstalovat
lsp-installing = Instaluji rust-analyzer...
lsp-install-success = rust-analyzer byl úspěšně nainstalován. Restartuji LSP...
lsp-install-error = Instalace selhala: { $error }

## Find References (Shift+F12)
lsp-references-heading = Reference
lsp-references-searching = Hledám reference...
lsp-references-none = Nenalezeny žádné reference.
lsp-references-found =
    { $count ->
        [one] Nalezena 1 reference.
        [few] Nalezeny { $count } reference.
       *[other] Nalezeno { $count } referencí.
    }
lsp-references-error = LSP: Chyba při hledání referencí.

## Terminál
terminal-unavailable = Terminál není dostupný.
terminal-retry = Zkusit znovu
terminal-exited = [Proces skončil — stiskněte R pro restart]
terminal-close-confirm-title = Zavřít terminál?
terminal-close-confirm-msg = V terminálu stále běží proces. Opravdu jej chcete ukončit?

## Dialog „Přejít na řádek" (Ctrl+G)
goto-line-prompt = Přejít na řádek:
goto-line-placeholder = číslo řádku

## Command Palette (Ctrl+Shift+P)
command-palette-heading = Příkazy
command-palette-placeholder = Hledat příkaz…
command-palette-no-results = Žádné výsledky

command-name-open-file = Otevřít soubor
command-name-project-search = Hledat v projektu
command-name-build = Sestavit (Build)
command-name-run = Spustit (Run)
command-name-save = Uložit aktuální soubor
command-name-close-tab = Zavřít aktuální záložku
command-name-new-project = Nový projekt
command-name-open-project = Otevřít projekt (v novém okně)
command-name-open-folder = Otevřít složku (v tomto okně)
command-name-toggle-left = Přepnout panel souborů
command-name-toggle-right = Přepnout AI panel
command-name-toggle-build = Přepnout build terminál
command-name-toggle-float = Přepnout plovoucí AI panel
command-name-show-about = O aplikaci
command-name-show-settings = Nastavení
command-name-quit = Ukončit PolyCredo Editor

## Rychlé otevření souboru (Ctrl+P)
file-picker-heading = Otevřít soubor
file-picker-placeholder = Rychlé otevření souboru…
file-picker-no-results = Žádné výsledky
file-picker-count = { $count } souborů
file-picker-count-filtered = { $filtered }/{ $total } souborů
file-picker-more = … a { $count } dalších

## Hledání napříč projektem (Ctrl+Shift+F)
project-search-heading = Hledat v projektu
project-search-placeholder = Hledat v projektu…
project-search-hint = Hledaný výraz…
project-search-btn = Hledat
project-search-loading = Hledání…
project-search-result-label = Výsledky hledání „{ $query }" ({ $count })
project-search-results =
    { $count ->
        [one] 1 výsledek
       *[other] { $count } výsledků
    }
project-search-no-results = Žádné výsledky
project-search-max-results = Zobrazeno max. { $max } výsledků

## Obecná tlačítka
btn-ok = OK
btn-confirm = Potvrdit
btn-cancel = Zrušit
btn-close = Zavřít
btn-browse = Procházet…
btn-create = Vytvořit
btn-open = Otevřít
btn-refresh = Obnovit
btn-save = Uložit
btn-rename = Přejmenovat
btn-copy = Kopírovat
btn-paste = Vložit
btn-delete = Smazat
btn-name-label = Název:

## AI panel
ai-panel-title = AI terminál
ai-tool-not-found = Nástroj { $tool } nebyl nalezen v PATH.
ai-tool-detecting = Detekuji AI nástroje…
ai-label-assistant = Asistent:
ai-tool-status-checking = { $tool } (ověřuji…)
ai-tool-status-available = { $tool } (nainstalováno)
ai-tool-status-missing = { $tool } (není v PATH)
ai-hover-reverify = Znovu ověřit dostupnost AI CLI nástrojů
ai-hover-checking = Ověřuji dostupnost AI CLI nástrojů…
ai-hover-start = Spustí { $tool } (`{ $cmd }`) v terminálu
ai-hover-missing = Příkaz `{ $cmd }` nebyl nalezen v PATH. Nainstaluj nástroj a klikni na ↻.
ai-btn-start = ▶ Spustit
ai-btn-sync = ⟳ Sync
ai-hover-sync = Odeslat kontext (otevřené soubory, chyby buildu) AI agentovi
ai-diff-heading = Kontrola změn navržených AI
ai-diff-new-file = Nový soubor navržen
ai-float-dock = Přikovat do panelu
ai-float-undock = Odpojit do plovoucího okna
ai-viewport-open = Otevřít v samostatném okně
ai-tab-close-hover = Zavřít záložku
ai-tab-new-hover = Nová záložka terminálu
ai-staged-bar-msg = AI navrhlo změny v projektu
ai-staged-bar-review = Zkontrolovat změny
ai-staged-files = Navržené změny (Sandbox)
ai-staged-new = [NOVÝ]
ai-staged-mod = [MOD]
ai-promotion-success-title = Změny aplikovány
ai-promotion-success-body = Následující soubor byl úspěšně aktualizován ve vašem projektu:
ai-promotion-success = Změny byly úspěšně aplikovány do projektu.
ai-promotion-failed = Nepodařilo se aplikovat změny: { $error }

## Nastavení
settings-title = Nastavení
settings-language = Jazyk
settings-language-restart = Jazyk se změní okamžitě.
settings-theme = Téma
settings-theme-dark = Tmavé
settings-theme-light = Světlé
settings-diff-mode = Zobrazení AI Diffu
settings-diff-inline = Sloučené (+ / -)
settings-diff-side-by-side = Vedle sebe
settings-editor-font = Editor — velikost fontu
settings-ai-font = AI terminál — velikost fontu
settings-default-path = Výchozí cesta projektů
settings-creates-in = Vytvoří se v:

## Soubory
file-tree-new-file = Nový soubor
file-tree-new-dir = Nový adresář
file-tree-rename = Přejmenovat
file-tree-copy = Kopírovat
file-tree-paste = Vložit
file-tree-delete = Smazat
file-tree-confirm-delete = Smazat { $name }?
file-tree-unsafe-name = Neplatný název: nesmí obsahovat /, \ ani ..
file-tree-outside-project = Cesta by vedla mimo projekt
file-tree-paste-error = Nelze vložit: { $reason }
file-tree-create-dir-error = Nelze vytvořit adresář: { $reason }
file-tree-create-file-error = Nelze vytvořit soubor: { $reason }
file-tree-rename-error = Nelze přejmenovat: { $reason }
file-tree-delete-error = Nelze smazat: { $reason }

## Dialog externího konfliktu
conflict-title = Soubor změněn externě
conflict-message = Soubor „{ $name }" byl změněn jiným programem, ale obsahuje neuložené změny v editoru.
conflict-choose = Vyberte, která verze má vyhrát:
conflict-load-disk = Načíst z disku
conflict-keep-editor = Zachovat moje
conflict-dismiss = Ignorovat
conflict-hover-disk = Zahodit změny v editoru a načíst verzi uloženou na disku
conflict-hover-keep = Ponechat změny v editoru; soubor na disku bude přepsán při uložení
conflict-hover-dismiss = Zavřít upozornění bez změny

md-open-external = ↗ Otevřít v externím prohlížeči

svg-open-external = ↗ Otevřít náhled v prohlížeči

svg-modal-title = SVG soubor
svg-modal-body = Tento soubor je SVG obrázek. Chcete ho otevřít v systémovém prohlížeči, nebo upravovat jako XML text?
svg-modal-edit = Upravovat jako text
