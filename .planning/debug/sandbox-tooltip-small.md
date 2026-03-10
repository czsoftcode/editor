---
status: diagnosed
trigger: "Investigate issue: UAT-04-04 — Tooltip/poznámka u sandbox přepínače je příliš malá a snadno přehlédnutelná."
created: 2026-03-05T03:47:33Z
updated: 2026-03-05T03:49:12Z
---

## Current Focus

hypothesis: Potvrzeno: problém je vizuální de-emphasis a malá hover oblast tooltipu.
test: Diagnóza uzavřena na základě render kódu + i18n verifikace.
expecting: Náprava vyžaduje změnu vizuální prominence (větší/kontrastnější hint + větší hover target).
next_action: Vrátit strukturovanou diagnózu (root cause only).

## Symptoms

expected: Tooltip s vysvětlením OFF režimu a inline poznámka jsou jasně viditelné a upoutají pozornost.
actual: "popis je small, vubec jsem si ho nevsiml"
errors: None reported
reproduction: Test 4 in UAT (Settings > Projekt, hover u přepínače)
started: Discovered during UAT

## Eliminated

## Evidence

- timestamp: 2026-03-05T03:47:55Z
  checked: "Vyhledání výskytů sandbox tooltipu přes `rg`"
  found: "Kritický blok je v `src/app/ui/workspace/modal_dialogs/settings.rs:271-284`, kde je `RichText::new(\"ℹ\").small().weak()` a inline text používá `.small().weak()`."
  implication: "UI je navrženo s nízkou vizuální prominencí; to může přímo vysvětlovat, proč si uživatel popisu nevšiml."

- timestamp: 2026-03-05T03:48:18Z
  checked: "Kompletní čtení `src/app/ui/workspace/modal_dialogs/settings.rs`"
  found: "Přepínač sandbox režimu je běžný checkbox; tooltip je navázán výhradně na samostatný label s jedním znakem `ℹ`, navíc `small().weak()`. Obě inline poznámky (`settings-safe-mode-hint`, `settings-safe-mode-terminal-note`) jsou také `small().weak()`."
  implication: "Není tam žádný alternativní výrazný hint (barva, ikonka s pozadím, heading, větší písmo). UX je konzistentně de-emphasized."

- timestamp: 2026-03-05T03:48:43Z
  checked: "Vyhledání i18n klíčů `settings-safe-mode-*` v `locales/*.ftl`"
  found: "V `cs` a `en` existují plné texty tooltipu i obou inline poznámek; nejsou prázdné. (Vedlejší nález: ru/sk/de mají odlišný rozsah klíčů.)"
  implication: "Pro reported UAT symptom (`popis je small`) není primární příčina v obsahu textu, ale ve způsobu renderu."

- timestamp: 2026-03-05T03:49:12Z
  checked: "Obsah klíčů v `locales/en/ui.ftl` a `locales/cs/ui.ftl`"
  found: "Tooltip i obě poznámky mají explicitní text s informací o OFF režimu a reopen efektu."
  implication: "Text existuje a je věcně správný; symptom je způsoben tím, jak je text vizuálně prezentován."

## Resolution

root_cause: "V `src/app/ui/workspace/modal_dialogs/settings.rs` je pomocná informace renderována nízko-kontrastně a malým fontem (`small().weak()`), zatímco tooltip je dostupný pouze přes malý jednoznakový hover target (`ℹ`). Kombinace de-emphasis + malá interakční plocha vede k tomu, že uživatel popis snadno přehlédne."
fix: ""
verification: "Kořen potvrzen čtením render kódu a ověřením, že texty v i18n (cs/en) existují."
files_changed: []
