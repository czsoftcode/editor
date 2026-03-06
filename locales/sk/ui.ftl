# Všeobecné prvky používateľského rozhrania

## Panely
panel-files = Súbory
btn-tree-project = Projekt
panel-runners = Spúšťače
panel-build = Zostavenie
panel-git = Git
panel-build-errors =
    { $count ->
        [one] Chyba (1)
        [few] Chyby ({ $count })
       *[other] Chýb ({ $count })
    }

## Tlačidlá buildu
btn-build = ▶ Build
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Vytvoriť .deb
hover-create-deb = Zostaviť a vytvoriť vývojový .deb balíček s číslom zostavenia (build)
btn-run-profile = ▶ Spustiť...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Upraviť
runner-none = Nie sú definované žiadne profily.
## Dependency Wizard
dep-wizard-title = Sprievodca inštaláciou závislostí
dep-wizard-install-question = Chcete stiahnuť a nainštalovať { $tool } do { $path }?
dep-wizard-install-cmd-question = Chcete spustiť inštaláciu { $tool } pomocou systémového príkazu?
dep-wizard-btn-install = Inštalovať
dep-wizard-btn-run-cmd = Spustiť inštaláciu (vyžaduje sudo)
dep-wizard-status-downloading = Sťahujem...
dep-wizard-status-running = Inštalujem...
dep-wizard-status-success = Inštalácia bola úspešná!
dep-wizard-status-error = Chyba inštalácie: { $error }

## Git operácie
git-add-all = git add .
git-commit = git commit -m "..."
git-push = git push
git-status = git status
git-diff = git diff
git-checkout-file = git checkout (súbor)
git-checkout-branch = git checkout (vetva)
git-pull = git pull
git-reset-hard = git reset --hard

