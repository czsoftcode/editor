---
status: diagnosed
phase: 15-streaming-chat-ui
source: 15-00-SUMMARY.md, 15-01-SUMMARY.md, 15-02-SUMMARY.md, 15-03-SUMMARY.md
started: 2026-03-06T13:30:00Z
updated: 2026-03-06T14:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Streaming odpoved token po tokenu
expected: Otevri AI chat, zadej dotaz a odesli. Odpoved se zobrazuje postupne (token po tokenu), ne cela najednou. Behem generovani se zobrazuje spinner v info baru s textem "Generating...".
result: pass

### 2. Stop/Cancel preruseni streamingu
expected: Behem generovani odpovedi stiskni Escape nebo klikni na cervene "Stop" tlacitko vedle promptu. Streaming se okamzite zastavi, castecna odpoved zustane zobrazena s oznacenim *[preruseno]* na konci.
result: pass

### 3. Theme-aware barvy zprav
expected: User zpravy maji jine pozadi nez AI odpovedi (odvozene z tematu). Prepni dark/light mode v Settings — barvy chatu se automaticky prizpusobi obema rezimum bez hardcoded barev.
result: issue
reported: "generovane zpravy maji v light mode bile pozadi, udelej ho kremovejsi, at tolik nesviti"
severity: cosmetic

### 4. Metadata u zprav
expected: Nad kazdou zpravou je metadata bar: role label ("You" u user, nazev modelu u AI), casove razitko (HH:MM). U posledni AI odpovedi se zobrazuje token count. Copy tlacitko je na konci kazdeho bloku zpravy.
result: pass

### 5. Auto-scroll a Scroll to bottom
expected: Behem streamovani dlouhe odpovedi se chat automaticky scrolluje dolu. Scrollni rucne nahoru — auto-scroll se zastavi a objevi se tlacitko "Scroll to bottom". Kliknuti na tlacitko obnovi auto-scroll.
result: issue
reported: "pri scrolovani se okno postupne natahuje na vysku a Scroll to bottom nefunguje"
severity: major

### 6. Model picker v hlavicce
expected: V hlavicce chatu je ComboBox s dostupnymi Ollama modely. Vedle je status indikator (barevna tecka: zelena=connected, oranzova=checking, cervena=disconnected). Vyber jineho modelu zmeni model pro dalsi dotaz.
result: issue
reported: "barva je modra"
severity: cosmetic

### 7. AI Settings v Settings dialogu
expected: Otevri Settings dialog. Je tam AI sekce s poli: Ollama URL, API Key (maskovany), Default Model, Expertise, Reasoning Depth. Zmena URL a ulozeni spusti reconnect k novemu serveru.
result: issue
reported: "rank se meni az s novym vlaknem"
severity: minor

### 8. API Key se propaguje do requestu
expected: Nastav API key v Settings — AI, uloz. Odesli dotaz na cloud Ollama endpoint (https://ollama.com). Request obsahuje Authorization header s klicem — nedostanes 401 Unauthorized.
result: pass

## Summary

total: 8
passed: 4
issues: 4
pending: 0
skipped: 0

## Gaps

- truth: "AI zpravy maji theme-aware pozadi odlisne od bile v light mode"
  status: failed
  reason: "User reported: generovane zpravy maji v light mode bile pozadi, udelej ho kremovejsi, at tolik nesviti"
  severity: cosmetic
  test: 3
  root_cause: "conversation.rs:94 pouziva ui.visuals().extreme_bg_color, coz je v light mode bila (255,255,255). Melo by pouzivat faint_bg_color nebo vlastni kremovou barvu."
  artifacts:
    - path: "src/app/ui/widgets/ai/chat/conversation.rs"
      issue: "Radek 94: let ai_bg = ui.visuals().extreme_bg_color — bila v light mode"
  missing:
    - "Zmenit ai_bg na faint_bg_color nebo vlastni kremovou barvu odvozenou z tematu"
  debug_session: ""

- truth: "Chat okno zachovava fixni vysku a Scroll to bottom tlacitko funguje"
  status: failed
  reason: "User reported: pri scrolovani se okno postupne natahuje na vysku a Scroll to bottom nefunguje"
  severity: major
  test: 5
  root_cause: "render.rs:137-147 — ScrollArea pouziva max_height z history_content_h_prev, ktera roste s obsahem a roztahuje okno. Radky 190-204 — auto-scroll detekce ve stejnem frame vypina auto_scroll hned po tom, co ho button zapne."
  artifacts:
    - path: "src/app/ui/terminal/ai_chat/render.rs"
      issue: "Radky 137-147: ScrollArea bez fixed height, history_display_h roste s obsahem"
    - path: "src/app/ui/terminal/ai_chat/render.rs"
      issue: "Radky 190-204: auto-scroll detekce vypina button ve stejnem frame"
  missing:
    - "Pouzit history_h_max jako pevnou vysku ScrollArea misto history_content_h_prev"
    - "Opravit timing auto-scroll detekce — ignorovat frame kde se auto_scroll prave zapnul"
  debug_session: ""

- truth: "Status indikator pouziva zelena/oranzova/cervena barvy"
  status: failed
  reason: "User reported: barva je modra"
  severity: cosmetic
  test: 6
  root_cause: "render.rs:20 pouziva ui.visuals().selection.bg_fill pro Connected stav, coz je modra (selekce). Mel by pouzivat zelenou jako ai_bar.rs (Color32::from_rgb(0, 180, 0))."
  artifacts:
    - path: "src/app/ui/terminal/ai_chat/render.rs"
      issue: "Radek 20: OllamaConnectionStatus::Connected => ui.visuals().selection.bg_fill (modra)"
  missing:
    - "Zmenit Connected barvu na Color32::from_rgb(0, 180, 0) jako v ai_bar.rs"
  debug_session: ""

- truth: "Reasoning depth se aplikuje okamzite po zmene v Settings"
  status: failed
  reason: "User reported: rank se meni az s novym vlaknem"
  severity: minor
  test: 7
  root_cause: "reasoning_depth se nacita pouze pri inicializaci workspace (init.rs:74-76) a neuklada se do ws.ai.settings dynamicky. V logic.rs send_query_to_agent se reasoning_depth vubec nepouziva pri tvorbe system promptu."
  artifacts:
    - path: "src/app/ui/terminal/ai_chat/logic.rs"
      issue: "send_query_to_agent nepouziva reasoning_depth pri tvorbe zpravy"
    - path: "src/app/ui/workspace/state/init.rs"
      issue: "Radky 74-76: reasoning_depth se nacita jen jednou pri init"
  missing:
    - "Dynamicky cist reasoning_depth z aktualnich settings pri kazdem send_query_to_agent"
    - "Vlozit reasoning_depth do system promptu pred odeslanim"
  debug_session: ""
