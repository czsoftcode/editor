# Dialogy aplikace

## Startup dialog
startup-title = PolyCredo Editor
startup-subtitle = AI Polyglot Code Editor
startup-open-folder = Otevřít složku
startup-new-project = Nový projekt
startup-recent-projects = Nedávné projekty
startup-no-recent = Žádné nedávné projekty
startup-quit = Ukončit
startup-missing-session =
    { $count ->
        [one] 1 projekt z předchozí session nebyl nalezen.
       *[other] { $count } projektů z předchozí session nebylo nalezeno.
    }
startup-missing-session-label = Tyto projekty z předchozí session nebylo možné obnovit:
startup-path-label = Cesta:

## Dialog výběru otevření projektu
open-project-title = Otevřít projekt
open-project-question = Projekt je již otevřen. Kde chcete otevřít nový?
open-project-in-window = V tomto okně
open-project-new-window = V novém okně
open-project-cancel = Zrušit

## Průvodce novým projektem
wizard-title = Nový projekt
wizard-project-type = Typ projektu
wizard-project-name = Název projektu
wizard-project-path = Cesta
wizard-type-rust = Rust
wizard-type-symfony = Symfony
wizard-creating = Vytváření projektu…
wizard-name-hint = Povoleny jsou pouze písmena, číslice, _ a -
wizard-error-empty-name = Název projektu nesmí být prázdný.
wizard-error-invalid-name = Neplatný název. Povoleny jsou pouze písmena, číslice, _ a -.
wizard-error-starts-with-dash = Název nesmí začínat pomlčkou.
wizard-error-exists = Projekt s tímto názvem na dané cestě již existuje.
wizard-error-create = Chyba vytváření projektu: { $reason }

## Dialog zavření projektu
close-project-title = Zavřít projekt
close-project-message = Opravdu chcete zavřít tento projekt?
close-project-confirm = Zavřít
close-project-cancel = Zrušit

## Dialog ukončení aplikace
quit-title = Ukončit aplikaci
quit-message = Opravdu chcete ukončit PolyCredo Editor?
quit-confirm = Ukončit
quit-cancel = Zrušit

## Dialog O aplikaci
about-title = O aplikaci
about-version = Verze { $version }
about-build = Build { $build }
about-description = AI Polyglot Code Editor
about-copyright = © 2024–2026 PolyCredo
about-close = Zavřít

## Potvrzovací dialogy (obecné)
confirm-delete-file = Opravdu chcete smazat soubor { $name }?
confirm-delete-folder = Opravdu chcete smazat složku { $name } a veškerý její obsah?
confirm-delete-confirm = Smazat
confirm-delete-cancel = Zrušit

## Přejmenování
rename-title = Přejmenovat
rename-label = Nový název:
rename-confirm = Přejmenovat
rename-cancel = Zrušit
