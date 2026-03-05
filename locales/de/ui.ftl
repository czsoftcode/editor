# Allgemeine Benutzeroberflächenelemente

## Panels
panel-files = Dateien
panel-files-sandbox = Dateien (Sandbox)
btn-tree-project = Projekt
btn-tree-sandbox = Sandbox
panel-runners = Runner
panel-build = Build
panel-git = Git
panel-build-errors =
    { $count ->
        [one] Fehler (1)
       *[other] Fehler ({ $count })
    }

## Build-Schaltflächen
btn-build = ▶ Build
btn-build-sandbox-on = Sandbox EIN
btn-build-sandbox-off = Sandbox AUS
hover-build-sandbox = Zwischen Projekt-Root und KI-Sandbox zum Ausführen umschalten
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Create .deb
hover-create-deb = Ein Entwicklungs-.deb-Paket mit Build-Nummer erstellen
hover-create-deb-disabled = Im Sandbox-Modus kann kein Paket erstellt werden. Wechseln Sie zu Sandbox AUS.
hover-build-menu-disabled = Das Erstellen ist im Sandbox-Modus oder bei nicht übertragenen Sandbox-Dateien deaktiviert.
btn-run-profile = ▶ Run Profile...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Edit
runner-none = No profiles defined.


## Dependency Wizard
dep-wizard-title = Dependency Installation Wizard
dep-wizard-install-question = Do you want to download and install { $tool } to { $path }?
dep-wizard-install-cmd-question = Do you want to start the installation of { $tool } using a system command?
dep-wizard-btn-install = Install
dep-wizard-btn-run-cmd = Start Installation (requires sudo)
dep-wizard-status-downloading = Downloading...
dep-wizard-status-running = Installing...
dep-wizard-status-success = Installation successful!
dep-wizard-status-error = Installation error: { $error }





## Git-Operationen
git-add-all = git add .
git-commit = git commit -m "..."
git-push = git push
git-status = git status
git-diff = git diff
git-checkout-file = git checkout (Datei)
git-checkout-branch = git checkout (Zweig)
git-pull = git pull
git-reset-hard = git reset --hard
hover-git-disabled-sandbox = Git-Operationen sind deaktiviert, bis alle Sandbox-Änderungen gelöst sind (Schaltfläche 'Änderungen überprüfen' oder 'Alle übernehmen' in der gelben Leiste verwenden).

