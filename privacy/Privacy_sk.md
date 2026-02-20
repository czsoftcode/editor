# PolyCredo Editor — Zásady ochrany súkromia

**Posledná aktualizácia:** 2026-02-20

---

## Čo je PolyCredo Editor

PolyCredo Editor je desktopový editor kódu. Ide o grafické rozhranie pre prácu so súbormi na vašom stroji a pre spúšťanie AI CLI nástrojov (ako Claude Code alebo Gemini CLI) v integrovanom termináli.

---

## Zber dát

**PolyCredo Editor žiadne dáta neodosiela ani nezhromažďuje.**

Konkrétne editor:

- neodosiela váš zdrojový kód, obsah súborov ani cesty k súborom na žiadny server
- neobsahuje telemetriu, analytiku ani hlásenie chýb
- sám od seba nekomunikuje s žiadnym externým API
- nevyžaduje účet ani registráciu
- neukladá žiadne dáta mimo váš stroj

Všetok stav, ktorý editor ukladá (otvorené session, nedávne projekty, nastavenia), sa ukladá lokálne do `~/.config/polycredo-editor/`.

---

## AI nástroje

PolyCredo Editor sa integruje s AI CLI nástrojmi tretích strán, ako je **Claude Code** (Anthropic) a **Gemini CLI** (Google). Tieto nástroje bežia ako samostatné procesy v integrovanom termináli editora — rovnako ako keby ste ich spustili v akomkoľvek inom termináli.

**Editor nekontroluje, nezachytáva ani nepreposiela žiadne dáta, ktoré tieto nástroje odosielajú alebo prijímajú.**

Keď AI nástroj v PolyCredo Editore používate:

- nástroj komunikuje priamo so servermi svojho poskytovateľa (Anthropic, Google a pod.)
- platia zásady ochrany súkromia a podmienky používania daného poskytovateľa
- za to, čo s AI nástrojom zdieľate, zodpovedáte vy — rovnako ako keby ste ho spustili v samostatnom termináli

Rola PolyCredo Editora je ekvivalentná terminálovému emulátoru — zobrazuje výstup, obsah nespracováva ani nepreposiela.

### Offline / lokálne AI

Pokiaľ používate lokálne bežiaci model (napr. Ollama + Aider), žiadne dáta neopúšťajú váš stroj. PolyCredo Editor podporuje akýkoľvek AI nástroj, ktorý beží ako CLI proces — vrátane plne lokálnych riešení.

---

## Rozdelenie zodpovednosti

| Kto | Zodpovednosť |
|---|---|
| **Autor PolyCredo Editora** | Zaistiť, aby editor sám neodosielal žiadne dáta bez explicitnej akcie používateľa |
| **Používateľ** | Voľba AI nástrojov a prijatie ich podmienok používania |
| **Poskytovateľ AI nástroja** (Anthropic, Google, …) | Nakladanie s dátami podľa vlastných zásad ochrany súkromia |

---

## Cloudové funkcie

Prípadné budúce cloudové funkcie (synchronizácia nastavení, zdieľanie snippetov a pod.) budú:

- **opt-in** — vo východiskovom stave vypnuté
- jasne zdokumentované — čo odosielajú a kam
- úplne voliteľné — editor funguje plnohodnotne aj bez nich

---

## Kontakt

Otázky a hlásenia problémov: [https://github.com/czsoftcode/editor/issues](https://github.com/czsoftcode/editor/issues)
