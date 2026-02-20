# PolyCredo Editor — Datenschutzrichtlinie

**Zuletzt aktualisiert:** 20.02.2026

---

## Was ist der PolyCredo Editor?

PolyCredo Editor ist ein Desktop-Code-Editor. Er ist eine grafische Benutzeroberfläche für die Arbeit mit Dateien auf Ihrem Rechner und für die Ausführung von KI-CLI-Tools (wie Claude Code oder Gemini CLI) in einem integrierten Terminal.

---

## Datenerhebung

**PolyCredo Editor sendet oder sammelt keine Daten.**

Im Einzelnen gilt für den Editor:

- Er sendet Ihren Quellcode, Dateiinhalte oder Dateipfade nicht an Server.
- Er enthält keine Telemetrie, Analysen oder Fehlerberichte.
- Er kommuniziert von sich aus mit keiner externen API.
- Er erfordert kein Konto und keine Registrierung.
- Er speichert keine Daten außerhalb Ihres Rechners.

Alle vom Editor gespeicherten Zustände (geöffnete Sitzungen, letzte Projekte, Einstellungen) werden lokal unter `~/.config/polycredo-editor/` gespeichert.

---

## KI-Tools

PolyCredo Editor lässt sich mit KI-CLI-Tools von Drittanbietern wie **Claude Code** (Anthropic) und **Gemini CLI** (Google) integrieren. Diese Tools laufen als separate Prozesse im integrierten Terminal des Editors — genau so, als ob Sie sie in einem anderen Terminal ausführen würden.

**Der Editor kontrolliert, erfasst oder leitet keine Daten weiter, die diese Tools senden oder empfangen.**

Wenn Sie ein KI-Tool im PolyCredo Editor verwenden:

- Kommuniziert das Tool direkt mit den Servern seines Anbieters (Anthropic, Google usw.).
- Gelten die Datenschutzrichtlinien und Nutzungsbedingungen des jeweiligen Anbieters.
- Sind Sie dafür verantwortlich, was Sie mit dem KI-Tool teilen — genau so, als ob Sie es in einem separaten Terminal starten würden.

Die Rolle des PolyCredo Editors entspricht der eines Terminal-Emulators — er zeigt die Ausgabe an, verarbeitet oder leitet den Inhalt jedoch nicht weiter.

### Offline / Lokale KI

Wenn Sie ein lokal laufendes Modell verwenden (z. B. Ollama + Aider), verlassen keine Daten Ihren Rechner. PolyCredo Editor unterstützt jedes KI-Tool, das als CLI-Prozess läuft — einschließlich vollständig lokaler Lösungen.

---

## Verteilung der Verantwortung

| Wer | Verantwortung |
|---|---|
| **Autor des PolyCredo Editors** | Sicherstellen, dass der Editor selbst keine Daten ohne ausdrückliche Benutzeraktion sendet. |
| **Benutzer** | Auswahl der KI-Tools und Akzeptanz ihrer Nutzungsbedingungen. |
| **Anbieter des KI-Tools** (Anthropic, Google, …) | Umgang mit Daten gemäß den eigenen Datenschutzrichtlinien. |

---

## Cloud-Funktionen

Mögliche zukünftige Cloud-Funktionen (Synchronisierung von Einstellungen, Teilen von Snippets usw.) werden:

- **opt-in** sein — standardmäßig deaktiviert.
- klar dokumentiert sein — was sie wohin senden.
- vollständig optional sein — der Editor ist auch ohne sie voll funktionsfähig.

---

## Kontakt

Fragen und Fehlermeldungen: [https://github.com/czsoftcode/editor/issues](https://github.com/czsoftcode/editor/issues)
