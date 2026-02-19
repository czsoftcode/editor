# Obecné prvky uživatelského rozhraní

## Panely
panel-files = Soubory
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
btn-test = ▶ Test
btn-clean = ✖ Clean

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

## Terminál
terminal-unavailable = Terminál není dostupný.
terminal-retry = Zkusit znovu
terminal-exited = [Proces skončil — stiskněte R pro restart]

## Dialog „Přejít na řádek" (Ctrl+G)
goto-line-prompt = Přejít na řádek:
goto-line-placeholder = číslo řádku

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
ai-float-dock = Přikovat do panelu
ai-float-undock = Odpojit do plovoucího okna
ai-tab-close-hover = Zavřít záložku
ai-tab-new-hover = Nová záložka terminálu

## Nastavení
settings-title = Nastavení
settings-language = Jazyk
settings-language-restart = Jazyk se změní okamžitě.
settings-theme = Téma
settings-theme-dark = Tmavé
settings-theme-light = Světlé
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
