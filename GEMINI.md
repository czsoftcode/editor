# PolyCredo Editor — Gemini CLI Guidelines

Tento soubor obsahuje instrukce a pravidla pro vývoj projektu PolyCredo Editor. Gemini CLI se těmito pravidly řídí při každé interakci.

## Komunikace a Jazyk
- **Interakce s uživatelem:** Probíhá výhradně v **češtině**.
- **Komentáře v kódu, CHANGELOG.md a skripty:** Musí být psány v **angličtině** (pro zachování technické konzistence).
- **Názvy proměnných a funkcí:** Anglicky.

## Vize a Strategie
- **Směřování projektu:** Vývoj se řídí dokumentem `docs/vize.md`. Prioritou je stabilita, výkon (Rust) a integrace AI agentů při zachování soukromí (local-first).
- **Roadmapa:** Aktuální stav a úkoly sleduj v souboru `ROADMAPA.md`. Dodržuj kvartální cíle tam definované.

## Technické standardy
- **Lokalizace (i18n):** 
    - Je zakázáno vkládat texty uživatelského rozhraní přímo do kódu (hardcoding).
    - Všechny řetězce musí být definovány v souborech `.ftl` v adresáři `locales/`.
    - Používej systém `i18n.rs` a makra/metody pro přístup k překladům.
- **Kvalita kódu:**
    - Dodržuj Rust Edition 2024 standardy.
    - Před dokončením úkolu vždy ověř kompilaci pomocí `cargo check`.
- **Markdown:**
    - Pro náhled používej výhradně knihovnu `egui_commonmark` s dynamickým škálováním fontu.

## Architektura a údržba
- **Modularita:** 
    - Preferuj menší, jednoúčelové moduly před velkými monolitickými soubory.
    - Pokud soubor přesáhne **500–700 řádků**, Gemini CLI by mělo navrhnout jeho rozdělení do logických podmodulů.
- **UI Komponenty:** Nové složitější UI prvky umísťuj do `src/app/ui/widgets/`.

## Pracovní postup
- **Výzkum -> Strategie -> Realizace:** Vždy nejdříve analyzuj dopady změn v souvislostech celého projektu.
- **Dokumentace:** Po dokončení úkolu z roadmapy vždy aktualizuj `ROADMAPA.md` (označ jako hotové) a přidej záznam do `CHANGELOG.md` (nové záznamy nahoru).
- **Bezpečnost:** Nikdy neukládej ani necommituj citlivá data.
