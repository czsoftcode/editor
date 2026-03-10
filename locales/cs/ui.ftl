# Obecné prvky uživatelského rozhraní

## Panely
panel-files = Soubory
btn-tree-project = Projekt
panel-runners = Spouštěče
panel-build = Sestavit
panel-git = Git
panel-build-errors =
    { $count ->
        [one] Chyba (1)
        [few] Chyby ({ $count })
       *[other] Chyb ({ $count })
    }

## Build tlačítka
btn-build = ▶ Sestavit
btn-run = ▶ Spustit
btn-run-new = ▶ Spustit+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Vytvořit .deb
hover-create-deb = Sestavit a vytvořit vývojový .deb balíček s číslem sestavení (build)
btn-run-profile = ▶ Spustit...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Upravit
runner-none = Nejsou definovány žádné profily.

## Git operace
git-add-all = git add .
git-commit = git commit -m "..."
git-push = git push
git-status = git status
git-diff = git diff
git-checkout-file = git checkout (soubor)
git-checkout-branch = git checkout (větev)
git-pull = git pull
git-reset-hard = git reset --hard

## Status bar
statusbar-line-col = Řádek { $line }, Sloupec { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Neuloženo
statusbar-saving = Ukládání…
statusbar-saved = Uloženo
statusbar-lsp-initializing = LSP se inicializuje...
statusbar-filetype-plain = Prostý text
statusbar-save-mode-automatic = Auto ukládání
statusbar-save-mode-manual = Ruční ukládání

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
btn-cancel = Storno
btn-close = Zavřít
cancel-confirm-title = Zahodit změny?
cancel-confirm-msg = Opravdu si přejete zahodit všechny neuložené změny a zavřít toto okno?
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
ai-diff-heading = Kontrola změn navržených AI
ai-diff-new-file = Nový soubor navržen
ai-float-dock = Přikovat do panelu
ai-float-undock = Odpojit do plovoucího okna
ai-viewport-open = Otevřít v samostatném okně
ai-tab-close-hover = Zavřít záložku
ai-tab-new-hover = Nová záložka terminálu
ai-staged-bar-msg = AI navrhlo změny v projektu
ai-staged-bar-review = Zkontrolovat změny
ai-staged-bar-promote-all = Přenést vše
ai-staged-modal-hint = Klikněte na soubor pro zobrazení rozdílů a schválení změn:
ai-staged-files = Navržené změny
ai-staged-new = [NOVÝ]
ai-staged-mod = [MOD]
ai-staged-del = [SMAZÁNO]
ai-promotion-success-title = Změny aplikovány
ai-promotion-success-body = Následující soubor byl úspěšně aktualizován ve vašem projektu:
ai-promotion-success = Změny byly úspěšně aplikovány do projektu.
ai-promotion-all-success = Úspěšně přeneseno { $count } souborů do projektu.
ai-promotion-failed = Nepodařilo se aplikovat změny: { $error }

## Synchronizace před spuštěním AI

## Oprávnění pluginů

## Nastavení
settings-title = Nastavení
settings-category-general = Obecné
settings-category-editor = Editor
settings-category-ai = AI Agenti
settings-language = Jazyk
settings-language-restart = Jazyk se změní okamžitě.
settings-theme = Téma
settings-theme-dark = Tmavé
settings-theme-light = Světlé
settings-light-variant = Světlá varianta
settings-light-variant-warm-ivory = Teplá slonová kost
settings-light-variant-cool-gray = Studená šedá
settings-light-variant-sepia = Sépie
settings-save-mode-title = Režim ukládání
settings-save-mode-automatic = Automatické ukládání
settings-save-mode-manual = Ruční ukládání
settings-save-mode-toast-automatic = Automatické ukládání zapnuto
settings-save-mode-toast-manual = Ruční ukládání zapnuto
settings-auto-show-diff = Automaticky otevírat náhled změn AI
settings-conflict-title = Nastavení změněno
settings-conflict-message = Nastavení bylo upraveno v jiném okně. Načtěte nejnovější hodnoty, nebo pokračujte v aktuálním návrhu.
settings-conflict-reload = Načíst
settings-conflict-keep = Ponechat
settings-diff-mode = Zobrazení AI Diffu
settings-diff-inline = Sloučené (+ / -)
settings-diff-side-by-side = Vedle sebe
settings-editor-font = Editor — velikost fontu
settings-ai-font = AI terminál — velikost fontu
settings-default-path = Výchozí cesta projektů
settings-ai-name = Název asistenta
settings-ai-command = Příkaz (binárka)
settings-ai-args = Parametry (volitelné)
settings-ai-add = Přidat agenta
settings-ai-hint = Zde si můžete nadefinovat vlastní CLI nástroje (např. gemini, claude, aider). Pokud seznam necháte prázdný, použijí se výchozí.
settings-creates-in = Vytvoří se v:
settings-blacklist = Blacklist (zakázané soubory pro pluginy)
settings-blacklist-hint = Podporuje vzory jako *.env, secret/* nebo konkrétní názvy souborů. Automaticky zakazuje soubory v .gitignore.
settings-blacklist-add = Přidat vzor

## Pluginy

## Command Palette – AI pluginy

## Sémantická indexace (RAG)
semantic-indexing-title = Sémantická indexace projektu
semantic-indexing-init = Inicializace ML modelu (stahování)...
semantic-indexing-processing = Zpracování: { $processed } / { $total }
semantic-indexing-btn-bg = Spustit na pozadí
semantic-indexing-status-bar = Indexace projektu...

## Nastavení
settings-suggested-patterns = Doporučené vzory:

## Chyba pluginu

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
conflict-message = Soubor „{ $name }" byl změněn mimo editor, ale obsahuje neuložené změny v editoru.
conflict-choose = Vyberte, kterou verzi chcete zachovat:
conflict-load-disk = Načíst z disku
conflict-keep-editor = Zachovat verzi editoru
conflict-dismiss = Zrušit
conflict-hover-disk = Zahodit neuložené změny v editoru a načíst verzi změněnou na disku
conflict-hover-keep = Ponechat rozpracované změny v editoru; verze na disku bude přepsána při vašem příštím uložení (Ctrl+S)
conflict-hover-dismiss = Zavřít upozornění bez provedení změn

md-open-external = ↗ Otevřít v externím prohlížeči
md-layout-pod-sebou = Pod sebou
md-layout-vedle-sebe = Vedle sebe
md-layout-jenom-kod = Jenom kód
md-layout-jenom-nahled = Jenom náhled

svg-open-external = ↗ Otevřít náhled v prohlížeči

svg-modal-title = SVG soubor
svg-modal-body = Tento soubor je SVG obrázek. Chcete ho otevřít v systémovém prohlížeči, nebo upravovat jako XML text?
svg-modal-edit = Upravovat jako text
## Support Modal
support-modal-title = Podpořit vývoj PolyCredo
support-modal-body = PolyCredo Editor je vyvíjen s vizí soukromí, rychlosti a bezpečné integrace AI asistentů. Pokud se vám projekt líbí, budeme vděční za jakoukoli podporu. Vaše příspěvky nám pomáhají věnovat více času vývoji nových funkcí a údržbě.
support-modal-github = Sledovat na GitHubu
support-modal-donate = Přispět na rozvoj
semantic-indexing-btn-stop = Zastavit indexaci

# Dependency Wizard
dep-wizard-title = Instalace chybějících nástrojů
dep-wizard-appimagetool-desc = Pro vytvoření AppImage balíčku je vyžadován nástroj { $tool }. Tento nástroj není součástí standardních repozitářů vašeho systému.
dep-wizard-install-question = Chcete jej nyní stáhnout z oficiálního zdroje a nainstalovat do { $path }?
dep-wizard-btn-install = Stáhnout a nainstalovat
dep-wizard-status-downloading = Stahování...
dep-wizard-status-success = Nástroj byl úspěšně nainstalován.
dep-wizard-status-error = Chyba při instalaci: { $error }

dep-wizard-install-cmd-question = Tento nástroj lze nainstalovat pomocí systémového správce balíčků. Chcete spustit instalaci?
dep-wizard-btn-run-cmd = Spustit instalaci (vyžaduje sudo)
dep-wizard-status-running = Instaluji...
btn-create-deb = Sestavit .deb
hover-create-deb = Sestaví vývojový balíček .deb pro aktuální architekturu.

## Strážce neuložených změn
unsaved-close-guard-title = Neuložené změny
unsaved-close-guard-message = Tento soubor má neuložené změny. Co chceš udělat před zavřením?
unsaved-close-guard-save = Uložit a zavřít
unsaved-close-guard-discard = Zahodit změny
unsaved-close-guard-cancel = Zrušit
