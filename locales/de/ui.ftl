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

menu-build-windows = Windows

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

command-name-install-nsis = Install NSIS
command-name-install-rpm = Install rpm-build (dnf)
command-name-install-generate-rpm = Install cargo-generate-rpm
command-name-install-appimage = Install cargo-appimage
command-name-install-flatpak = Install flatpak-builder
command-name-install-snap = Install snapcraft
command-name-configure-lxd = LXD konfigurieren (für Build)
command-name-install-deb-tools = Install Debian Build Tools
command-name-install-freebsd-target = FreeBSD Target installieren (rustup)
command-name-install-cross = cross installieren (Cross-Compilation)
command-name-install-fpm = fpm installieren (gem install fpm)
command-name-install-podman = Podman installieren (Container-Engine für cross)
command-name-install-appimagetool = Install appimagetool
command-name-install-windows-target = Install Windows Target (rustup)
command-name-install-xwin = Install cargo-xwin
command-name-install-clang = Install Clang (LLVM)
command-name-install-lld = Install LLD (Linker)

dep-wizard-xwin-desc = The cargo-xwin tool is required for cross-compiling for Windows MSVC from Linux.
dep-wizard-generate-rpm-desc = The cargo-generate-rpm tool is required to create an .rpm package directly from your Rust project.
dep-wizard-appimage-desc = The cargo-appimage tool is required to create a portable AppImage package directly from your Rust project.
dep-wizard-flatpak-desc = The flatpak-builder tool is required to build and package the application into the Flatpak format.
dep-wizard-snap-desc = The snapcraft tool is required to create Snap packages for Ubuntu and other distributions.
dep-wizard-lxd-desc = LXD ist ein Container-System, das von snapcraft zum Erstellen von Snap-Paketen benötigt wird. Fügt Ihren Benutzer zur lxd-Gruppe hinzu und initialisiert LXD.
dep-wizard-deb-desc = System tools like dpkg-dev, build-essential, and fakeroot are required to create .deb packages.
dep-wizard-freebsd-target-desc = Die Rust-Standardbibliothek für x86_64-unknown-freebsd ist für die Cross-Kompilierung für FreeBSD erforderlich.
dep-wizard-cross-desc = Das cross-Tool ermöglicht die Cross-Kompilierung für FreeBSD und andere Plattformen über Docker/Podman-Container.
dep-wizard-fpm-desc = fpm (Effing Package Manager) ermöglicht die Erstellung nativer FreeBSD .pkg-Pakete unter Linux.
dep-wizard-podman-desc = Podman ist eine Container-Engine, die vom cross-Tool für die Cross-Kompilierung benötigt wird. Eine daemonfreie Alternative zu Docker.
dep-wizard-clang-desc = Clang compiler is required for building native C/C++ dependencies for Windows.
dep-wizard-lld-desc = LLD linker is required for linking Windows binaries on Linux.
dep-wizard-windows-target-desc = Rust standard library for x86_64-pc-windows-msvc is required for compilation.
dep-wizard-nsis-desc = NSIS is required to create Windows installers.
dep-wizard-rpm-desc = The rpmbuild utility is required to create .rpm packages.
dep-wizard-appimagetool-desc = The appimagetool utility is required for final AppImage bundling.
dep-wizard-zigbuild-desc = cargo-zigbuild wird für die Cross-Kompilierung für macOS (Intel + Apple Silicon) von Linux aus benötigt. Verwendet den Zig-Compiler als Linker.
dep-wizard-macos-targets-desc = Rust-Standardbibliotheken für x86_64-apple-darwin und aarch64-apple-darwin werden für die Cross-Kompilierung für macOS benötigt.
dep-wizard-genisoimage-desc = genisoimage wird verwendet, um ein .dmg-Disk-Image aus dem macOS .app-Bundle zu erstellen.
dep-wizard-macos-deps-desc = Installiert alle für macOS-Builds benötigten Tools: cargo-zigbuild (Cross-Compiler), zig (Linker), Rust-Targets für x86_64-apple-darwin und aarch64-apple-darwin sowie LLVM (lipo für Universal Binary).
dep-wizard-llvm-desc = LLVM stellt das lipo-Tool bereit, um Intel- (x86_64) und Apple-Silicon-Binärdateien (aarch64) zu einem Universal Binary (.app / .dmg) zusammenzuführen.

menu-build-macos-sub = macOS
menu-build-macos-dmg = .dmg erstellen (macOS)
command-name-install-macos-deps = macOS-Abhängigkeiten installieren (cargo-zigbuild + zig + Targets)
command-name-install-llvm = LLVM installieren (lipo — Universal Binary)

menu-build-fedora = Fedora
menu-build-debian = Debian / Ubuntu
menu-build-freebsd = FreeBSD
menu-build-freebsd-pkg = .pkg erstellen (FreeBSD)
menu-build-flatpak-sub = Flatpak
menu-build-snap-sub = Snap
menu-build-appimage-sub = AppImage
menu-build-deb = .deb erstellen
menu-build-rpm = .rpm erstellen
menu-build-flatpak = Flatpak-Bundle erstellen
menu-build-snap = Snap-Paket erstellen
menu-build-appimage = .AppImage erstellen
menu-build-exe = .exe erstellen (Windows)
menu-build = Build

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
settings-auto-show-diff = KI-Änderungsvorschau automatisch öffnen
settings-safe-mode = Sicherer Modus (Projekt schreibgeschützt)
settings-safe-mode-hint = Wenn aktiviert, wechseln Dateibaum und Build zum Sandbox-Modus, und direktes Speichern im Projekt ist blockiert.
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

menu-build-all = Alle Pakete
build-all-status-running = Erstelle Pakete…
build-all-status-ok = ✔ Alle Pakete erfolgreich erstellt
build-all-status-error = ✘ Build mit Fehlern beendet (Code { $code })
build-all-status-waiting = Warte auf Start…
build-all-waiting-output = Starte scripts/build-all.sh…
build-all-btn-close = Schließen
build-all-btn-run = Ausführen
build-all-btn-rerun = Erneut ausführen
build-all-not-started = Paket auswählen und Ausführen klicken
build-all-hint-start = Paket oben auswählen und Ausführen klicken…
