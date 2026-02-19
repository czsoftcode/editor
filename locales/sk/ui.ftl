# Všeobecné prvky používateľského rozhrania

## Panely
panel-files = Súbory
panel-build = Build
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

## Stavový riadok
statusbar-line-col = Riadok { $line }, Stĺpec { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Neuložené
statusbar-saving = Ukladám…
statusbar-saved = Uložené
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

## Terminál
terminal-unavailable = Terminál nie je dostupný.
terminal-retry = Skúsiť znova
terminal-exited = [Proces skončil — stlačte R pre reštart]

## Dialóg Prejsť na riadok (Ctrl+G)
goto-line-prompt = Prejsť na riadok:
goto-line-placeholder = číslo riadku

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
btn-cancel = Zrušiť
btn-close = Zatvoriť
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
ai-tab-close-hover = Zatvoriť záložku
ai-tab-new-hover = Nová záložka terminálu

## Nastavenia
settings-title = Nastavenia
settings-language = Jazyk
settings-language-restart = Zmena jazyka sa prejaví okamžite.
settings-theme = Téma
settings-theme-dark = Tmavá
settings-theme-light = Svetlá
settings-editor-font = Editor — veľkosť písma
settings-ai-font = AI Terminál — veľkosť písma
settings-default-path = Predvolená cesta projektov
settings-creates-in = Bude vytvorené v:

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
conflict-message = Súbor „{ $name }" bol zmenený iným programom, ale v editore má neuložené zmeny.
conflict-choose = Vyberte, ktorá verzia má byť zachovaná:
conflict-load-disk = Načítať z disku
conflict-keep-editor = Ponechať moje
conflict-dismiss = Zavrieť
conflict-hover-disk = Zahodiť zmeny editora a načítať verziu uloženú na disku
conflict-hover-keep = Ponechať zmeny editora; súbor na disku bude prepísaný pri uložení
conflict-hover-dismiss = Zatvoriť upozornenie bez zmien

svg-open-external = ⧉ Otvoriť náhľad v prehliadači

svg-modal-title = SVG súbor
svg-modal-body = Tento súbor je SVG obrázok. Chcete ho otvoriť v systémovom prehliadači, alebo upravovať ako XML text?
svg-modal-edit = Upravovať ako text
