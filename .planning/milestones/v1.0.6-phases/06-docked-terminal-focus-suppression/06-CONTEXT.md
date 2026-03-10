# Phase 6: Docked Terminal Focus Suppression - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Docked terminály (pravý Claude panel, spodní build) přestanou krást klávesový fokus při otevřeném modálním okně nebo AI Chat panelu. AI Chat TextEdit dostane autofocus a udrží ho. Hover-to-focus se kompletně zruší — fokus na terminál jen klikem nebo klávesovou zkratkou.

</domain>

<decisions>
## Implementation Decisions

### Hover-to-focus zrušení
- Hover-to-focus na terminál se kompletně ruší (docked i float, všechny terminály)
- `TerminalAction::Hovered` NIKDY nemění `focused_panel`
- Terminál přebírá fokus pouze přes `TerminalAction::Clicked` nebo klávesovou zkratku (Ctrl+Alt+B/A)
- Toto platí univerzálně — ne jen když je otevřený modal/AI chat

### AI Chat není modal
- `show_ai_chat` se NESMÍ přidávat do `dialog_open_base`
- AI Chat neblokuje editor ani terminál — uživatel může psát do AI Chatu, editoru i terminálu
- AI Chat je běžný panel, ne modální okno

### AI Chat autofocus
- Při otevření AI Chatu se TextEdit okamžitě dostane fokus (autofocus)
- `FocusedPanel::AiChat` se nastaví při interakci s AI Chat TextEdit
- Terminál nedostane `focused = true` když `focused_panel == AiChat`

### Modální okna a terminálový fokus
- `dialog_open` blokuje terminálový fokus (klik i hover) — terminál nedostane fokus při otevřeném modalu
- Terminál volá `request_focus()` každý frame když `focused == true` — toto musí být potlačeno při `dialog_open`
- Informativní modaly (About) se zavřou klikem mimo modal
- Datové modaly (Settings, conflict, staged) se NEZAVŘOU klikem mimo — vyžadují explicitní zavření (X / Cancel / Save)

### Claude's Discretion
- Přesný mechanismus potlačení `request_focus()` v `TerminalView::focus()` — buď úprava `focused` parametru v callerech, nebo guard v samotném widgetu
- Zda je potřeba upravit `instance/mod.rs` keyboard forwarding (řádky 346-373) nebo stačí správné nastavení `focused` v callerech
- Implementace "klik mimo zavře modal" pro informativní modaly

</decisions>

<specifics>
## Specific Ideas

- "Uživatel nemusí hlídat myš" — klíčová motivace pro zrušení hover-to-focus, vhodné pro keyboard-driven workflow
- "Nemůžu ani kliknout do AI Chatu" — terminál přebíjí fokus i z kliknutí, ne jen hover
- "To samé dělá i bottom terminal — poslední terminál, do kterého se psalo, si přebírá fokus"
- Modaly (Settings, About) mají stejný problém jako AI Chat — terminál krade fokus z obou

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `FocusedPanel` enum: Build, Claude, Editor, AiChat, Files — hotový state machine pro fokus
- `dialog_open_base` boolean: existující pattern pro blokování fokus při modalech
- `TerminalAction` enum: Clicked, Hovered — Hovered handlery se prostě odstraní

### Established Patterns
- `focused = ws.focused_panel == FocusedPanel::Build && !dialog_open` — tento pattern je správný a funguje
- `TerminalView::set_focus(focused)` → `request_focus()` / `surrender_focus()` — řízeno parametrem
- Keyboard forwarding v `instance/mod.rs:346-373` gated přes `focused` — pokud caller nastaví `false`, keyboard capture se neaktivuje

### Integration Points
- `workspace/mod.rs:361-370` — `dialog_open_base` výpočet (nepřidávat `show_ai_chat`)
- `terminal/right/mod.rs` — docked path (řádek 266, 275-278) a float path (řádek 112, 118)
- `terminal/bottom/mod.rs` — docked path (řádek 149) a float path (řádek 66-86)
- `terminal/instance/mod.rs:215` — `TerminalView::set_focus(focused)` call
- `terminal/instance/mod.rs:256-259` — `TerminalAction::Hovered` generování (ponechat ale ignorovat v callerech)
- `terminal/ai_chat/mod.rs` — AI Chat window, autofocus TextEdit

</code_context>

<deferred>
## Deferred Ideas

- Focus indikátor (vizuální highlight aktivního panelu) — UX vylepšení, ne bug fix
- Konfigurovatelné hover-to-focus (settings toggle) — pokud by někdo hover-to-focus chtěl zpět

</deferred>

---

*Phase: 06-docked-terminal-focus-suppression*
*Context gathered: 2026-03-05*
