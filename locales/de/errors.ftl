# Fehler- und Informationsmeldungen

## Dateien
error-file-read = Fehler beim Lesen der Datei: { $path }
error-file-write = Fehler beim Schreiben der Datei: { $path }
error-file-save = Fehler beim Speichern von „{ $name }": { $reason }
error-file-deleted = Datei wurde gelöscht: { $path }
error-file-delete = Fehler beim Löschen von { $name }: { $reason }
error-file-rename = Fehler beim Umbenennen: { $reason }
error-file-create = Fehler beim Erstellen der Datei { $name }: { $reason }
error-file-read-only-error = "{ $name }" kann nicht gespeichert werden, da sie nicht korrekt gelesen werden konnte. Dieser Tab ist nun schreibgeschützt, um Datenverlust zu vermeiden.
error-safe-mode-blocked = Das Projekt befindet sich im sicheren Modus (schreibgeschützt). Sie können Änderungen nur in der Sandbox vornehmen oder den sicheren Modus in den Einstellungen deaktivieren.
error-file-watch = Fehler bei der Dateiüberwachung

## Verzeichnisse
error-folder-create = Fehler beim Erstellen des Ordners { $name }: { $reason }
error-folder-delete = Fehler beim Löschen des Ordners { $name }: { $reason }

## Projekte
error-project-create = Fehler beim Erstellen des Projekts: { $reason }
error-project-open = Fehler beim Öffnen des Projekts: { $path }
error-project-not-found = Projekt nicht gefunden: { $path }
error-project-dir-create = Projektverzeichnis kann nicht erstellt werden: { $reason }
error-cmd-failed = Befehl fehlgeschlagen mit Code: { $code }
error-cmd-start = Befehl konnte nicht gestartet werden: { $reason }
error-projects-dir-prepare = Projektverzeichnis kann nicht vorbereitet werden: { $reason }

## Sitzung
error-session-restore = Projekt der vorherigen Sitzung nicht gefunden: { $path }
error-session-load = Fehler beim Laden der Sitzung.
error-session-save = Fehler beim Speichern der Sitzung.

## Build
error-build-parse = Fehler beim Verarbeiten der Build-Ausgabe.

## Zwischenablage
error-clipboard = Fehler in der Zwischenablage: { $reason }

## IPC
error-ipc-connect = Fehler beim Verbinden mit der laufenden Instanz.

## Allgemein
error-unknown = Ein unbekannter Fehler ist aufgetreten.

## Informationsmeldungen (Toast-Info)
info-file-saved = Datei gespeichert.
info-project-created = Projekt { $name } erfolgreich erstellt.
info-session-restored =
    { $count ->
        [one] 1 Fenster der vorherigen Sitzung wiederhergestellt.
       *[other] { $count } Fenster der vorherigen Sitzung wiederhergestellt.
    }