## Status bar
statusbar-line-col = Riadok { $line }, Stĺpec { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Neuložené
statusbar-saving = Ukladám…
statusbar-saved = Uložené
statusbar-lsp-initializing = LSP sa inicializuje...
statusbar-filetype-plain = Obyčajný text

## Záložky editora
tab-unsaved-indicator = ●
tab-deleted-indicator = ⚠

## Hľadanie a nahrádzanie
search-label = Hľadať:
replace-label = Nahradiť:
search-replace-expand = Nahradiť…
search-placeholder = Hľadať…
replace-placeholder = Nahradiť…
search-prev = ▲
search-next = ▼
search-replace-one = Nahradiť
search-replace-all = Nahradiť všetko
search-results =
    { $count ->
        [one] 1 výsledok
        [few] { $count } výsledky
       *[other] { $count } výsledkov
    }
search-no-results = Žiadne zhody

## Editor
editor-empty-hint = Otvorte súbor zo stromu súborov vľavo
editor-preview-label = Náhľad

# LSP / rust-analyzer
lsp-missing-title = Chýba rust-analyzer
lsp-missing-msg = Pre chytré funkcie (doplňovanie, chyby) je potrebný rust-analyzer. Chcete ho nainštalovať?
lsp-install-btn = Inštalovať
lsp-installing = Inštalujem rust-analyzer...
lsp-install-success = rust-analyzer bol úspešne nainštalovaný. Reštartujem LSP...
lsp-install-error = Inštalácia zlyhala: { $error }

## Terminál
terminal-unavailable = Terminál nie je dostupný.
terminal-retry = Skúsiť znova
terminal-exited = [Proces skončil — stlačte R pre reštart]
terminal-close-confirm-title = Zavrieť terminál?
terminal-close-confirm-msg = V termináli stále beží proces. Naozaj ho chcete ukončiť?

## Dialóg Prejsť na riadok (Ctrl+G)
goto-line-prompt = Prejsť na riadok:
goto-line-placeholder = číslo riadku

## Command Palette (Ctrl+Shift+P)
command-palette-heading = Príkazy
command-palette-placeholder = Hľadať príkaz…
command-palette-no-results = Žiadne výsledky

command-name-open-file = Otvoriť súbor
command-name-project-search = Hľadať v projekte
command-name-build = Zostaviť
command-name-run = Spustiť
command-name-save = Uložiť aktuálny súbor
command-name-close-tab = Zatvoriť aktuálnu záložku
command-name-new-project = Nový projekt
command-name-open-project = Otvoriť projekt (v novom okne)
command-name-open-folder = Otvoriť priečinok (v tomto okne)
command-name-toggle-left = Prepnúť panel súborov
command-name-toggle-right = Prepnúť AI panel
command-name-toggle-build = Prepnúť build terminál
command-name-toggle-float = Prepnúť plávajúci AI panel
command-name-show-about = O aplikácii
command-name-show-settings = Nastavenia
command-name-quit = Ukončiť PolyCredo Editor

## Rýchle otvorenie súboru (Ctrl+P)
file-picker-heading = Otvoriť súbor
file-picker-placeholder = Rýchle otvorenie súboru…
file-picker-no-results = Žiadne výsledky
file-picker-count = { $count } súborov
file-picker-count-filtered = { $filtered }/{ $total } súborov
file-picker-more = … a { $count } ďalších

## Hľadanie v projekte (Ctrl+Shift+F)
project-search-heading = Hľadať v projekte
project-search-placeholder = Hľadať v projekte…
project-search-hint = Hľadaný výraz…
project-search-btn = Hľadať
project-search-loading = Hľadám…
project-search-result-label = Výsledky pre „{ $query }" ({ $count })
project-search-results =
    { $count ->
        [one] 1 výsledok
        [few] { $count } výsledky
       *[other] { $count } výsledkov
    }
project-search-no-results = Žiadne výsledky
project-search-max-results = Zobrazených max. { $max } výsledkov

## Spoločné tlačidlá
btn-ok = OK
btn-confirm = Potvrdiť
btn-cancel = Storno
btn-close = Zavrieť
cancel-confirm-title = Zahodiť zmeny?
cancel-confirm-msg = Naozaj si prajete zahodiť všetky neuložené zmeny a zavrieť toto okno?

btn-browse = Prehľadávať…
btn-create = Vytvoriť
btn-open = Otvoriť
btn-refresh = Obnoviť
btn-save = Uložiť
btn-rename = Premenovať
btn-copy = Kopírovať
btn-paste = Vložiť
btn-delete = Odstrániť
btn-name-label = Názov:

## AI panel
ai-panel-title = AI Terminál
ai-tool-not-found = Nástroj { $tool } nebol nájdený v PATH.
ai-tool-detecting = Zisťujem AI nástroje…
ai-label-assistant = Asistent:
ai-tool-status-checking = { $tool } (zisťujem…)
ai-tool-status-available = { $tool } (nainštalovaný)
ai-tool-status-missing = { $tool } (nie je v PATH)
ai-hover-reverify = Znovu overiť dostupnosť AI CLI nástrojov
ai-hover-checking = Zisťujem dostupnosť AI CLI nástrojov…
ai-hover-start = Spustí { $tool } (`{ $cmd }`) v termináli
ai-hover-missing = Príkaz `{ $cmd }` nebol nájdený v PATH. Nainštalujte nástroj a kliknite ↻.
ai-btn-start = ▶ Spustiť
ai-float-dock = Umiestniť do panela
ai-float-undock = Odpojiť do plávajúceho okna
ai-viewport-open = Otvoriť v samostatnom okne
ai-tab-close-hover = Zatvoriť záložku
ai-tab-new-hover = Nová záložka terminálu
ai-staged-bar-msg = AI navrhlo zmeny v projekte
ai-staged-bar-review = Skontrolovať zmeny
ai-staged-bar-promote-all = Preniesť všetko
ai-staged-modal-hint = Kliknite na súbor pre zobrazenie rozdielov a schválenie zmien:
ai-staged-files = Navrhnuté zmeny
ai-staged-new = [NOVÝ]
ai-staged-mod = [MOD]
ai-staged-del = [ZMAZANÝ]
ai-promotion-success-title = Zmeny aplikované
ai-promotion-success-body = Nasledujúci súbor bol úspešne aktualizovaný vo vašom projekte:
ai-promotion-success = Zmeny boli úspešne aplikované do projektu.
ai-promotion-all-success = Úspešne prenesených { $count } súborov do projektu.
ai-promotion-failed = Nepodarilo sa aplikovať zmeny: { $error }


## Synchronizácia pred spustením AI

## Oprávnenia pluginov

## Nastavenia
settings-title = Nastavenia
settings-category-general = Všeobecné
settings-category-editor = Editor
settings-category-ai = AI Agenti
settings-language = Jazyk
settings-language-restart = Zmena jazyka sa prejaví okamžite.
settings-theme = Téma
settings-theme-dark = Tmavá
settings-theme-light = Svetlá
settings-light-variant = Svetlá varianta
settings-light-variant-warm-ivory = Teplá slonovinová
settings-light-variant-cool-gray = Studená sivá
settings-light-variant-sepia = Sepia
settings-auto-show-diff = Automaticky otvárať náhľad zmien AI
settings-diff-mode = Zobrazenie AI Diffu
settings-diff-inline = Zlúčené (+ / -)
settings-diff-side-by-side = Vedľa seba
settings-editor-font = Editor — veľkosť písma
settings-ai-font = AI Terminál — veľkosť písma
settings-default-path = Predvolená cesta projektov
settings-creates-in = Bude vytvorené v:
settings-ai-name = Názov asistenta
settings-ai-command = Príkaz (binárka)
settings-ai-args = Parametre (voliteľné)
settings-ai-add = Pridať agenta
settings-ai-hint = Tu si môžete nadefinovať vlastné CLI nástroje (napr. gemini, claude, aider). Ak zoznam necháte prázdny, použijú sa predvolené.
settings-blacklist = Blacklist (zakázané súbory pre pluginy)
settings-blacklist-hint = Podporuje vzory ako *.env, secret/*. Automaticky zakazuje súbory v .gitignore.
settings-blacklist-add = Pridať vzor
settings-suggested-patterns = Odporúčané vzory:

## Pluginy
## Pluginy

## Sémantická indexácia (RAG)
semantic-indexing-title = Sémantická indexácia projektu
semantic-indexing-init = Inicializácia ML modelu (sťahovanie)...
semantic-indexing-processing = Spracovanie: { $processed } / { $total }
semantic-indexing-btn-bg = Spustiť na pozadí
semantic-indexing-status-bar = Indexácia projektu...

## Chyba pluginu

## Strom súborov
file-tree-new-file = Nový súbor
file-tree-new-dir = Nový priečinok
file-tree-rename = Premenovať
file-tree-copy = Kopírovať
file-tree-paste = Vložiť
file-tree-delete = Odstrániť
file-tree-confirm-delete = Odstrániť { $name }?
file-tree-unsafe-name = Neplatný názov: nesmie obsahovať /, \ ani ..
file-tree-outside-project = Cesta by viedla mimo projekt
file-tree-paste-error = Nemožno vložiť: { $reason }
file-tree-create-dir-error = Nemožno vytvoriť priečinok: { $reason }
file-tree-create-file-error = Nemožno vytvoriť súbor: { $reason }
file-tree-rename-error = Nemožno premenovať: { $reason }
file-tree-delete-error = Nemožno odstrániť: { $reason }

## Dialóg externého konfliktu
conflict-title = Súbor zmenený externe
conflict-message = Súbor „{ $name }" bol zmenený (mimo editor), ale v editore má neuložené zmeny.
conflict-choose = Vyberte, ktorú verziu chcete zachovať:
conflict-load-disk = Načítať z disku
conflict-keep-editor = Zachovať z projektu
conflict-dismiss = Zrušiť
conflict-hover-disk = Zahodiť neuložené zmeny v editore a načítať verziu, ktorá bola zmenenú na disku
conflict-hover-keep = Ponechať rozpracované zmeny v editore; verzia na disku bude prepísaná pri vašom najbližšom uložení (Ctrl+S)
conflict-hover-dismiss = Zatvoriť upozornenie bez vykonania zmien

md-open-external = ⧉ Otvoriť vo vonkajšom prehliadači

svg-open-external = ⧉ Otvoriť náhľad v prehliadači

svg-modal-title = SVG súbor
svg-modal-body = Tento súbor je SVG obrázok. Chcete ho otvoriť v systémovom prehliadači, alebo upravovať ako XML text?
svg-modal-edit = Upravovať ako text
settings-conflict-title = Nastavenia zmenené
settings-conflict-message = Nastavenia boli aktualizované v inom okne. Načítať najnovšie hodnoty, alebo pokračovať v úprave?
settings-conflict-reload = Načítať
settings-conflict-keep = Pokračovať v úprave

## Find References (Shift+F12)
lsp-references-heading = Referencie
lsp-references-searching = Vyhľadávanie referencií...
lsp-references-none = Žiadne referencie neboli nájdené.
lsp-references-found =
    { $count ->
        [one] 1 referencia nájdená.
        [few] { $count } referencie nájdené.
       *[other] { $count } referencií nájdených.
    }
lsp-references-error = LSP: Chyba pri hľadaní referencií.

## AI panel updates
ai-diff-heading = Kontrola zmien navrhnutých AI
ai-diff-new-file = Navrhnutý nový súbor

## Pluginy

## Support Modal
support-modal-title = Podporiť vývoj PolyCredo
support-modal-body = PolyCredo Editor je vyvíjaný s víziou súkromia, rýchlosti a bezpečnej integrácie AI asistentov. Ak sa vám projekt páči, budeme vděční za akúkoľvek podporu. Vaše príspevky nám pomáhajú venovať viac času vývoju nových funkcií a údržbe.
support-modal-github = Sledovať na GitHub-e
support-modal-donate = Prispieť na rozvoj
semantic-indexing-btn-stop = Zastaviť indexáciu

dep-wizard-appimagetool-desc = ...
