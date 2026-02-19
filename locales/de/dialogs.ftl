# Anwendungsdialoge

## Startdialog
startup-title = PolyCredo Editor
startup-subtitle = AI Polyglot Code Editor
startup-open-folder = Ordner öffnen
startup-new-project = Neues Projekt
startup-recent-projects = Zuletzt geöffnet
startup-no-recent = Keine zuletzt geöffneten Projekte
startup-quit = Beenden
startup-missing-session =
    { $count ->
        [one] 1 Projekt der vorherigen Sitzung wurde nicht gefunden.
       *[other] { $count } Projekte der vorherigen Sitzung wurden nicht gefunden.
    }
startup-missing-session-label = Projekte der vorherigen Sitzung konnten nicht wiederhergestellt werden:
startup-path-label = Pfad:

## Dialog Projekt öffnen
open-project-title = Projekt öffnen
open-project-question = Ein Projekt ist bereits geöffnet. Wo soll das neue geöffnet werden?
open-project-in-window = In diesem Fenster
open-project-new-window = In neuem Fenster
open-project-cancel = Abbrechen

## Assistent für neues Projekt
wizard-title = Neues Projekt
wizard-project-type = Projekttyp
wizard-project-name = Projektname
wizard-project-path = Pfad
wizard-type-rust = Rust
wizard-type-symfony = Symfony
wizard-creating = Projekt wird erstellt…
wizard-name-hint = Nur Buchstaben, Ziffern, _ und - sind erlaubt
wizard-error-empty-name = Der Projektname darf nicht leer sein.
wizard-error-invalid-name = Ungültiger Name. Nur Buchstaben, Ziffern, _ und - sind erlaubt.
wizard-error-starts-with-dash = Der Name darf nicht mit einem Bindestrich beginnen.
wizard-error-exists = Ein Projekt mit diesem Namen existiert bereits am angegebenen Pfad.
wizard-error-create = Fehler beim Erstellen des Projekts: { $reason }

## Dialog Projekt schließen
close-project-title = Projekt schließen
close-project-message = Möchten Sie dieses Projekt wirklich schließen?
close-project-confirm = Schließen
close-project-cancel = Abbrechen

## Beenden-Dialog
quit-title = Anwendung beenden
quit-message = Möchten Sie PolyCredo Editor wirklich beenden?
quit-confirm = Beenden
quit-cancel = Abbrechen

## Über-Dialog
about-title = Über
about-version = Version { $version }
about-build = Build { $build }
about-description = AI Polyglot Code Editor
about-copyright = © 2024–2026 PolyCredo
about-close = Schließen

## Bestätigungsdialoge (allgemein)
confirm-delete-file = Möchten Sie { $name } wirklich löschen?
confirm-delete-folder = Möchten Sie { $name } und den gesamten Inhalt wirklich löschen?
confirm-delete-confirm = Löschen
confirm-delete-cancel = Abbrechen

## Umbenennen
rename-title = Umbenennen
rename-label = Neuer Name:
rename-confirm = Umbenennen
rename-cancel = Abbrechen
