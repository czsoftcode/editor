---
status: complete
phase: 03-light-varianty-settings-ui
source: [03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md, 03-04-SUMMARY.md]
started: 2026-03-04T23:46:25Z
updated: 2026-03-04T23:55:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Přepnutí dark/light módu
expected: Otevři Settings modal. Je tam přepínač dark/light tématu. Po přepnutí do Light mode se celé UI okamžitě přepne na světlé téma — ještě v otevřeném modalu. Po přepnutí zpět na Dark mode se UI okamžitě vrátí na tmavé.
result: pass

### 2. Picker karet light variant (viditelnost)
expected: V Settings modalu se světlým tématem jsou vidět 3 klikatelné karty variant: WarmIvory, CoolGray, Sepia. Každá karta má barevný vzorek a popisek. V dark mode nejsou karty viditelné (sekce picker zmizí).
result: issue
reported: "je tam jenom slonova kost, ostatni dve chyby"
severity: major

### 3. Live preview varianty
expected: Kliknutí na jinou kartu varianty (např. z WarmIvory na Sepia) okamžitě změní pozadí editoru/UI na teplou hnědavou paletu — bez zavření modalu, změna je vidět hned.
result: skipped
reason: pouze WarmIvory karta dostupná (CoolGray a Sepia chybí)

### 4. Save persistuje variantu
expected: Zvol variantu CoolGray v Settings a klikni Save. Modal se zavře. Zavři a znovu spusť editor. Po restartu se editor otevře s CoolGray light variantou (chladné šedavé pozadí).
result: pass

### 5. Cancel reverts preview
expected: V Settings modalu (light mode) přepni na jinou variantu než aktuální — live preview se aplikuje. Klikni Cancel. UI se vrátí na původní variantu (preview se zruší), modal se zavře.
result: pass

### 6. Odlišnost variant terminálu
expected: V light mode otevři terminál. Přepínej mezi třemi variantami (WarmIvory / CoolGray / Sepia) a pokaždé ulož. Pozadí terminálu by mělo být viditelně odlišné: WarmIvory = teplý krémový tón, CoolGray = chladný šedavý, Sepia = hnědavý.
result: issue
reported: "terminal pri light moc sviti, ale mam jenom slonovou kost. bylo by dobre, kdyby terminal byl ve slonove kosti - jine tema nemam, tak nevim"
severity: minor

### 7. Git statusy v file tree (light mode)
expected: Otevři projekt s git repozitářem v light mode. V file tree jsou soubory obarveny podle git stavu: Modified = zlatavá, Added = zelená, Untracked = šedá, Deleted = červená. Barvy jsou čitelné na světlém pozadí (ne příliš světlé, ne tmavé).
result: pass

### 8. Dark mode stabilita po přepnutí
expected: Přepni do light mode, zvol variantu, pak přepni zpět na dark mode. V dark mode jsou barvy terminálu a git statusů v file tree stejné jako před přepnutím — žádné vizuální artefakty ani přebarvení.
result: pass

## Summary

total: 8
passed: 5
issues: 2
pending: 0
skipped: 1

## Gaps

- truth: "V Settings modalu (light mode) jsou vidět 3 klikatelné karty variant: WarmIvory, CoolGray, Sepia"
  status: failed
  reason: "User reported: je tam jenom slonova kost, ostatni dve chyby"
  severity: major
  test: 2
  root_cause: "show_light_variant_card() volá ui.with_layout(right_to_left) uvnitř ui.horizontal — spotřebuje veškerou zbývající šířku, karta expanduje na full-width. V horizontal_wrapped se každá karta zabalí na vlastní řádek a karty 2 a 3 jsou mimo viditelný viewport modalu."
  artifacts:
    - path: "src/app/ui/workspace/modal_dialogs/settings.rs"
      issue: "ui.with_layout(right_to_left) uvnitř ui.horizontal způsobuje full-width expanzi karty"
  missing:
    - "Nahradit ui.with_layout(right_to_left) přímým podmíněným ui.label(checkmark) bez expanze"

- truth: "Terminál v light mode WarmIvory variantě má teplý krémový (ivory) tón pozadí"
  status: failed
  reason: "User reported: terminal pri light moc sviti, ale mam jenom slonovou kost. bylo by dobre, kdyby terminal byl ve slonove kosti - jine tema nemam, tak nevim"
  severity: minor
  test: 6
  root_cause: "panel_fill WarmIvory=(255,252,240) je téměř bílý — blend s base #f3f5f7 (t=0.42) dává (248,248,244), téměř bílou. Base je studená (vyšší B kanál) a blend s téměř bílým tónem nezvrátí studenost na teplý ivory tón."
  artifacts:
    - path: "src/app/ui/terminal/instance/theme.rs"
      issue: "blend ratio t=0.42 pro background nestačí při téměř bílém panel_fill tónu"
    - path: "src/app/ui/terminal/instance/theme.rs"
      issue: "base background #f3f5f7 je studená neutrální šeď — nevhodná pro teplé tónování"
  missing:
    - "Pro WarmIvory použít teplejší base background (např. #f5f2e8) nebo zvýšit blend ratio na 0.65+"
