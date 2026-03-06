Oprava automatické indexace souborů pro RAG.

Byl zjištěn problém, kdy se RAG index aktualizoval pouze při startu aplikace. To znamenalo, že AI agent neměl přístup k nejnovějším změnám v kódu.

Řešení:
1. Upraven `src/app/ui/workspace/semantic_index.rs`:
   - Přidána struktura `IndexingContext`, která drží `Arc` pointery na model a tokenizér. To umožňuje provádět výpočty embeddingů v jiném vlákně bez nutnosti držet zámek na hlavním `SemanticIndex`.
   - Implementována funkce `compute_snippets_for_file`, která vezme kontext a obsah souboru, vypočítá embeddingy a vrátí seznam snippetů.
   - Implementována metoda `update_snippets_for_file`, která rychle (s krátkým držením zámku) aktualizuje index o nové snippety.

2. Upraven `src/app/ui/background.rs`:
   - V `process_background_events` (uvnitř smyčky `ws.project_watcher.poll()`) přidána detekce změn souborů v sandboxu (`Created`, `Modified`, `Removed`).
   - Při detekci změny se nyní:
     a) Získá `IndexingContext` (krátký zámek).
     b) Spustí nové vlákno.
     c) Ve vlákně se načte obsah souboru a vypočítají embeddingy (dlouhá operace, bez zámku).
     d) Výsledek se uloží do indexu (krátký zámek).

Tímto je zajištěno, že AI má vždy aktuální kontext, aniž by docházelo k zasekávání UI při ukládání souborů.
