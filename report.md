Opraveno. Soubory v sandboxu se nyní automaticky přeindexují po každém uložení (resp. po detekci změny souborovým systémem).

Změny:
1. Upraven `SemanticIndex` (src/app/ui/workspace/semantic_index.rs) tak, aby umožňoval thread-safe aktualizaci jednotlivých souborů bez blokování celého UI během výpočtu embeddingů (použití `IndexingContext` s `Arc` pointery na model).
2. Upraven `background.rs` (src/app/ui/background.rs), kde se nyní naslouchá na `FsChange::Created`, `Modified` a `Removed` v sandboxu a spouští se asynchronní aktualizace indexu.

AI agent v RAG (chat) by měl nyní vidět změny v kódu téměř okamžitě po uložení souboru.