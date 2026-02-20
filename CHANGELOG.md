# Changelog

Všechny významné změny v projektu PolyCredo Editor budou zaznamenány v tomto souboru.

## [0.4.0] - 2026-02-20

### Přidáno
- **CI/CD Quality Gate**: Zavedeny automatizované kontroly kvality kódu (formátování, clippy, testy) pomocí GitHub Actions a lokálního skriptu `check.sh`.
- **Sdílený souborový index**: Implementován `ProjectIndex` pro asynchronní a inkrementální indexování souborů v projektu. Sjednocuje data pro Ctrl+P, globální vyhledávání i strom souborů.
- **Command Palette (Ctrl+Shift+P)**: Přidána centrální nabídka příkazů s podporou i18n pro rychlé ovládání editoru klávesnicí.
- **Rychlé otevírání souborů (Ctrl+P)**: Implementováno fuzzy vyhledávání souborů s automatickým scrollováním na vybranou položku.

### Opraveno
- **Scrollování v Ctrl+P**: Opravena chyba, kdy vybraná položka mizela mimo viditelnou oblast seznamu při navigaci šipkami.
- **Výkon vyhledávání**: Globální vyhledávání (`Ctrl+Shift+F`) nyní využívá sdílený index místo opakovaného procházení disku.