## Statusleiste
statusbar-line-col = Zeile { $line }, Spalte { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Nicht gespeichert
statusbar-saving = Speichern…
statusbar-saved = Gespeichert
statusbar-lsp-initializing = LSP wird initialisiert...
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

# LSP / rust-analyzer
lsp-missing-title = rust-analyzer fehlt
lsp-missing-msg = Für intelligente Funktionen (Vervollständigung, Fehler) ist rust-analyzer erforderlich. Möchten Sie es installieren?
lsp-install-btn = Installieren
lsp-installing = Installiere rust-analyzer...
lsp-install-success = rust-analyzer wurde erfolgreich installiert. LSP wird neu gestartet...
lsp-install-error = Installation fehlgeschlagen: { $error }

## Terminal
terminal-unavailable = Terminal ist nicht verfügbar.
terminal-retry = Erneut versuchen
terminal-exited = [Prozess beendet — R drücken um neu zu starten]
terminal-close-confirm-title = Terminal schließen?
terminal-close-confirm-msg = Im Terminal läuft noch ein Prozess. Möchten Sie ihn wirklich beenden?

## Dialog Zur Zeile springen (Ctrl+G)
goto-line-prompt = Zur Zeile springen:
goto-line-placeholder = Zeilennummer

## Command Palette (Ctrl+Shift+P)
command-palette-heading = Befehle
command-palette-placeholder = Befehl suchen…
command-palette-no-results = Keine Ergebnisse

command-name-open-file = Datei öffnen
command-name-project-search = Im Projekt suchen
command-name-build = Erstellen (Build)
command-name-run = Ausführen (Run)
command-name-save = Aktuelle Datei speichern
command-name-close-tab = Aktuellen Tab schließen
command-name-new-project = Neues Projekt
command-name-open-project = Projekt öffnen (in neuem Fenster)
command-name-open-folder = Ordner öffnen (in diesem Fenster)
command-name-toggle-left = Dateipanel umschalten
command-name-toggle-right = KI-Panel umschalten
command-name-toggle-build = Build-Terminal umschalten
command-name-toggle-float = Schwebendes KI-Panel umschalten
command-name-show-about = Über
command-name-show-settings = Einstellungen
command-name-quit = PolyCredo Editor beenden
command-name-plugin-hello = Plugin: Hallo sagen
command-name-plugin-gemini = Plugin: Gemini fragen
command-name-show-plugins = Pluginy

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
btn-confirm = Bestätigen
btn-cancel = Abbrechen
btn-close = Schließen
cancel-confirm-title = Änderungen verwerfen?
cancel-confirm-msg = Möchten Sie wirklich alle ungespeicherten Änderungen verwerfen und dieses Fenster schließen?
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
ai-plugin-bar-label = KI:
ai-plugin-bar-settings = ⚙
ai-plugin-bar-start-hover = KI-Chat mit ausgewähltem Plugin starten
ai-plugin-bar-settings-hover = Einstellungen für das ausgewählte KI-Plugin öffnen
ai-float-dock = Im Panel andocken
ai-float-undock = Als schwebendes Fenster lösen
ai-viewport-open = In separatem Fenster öffnen
ai-tab-close-hover = Tab schließen
ai-tab-new-hover = Neuer Terminal-Tab
ai-staged-bar-msg = KI hat Änderungen im Projekt vorgeschlagen
ai-staged-bar-review = Änderungen überprüfen
ai-staged-bar-promote-all = Alle übernehmen
ai-staged-modal-hint = Klicken Sie auf eine Datei, um Unterschiede anzuzeigen und Änderungen zu genehmigen:
ai-staged-files = Vorgeschlagene Änderungen (Sandbox)
ai-staged-new = [NEU]
ai-staged-mod = [MOD]
ai-staged-del = [GELÖSCHT]
ai-promotion-success-title = Änderungen angewendet
ai-promotion-success-body = Die folgende Datei wurde erfolgreich in Ihrem Projekt aktualisiert:
ai-promotion-success = Die Änderungen wurden erfolgreich in das Projekt übernommen.
ai-promotion-all-success = Erfolgreich { $count } Dateien in das Projekt übertragen.
ai-promotion-failed = Änderungen konnten nicht angewendet werden: { $error }

## Synchronisierung vor dem Start von AI
ai-sync-title = Synchronisierung vor dem Start
ai-sync-msg = Unterschiede zwischen Projekt und Sandbox erkannt. Die neuesten Versionen sollten synchronisiert werden.
ai-sync-to-sandbox = Sandbox aktualisieren ({ $count } neuere im Projekt)
ai-sync-to-project = Ins Projekt befördern ({ $count } neuere in der Sandbox)
ai-sync-btn-sync = Synchronisieren und Starten
ai-sync-btn-skip = Ohne Synchronisierung starten

## Plugin-Berechtigungen
plugin-auth-bar-msg = Das Plugin „{ $name }“ beantragt Internetzugriff ({ $hosts }).
plugin-auth-bar-allow = Zulassen und Starten
plugin-auth-bar-deny = Ablehnen

## Einstellungen
settings-title = Einstellungen
settings-category-general = Allgemein
settings-category-editor = Editor
settings-category-ai = KI-Agenten
settings-language = Sprache
settings-language-restart = Sprachänderungen werden sofort wirksam.
settings-theme = Design
settings-theme-dark = Dunkel
settings-theme-light = Hell
settings-light-variant = Helle Variante
settings-light-variant-warm-ivory = Warmes Elfenbein
settings-light-variant-cool-gray = Kühles Grau
settings-light-variant-sepia = Sepia
settings-auto-show-diff = KI-Änderungsvorschau automatisch öffnen
settings-safe-mode = Sicherer Modus (Projekt schreibgeschützt)
settings-safe-mode-hint = Wenn aktiviert, wechseln Dateibaum und Build zum Sandbox-Modus, und direktes Speichern im Projekt ist blockiert.
settings-safe-mode-tooltip = Aus: Arbeiten direkt im Projektstamm und Terminals laufen im Projektstamm. Gilt nach erneutem Öffnen des Projekts.
settings-safe-mode-terminal-note = Nach dem Moduswechsel werden Terminal-Prozesse nach erneutem Öffnen des Projekts neu gestartet.
settings-sandbox-toast-off = Sandbox-Modus ausgeschaltet. Die Änderung gilt nach erneutem Öffnen des Projekts.
settings-sandbox-toast-on = Sandbox-Modus eingeschaltet. Terminals starten nach dem erneuten Öffnen im Sandbox-Modus.
settings-diff-mode = KI-Diff-Layout
settings-diff-inline = Zusammengefügt (+ / -)
settings-diff-side-by-side = Nebeneinander
settings-editor-font = Editor — Schriftgröße
settings-ai-font = KI-Terminal — Schriftgröße
settings-default-path = Standard-Projektpfad
settings-creates-in = Wird erstellt unter:
settings-ai-name = Assistenten-Name
settings-ai-command = Befehl (Binärdatei)
settings-ai-args = Argumente (optional)
settings-ai-add = Agent hinzufügen
settings-ai-hint = Hier können Sie Ihre eigenen CLI-Tools definieren (z. B. gemini, claude, aider). Wenn die Liste leer ist, werden Standardeinstellungen verwendet.
settings-blacklist = Blacklist (gesperrte Dateien für Plugins)
settings-blacklist-hint = Unterstützt Muster wie *.env, secret/*. Sperrt automatisch .gitignore-Dateien.
settings-blacklist-add = Muster hinzufügen
settings-suggested-patterns = Empfohlene Muster:

## Pluginy
plugins-title = Plugin-Manager
plugins-config-label = Plugin-Konfiguration:
plugins-unknown-agent = Unbekannter Agent
## Plugins
plugins-title = Plugin-Manager
plugins-list-label = Plugin-Liste
plugins-no-selection = Wählen Sie ein Plugin aus der Liste links
plugins-enabled-label = Dieses Plugin aktivieren
plugins-config-label = Plugin-Konfiguration:
plugins-unknown-agent = Unbekannter Agent
plugins-category-ai = 🤖 KI-Agenten
plugins-category-general = ⚙ Allgemein
plugins-item-settings = Einstellungen
plugins-item-welcome = Übersicht
plugins-welcome-title = Willkommen im Plugin-Manager
plugins-welcome-text = Der PolyCredo Editor verwendet ein modernes Plugin-System basierend auf der WebAssembly (WASM) Technologie. Dies gewährleistet hohe Leistung und maximale Sicherheit — Plugins laufen in einer isolierten Umgebung (Sandbox) und haben nur Zugriff auf das, was Sie explizit erlauben.
plugins-welcome-hint = Wählen Sie eine Kategorie oder ein bestimmtes Plugin aus der Liste links, um es zu konfigurieren.
plugins-security-info = 🛡 Sicherheit: Sie können die Datei/Ordner-Blacklist in den Haupteinstellungen verwalten.
plugins-settings-saved = Plugin-Einstellungen gespeichert. Neustart bei einigen Änderungen empfohlen.
plugins-placeholder-api-key = API-Schlüssel (z. B. Gemini, Anthropic)
command-name-show-plugins = Plugins

## Gemini AI

## Semantische Indexierung (RAG)
semantic-indexing-title = Semantische Projektindexierung
semantic-indexing-init = ML-Modell wird initialisiert (Download)...
semantic-indexing-processing = Verarbeitung: { $processed } / { $total }
semantic-indexing-btn-bg = Im Hintergrund ausführen
semantic-indexing-status-bar = Projekt-Indexierung...

## Plugin-Fehler
plugin-error-title = Plugin-Fehler
plugin-error-heading = Plugin-Fehlgeschlagen

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
conflict-message = Die Datei „{ $name }" wurde geändert (wahrscheinlich durch Sandbox-Übernahme), hat aber ungespeicherte Änderungen im Editor.
conflict-choose = Wählen Sie, welche Version Sie behalten möchten:
conflict-load-disk = Aus Sandbox überschreiben
conflict-keep-editor = Aus Projekt beibehalten
conflict-dismiss = Abbrechen
conflict-hover-disk = Nicht gespeicherte Editor-Änderungen verwerfen und die gerade aus der Sandbox übernommene Version laden
conflict-hover-keep = In Arbeit befindliche Änderungen im Editor behalten; die Sandbox-Version auf der Festplatte wird beim nächsten Speichern (Strg+S) überschrieben
conflict-hover-dismiss = Benachrichtigung schließen, ohne Änderungen vorzunehmen

md-open-external = ⧉ In externem Betrachter öffnen

svg-open-external = ⧉ Vorschau im Betrachter öffnen

svg-modal-title = SVG-Datei
svg-modal-body = Diese Datei ist ein SVG-Bild. Möchten Sie sie im Systembetrachter öffnen oder als XML-Text bearbeiten?
svg-modal-edit = Als Text bearbeiten

## Sandbox OFF / Staged / Sync Dialog
settings-sandbox-off-title = Sandbox-Modus deaktivieren?
settings-sandbox-off-message = Sie sind dabei, den Sandbox-Modus zu deaktivieren. Alle Terminals und der Dateibaum wechseln zum Projektstamm.
settings-sandbox-off-warning = Warnung: Änderungen werden direkt in die Projektdateien geschrieben.
settings-sandbox-off-blocked = Der Sandbox-Modus kann nicht deaktiviert werden, solange noch ausstehende Änderungen vorhanden sind. Lösen Sie diese zuerst.
settings-sandbox-apply-prompt = Ein anderer Dialog ist geöffnet. Sandbox-Änderung jetzt anwenden oder verschieben?
settings-sandbox-apply-now = Jetzt anwenden
settings-sandbox-apply-defer = Verschieben
settings-sandbox-remap-prompt = Sandbox-Modus wurde gewechselt. Offene Dateien auf den neuen Stamm umzuordnen?
settings-sandbox-remap-apply = Tabs umordnen
settings-sandbox-remap-skip = Beibehalten
settings-sandbox-persist-actions = Einstellungen konnten nicht gespeichert werden. Vorübergehend anwenden oder zurücksetzen?
settings-sandbox-persist-revert = Zurücksetzen
settings-sandbox-persist-keep = Vorübergehend beibehalten
settings-sandbox-persist-unsaved = Einstellungen sind vorübergehend angewendet und noch nicht gespeichert.
settings-sandbox-persist-reverted = Einstellungen wurden auf den letzten gespeicherten Zustand zurückgesetzt.
sandbox-sync-title = Projekt in Sandbox synchronisieren?
sandbox-sync-msg = Sandbox-Modus wurde aktiviert. Neueste Projektdateien in die Sandbox übertragen?
sandbox-sync-to-sandbox = Sandbox aktualisieren ({ $count } neuere im Projekt)
sandbox-sync-nothing = Keine neueren Dateien im Projekt zum Übertragen.
sandbox-sync-btn-sync = Jetzt synchronisieren
sandbox-sync-btn-skip = Überspringen
sandbox-sync-success = Sandbox aktualisiert ({ $count } Datei(en) synchronisiert).
sandbox-sync-error = Sandbox-Synchronisierung fehlgeschlagen: { $error }
settings-conflict-title = Einstellungen geändert
settings-conflict-message = Einstellungen wurden in einem anderen Fenster aktualisiert. Neu laden oder aktuellen Entwurf beibehalten?
settings-conflict-reload = Neu laden
settings-conflict-keep = Weiter bearbeiten

## Dialog zur Synchronisation des Löschens in der Sandbox
sandbox-delete-title = Datei in der Sandbox gelöscht
sandbox-delete-msg = Die Datei „{ $name }" wurde in der KI-Sandbox gelöscht, existiert aber im Projekt noch. Was möchten Sie tun?
sandbox-delete-keep-project = Im Projekt behalten (in der Sandbox wiederherstellen)
sandbox-delete-also-project = Auch im Projekt löschen
panel-runners = Runners
btn-run-profile = Run Profile...
btn-edit-profiles = Edit
runner-none = None

## Find References (Shift+F12)
lsp-references-heading = Referenzen
lsp-references-searching = Suche nach Referenzen...
lsp-references-none = Keine Referenzen gefunden.
lsp-references-found =
    { $count ->
        [one] 1 Referenz gefunden.
       *[other] { $count } Referenzen gefunden.
    }
lsp-references-error = LSP: Fehler bei der Suche nach Referenzen.

ai-btn-sync = ⟳ Sync
ai-hover-sync = Kontext (offene Dateien, Build-Fehler) an KI-Agenten senden
ai-diff-heading = Vorgeschlagene KI-Änderungen überprüfen
ai-diff-new-file = Neue Datei vorgeschlagen
ai-chat-title = AI Chat Assistant
ai-chat-label-response = Response:
ai-chat-loading = AI is thinking…
ai-chat-label-prompt = Your prompt:
ai-chat-placeholder-prompt = Enter instructions for AI (e.g. "Explain this code")...
ai-chat-btn-send = Send
ai-chat-btn-new = New Thread
ai-chat-settings-title = AI Settings
ai-chat-label-language = Language:
ai-chat-btn-reset = Reset
ai-chat-label-system-prompt = System Prompt:
ai-chat-default-prompt = Expert Rust Developer.
command-name-plugin-ai-chat = Plugin: Ask AI Agent
command-name-plugin-ollama = Plugin: Ask Ollama
command-name-open-file = Open File
command-name-project-search = Search in Project
command-name-build = Build
command-name-run = Run
command-name-save = Save Current File
command-name-close-tab = Close Current Tab
command-name-new-project = New Project
command-name-open-project = Open Project (in new window)
command-name-open-folder = Open Folder (in this window)
command-name-toggle-left = Toggle File Panel
command-name-toggle-right = Toggle AI Panel
command-name-toggle-build = Toggle Build Terminal
command-name-toggle-float = Toggle Floating AI Panel
command-name-show-about = About
command-name-show-settings = Settings
command-name-quit = Quit PolyCredo Editor
command-name-plugin-hello = Plugin: Say Hello
command-name-plugin-gemini = Plugin: Ask Gemini
plugins-title = Plugin Manager
plugins-list-label = Plugins List
plugins-no-selection = Select a plugin from the list on the left
plugins-enabled-label = Enable this plugin
plugins-config-label = Plugin Configuration:
plugins-unknown-agent = Unknown Agent
plugins-category-ai = 🤖 AI Agents
plugins-category-general = ⚙ General
plugins-item-settings = Settings
plugins-item-welcome = Overview
plugins-welcome-title = Welcome to Plugin Manager
plugins-welcome-text = PolyCredo Editor utilizes a modern plugin system based on WebAssembly (WASM). This ensures high performance and maximum security — plugins run in an isolated environment (sandbox) and only have access to what you explicitly authorize.
plugins-welcome-hint = Select a category or a specific plugin from the list on the left to configure it.
plugins-security-info = 🛡 Security: You can manage the file/directory blacklist in the main Settings.
plugins-settings-saved = Plugin settings saved. Restart recommended for some changes.
plugins-placeholder-api-key = API Key (e.g. Gemini, Anthropic)
plugins-placeholder-model = Model ID (e.g. gemini-1.5-flash)
command-name-show-plugins = Plugins
ai-chat-title = AI Chat Assistant
ai-chat-label-response = Response:
ai-chat-loading = AI is thinking…
ai-chat-label-prompt = Your prompt:
ai-chat-placeholder-prompt = Enter instructions for AI (e.g. "Explain this code")...
ai-chat-btn-send = Send
ai-chat-btn-new = New Thread
ai-chat-settings-title = AI Settings
ai-chat-label-language = Language:
ai-chat-btn-reset = Reset
ai-chat-label-system-prompt = System Prompt:
ai-chat-default-prompt = Expert Rust Developer.
command-name-plugin-gemini = Plugin: Ask Gemini
command-name-plugin-ollama = Plugin: Ask Ollama
command-name-plugin-ai-chat = Plugin: Ask AI Agent

## Support Modal
support-modal-title = PolyCredo-Entwicklung unterstützen
support-modal-body = PolyCredo Editor wird mit einer Vision von Privatsphäre, Geschwindigkeit und sicherer KI-Assistenten-Integration entwickelt. Wenn Ihnen das Projekt gefällt, wären wir für jede Unterstützung dankbar. Ihre Beiträge helfen uns, mehr Zeit für die Entwicklung neuer Funktionen und die Wartung aufzuwenden.
support-modal-github = Auf GitHub folgen
support-modal-donate = Zur Entwicklung beitragen
semantic-indexing-btn-stop = Indizierung stoppen

dep-wizard-appimagetool-desc = ...
