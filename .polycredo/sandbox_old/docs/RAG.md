# RAG (Retrieval-Augmented Generation) v PolyCredo

Tento dokument popisuje koncept a implementaci techniky RAG pro inteligentní analýzu kódu v editoru PolyCredo.

## Úvod do RAG

**Retrieval-Augmented Generation (RAG)** je metoda, která propojuje velké jazykové modely (LLM) s externími daty v reálném čase.

### Metafora: Maturant vs. Programátor s příručkou

*   **Standardní LLM:** Funguje jako student u zkoušky, který se vše naučil nazpaměť.
*   **RAG:** Funguje jako zkušený programátor s přístupem k dokumentaci.

---

## Technický proces (Workflow)

1. **Indexování:** Kód je převeden na vektory.
2. **Retrieval:** Vyhledání relevantních kousků kódu podle dotazu.
3. **Augmentation & Generation:** Obohacení dotazu o nalezený kód a odpověď.

---

## Výhody pro PolyCredo

1.  **Aktuálnost:** Reakce na změny v kódu bez přetrénování.
2.  **Přesnost:** Drastické snížení halucinací.
3.  **Soukromí:** Posílají se jen nezbytné fragmenty.
4.  **Auditovatelnost:** Odkazy na konkrétní soubory.

---

## Příklad

Uživatel se ptá: *"Jak přidat barvu do terminálu?"*
-> RAG najde `theme.rs`.
-> Model odpoví přesně podle tvého kódu.

---
*Dokumentace vygenerována jako součást návrhu PolyCredo AI.*
