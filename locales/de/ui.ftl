# Allgemeine Benutzeroberflächenelemente

## Panels
panel-files = Dateien
panel-build = Build
panel-build-errors =
    { $count ->
        [one] Fehler (1)
       *[other] Fehler ({ $count })
    }

## Build-Schaltflächen
btn-build = ▶ Build
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean

## Statusleiste
statusbar-line-col = Zeile { $line }, Spalte { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Nicht gespeichert
statusbar-saving = Speichern…
statusbar-saved = Gespeichert
statusbar-filetype-plain = Nur Text

## Editor-Tabs
tab-unsaved-indicator = ●
tab-deleted-indicator = ⚠

## Suchen und Ersetzen
search-label = Suchen:
replace-label = Ersetzen:
search-replace-expand = Ersetzen…
search-placeholder = Suchen…
replace-placeholder = Ersetzen…
search-prev = ▲
search-next = ▼
search-replace-one = Ersetzen
search-replace-all = Alle ersetzen
search-results =
    { $count ->
        [one] 1 Ergebnis
       *[other] { $count } Ergebnisse
    }
search-no-results = Keine Treffer gefunden

## Editor
editor-empty-hint = Öffnen Sie eine Datei aus dem Dateibaum links
editor-preview-label = Vorschau

## Terminal
terminal-unavailable = Terminal ist nicht verfügbar.
terminal-retry = Erneut versuchen
terminal-exited = [Prozess beendet — R drücken um neu zu starten]

## Dialog Zur Zeile springen (Ctrl+G)
goto-line-prompt = Zur Zeile springen:
goto-line-placeholder = Zeilennummer

## Schnelles Öffnen (Ctrl+P)
file-picker-heading = Datei öffnen
file-picker-placeholder = Datei schnell öffnen…
file-picker-no-results = Keine Ergebnisse
file-picker-count = { $count } Dateien
file-picker-count-filtered = { $filtered }/{ $total } Dateien
file-picker-more = … und { $count } weitere

## Projektweite Suche (Ctrl+Shift+F)
project-search-heading = Im Projekt suchen
project-search-placeholder = Im Projekt suchen…
project-search-hint = Suchbegriff…
project-search-btn = Suchen
project-search-loading = Suche läuft…
project-search-result-label = Ergebnisse für „{ $query }" ({ $count })
project-search-results =
    { $count ->
        [one] 1 Ergebnis
       *[other] { $count } Ergebnisse
    }
project-search-no-results = Keine Ergebnisse
project-search-max-results = Max. { $max } Ergebnisse werden angezeigt

## Gemeinsame Schaltflächen
btn-ok = OK
btn-cancel = Abbrechen
btn-close = Schließen
btn-browse = Durchsuchen…
btn-create = Erstellen
btn-open = Öffnen
btn-refresh = Aktualisieren
btn-save = Speichern
btn-rename = Umbenennen
btn-copy = Kopieren
btn-paste = Einfügen
btn-delete = Löschen
btn-name-label = Name:

## KI-Panel
ai-panel-title = KI-Terminal
ai-tool-not-found = Werkzeug { $tool } nicht in PATH gefunden.
ai-tool-detecting = KI-Werkzeuge werden erkannt…
ai-label-assistant = Assistent:
ai-tool-status-checking = { $tool } (prüfe…)
ai-tool-status-available = { $tool } (installiert)
ai-tool-status-missing = { $tool } (nicht in PATH)
ai-hover-reverify = Verfügbarkeit der KI-CLI-Werkzeuge erneut prüfen
ai-hover-checking = Verfügbarkeit der KI-CLI-Werkzeuge wird geprüft…
ai-hover-start = Startet { $tool } (`{ $cmd }`) im Terminal
ai-hover-missing = Befehl `{ $cmd }` nicht in PATH gefunden. Werkzeug installieren und ↻ klicken.
ai-btn-start = ▶ Starten
ai-float-dock = Im Panel andocken
ai-float-undock = Als schwebendes Fenster lösen
ai-tab-close-hover = Tab schließen
ai-tab-new-hover = Neuer Terminal-Tab

## Einstellungen
settings-title = Einstellungen
settings-language = Sprache
settings-language-restart = Sprachänderungen werden sofort wirksam.
settings-theme = Design
settings-theme-dark = Dunkel
settings-theme-light = Hell
settings-editor-font = Editor — Schriftgröße
settings-ai-font = KI-Terminal — Schriftgröße
settings-default-path = Standard-Projektpfad
settings-creates-in = Wird erstellt unter:

## Dateibaum
file-tree-new-file = Neue Datei
file-tree-new-dir = Neuer Ordner
file-tree-rename = Umbenennen
file-tree-copy = Kopieren
file-tree-paste = Einfügen
file-tree-delete = Löschen
file-tree-confirm-delete = { $name } löschen?
file-tree-unsafe-name = Ungültiger Name: darf /, \ oder .. nicht enthalten
file-tree-outside-project = Pfad würde außerhalb des Projekts führen
file-tree-paste-error = Einfügen nicht möglich: { $reason }
file-tree-create-dir-error = Ordner kann nicht erstellt werden: { $reason }
file-tree-create-file-error = Datei kann nicht erstellt werden: { $reason }
file-tree-rename-error = Umbenennen nicht möglich: { $reason }
file-tree-delete-error = Löschen nicht möglich: { $reason }

## Dialog für externen Konflikt
conflict-title = Datei extern geändert
conflict-message = Die Datei „{ $name }" wurde von einem anderen Programm geändert, hat aber ungespeicherte Änderungen im Editor.
conflict-choose = Wählen Sie, welche Version erhalten bleiben soll:
conflict-load-disk = Von Festplatte laden
conflict-keep-editor = Meine behalten
conflict-dismiss = Schließen
conflict-hover-disk = Editor-Änderungen verwerfen und die auf der Festplatte gespeicherte Version laden
conflict-hover-keep = Editor-Änderungen behalten; die Datei auf der Festplatte wird beim Speichern überschrieben
conflict-hover-dismiss = Benachrichtigung ohne Änderungen schließen

md-open-external = ⧉ In externem Betrachter öffnen

svg-open-external = ⧉ Vorschau im Betrachter öffnen

svg-modal-title = SVG-Datei
svg-modal-body = Diese Datei ist ein SVG-Bild. Möchten Sie sie im Systembetrachter öffnen oder als XML-Text bearbeiten?
svg-modal-edit = Als Text bearbeiten
