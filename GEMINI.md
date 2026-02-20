# PolyCredo Editor — Gemini CLI Guidelines

Tento soubor obsahuje instrukce a pravidla pro vývoj projektu PolyCredo Editor. Gemini CLI se těmito pravidly řídí při každé interakci.

## Komunikace a Jazyk
- **Interakce s uživatelem:** Probíhá výhradně v **češtině**.
- **Komentáře v kódu:** Musí být psány v **angličtině** (pro zachování idiomatické kvality Rust kódu).
- **Názvy proměnných a funkcí:** Anglicky.

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
- **Bezpečnost:** Nikdy neukládej ani necommituj citlivá data.
