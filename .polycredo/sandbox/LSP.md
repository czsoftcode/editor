# Implementace LSP Klienta v PolyCredo

Tento dokument sleduje postup implementace Language Server Protocol (LSP) klienta pro podporu `rust-analyzer` a dalších jazykových serverů v budoucnu.

## Fáze 1: MVP (Minimum Viable Product)

**Cíl:** Zprovoznit základní komunikaci s `rust-analyzer` a zobrazovat diagnostiku (chyby a varování) přímo v editoru.

### Plán implementace

1.  **Výzkum a volba knihoven:**
    *   **Stav:** Hotovo
    *   **Úkol:** Najít a vybrat nejvhodnější Rust `crates` pro LSP komunikaci (JSON-RPC, serializace, LSP typy).
    *   **Nalezené knihovny:**
        *   `lsp-client`: Pro obsluhu komunikace s LSP serverem.
        *   `lsp-types`: Pro datové typy a struktury definované ve specifikaci LSP.
        *   `tokio`: Pro asynchronní běhové prostředí.

2.  **Založení struktury:**
    *   **Stav:** Hotovo
    *   **Úkol:** Vytvořit nový modul `src/app/lsp/` pro veškerou související logiku.

3.  **Správa `rust-analyzer` procesu:**
    *   **Stav:** Hotovo
    *   **Úkol:** Implementovat spuštění `rust-analyzer` jako podprocesu a navázání komunikace přes `stdin`/`stdout`.

4.  **Základní LSP komunikace:**
    *   **Stav:** Hotovo
    *   **Úkol:** Implementovat odeslání `initialize` requestu a zpracování odpovědi.

5.  **Zpracování diagnostiky:**
    *   **Stav:** Hotovo
    *   **Úkol:** Implementovat handler pro `textDocument/publishDiagnostics` notifikace a signalizovat egui kontextu pro překreslení.

6.  **Vykreslení diagnostiky v editoru:**
    *   **Stav:** Hotovo
    *   **Úkol:** Upravit kód vykreslování editoru tak, aby podtrhával řádky s chybami a zobrazoval chybovou hlášku při najetí myší (tooltip).

## Fáze 2: Rozšířené funkce (budoucnost)

*   `textDocument/completion` (našeptávání kódu)
*   `textDocument/hover` (dokumentace po najetí)
*   `textDocument/definition` (přejít na definici)
*   `textDocument/references` (najít všechna použití)











