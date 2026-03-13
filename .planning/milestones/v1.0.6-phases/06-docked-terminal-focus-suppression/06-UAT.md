---
status: diagnosed
phase: 06-docked-terminal-focus-suppression
source: 06-01-PLAN.md (no SUMMARY exists)
started: 2026-03-05T20:45:00Z
updated: 2026-03-05T20:52:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Hover nad terminalem nekrade fokus
expected: Prejizdi mysi pres pravy i spodni terminal — fokus se NEMENI, klavesovy vstup zustava v puvodnim panelu.
result: pass

### 2. Modal blokuje terminal fokus (FSUP-01)
expected: Otevri Settings (Ctrl+,) nebo About. Klikni do terminalu (praveho i spodniho). Terminal NEPREVEZME fokus — modal zustava aktivni. Hover nad terminalem taky nema efekt.
result: issue
reported: "Pri kliknuti u nastaveni mimo modal, ktery neni editovany, se modal zavre, aniz bych mel moznost v nem neco psat. chce to interaktivny modaly nechat reagovat jenom na Ulozit, Storno"
severity: major

### 3. Modal overlay blokuje interakci za modalem
expected: Otevri Settings nebo About. Za modalem je tmave prekryti (overlay). Kliknuti mimo modal (na overlay) zavre modal. Nelze interagovat s editorem/terminalem za modalem.
result: pass

### 4. AI Chat drzi fokus (FSUP-02 + AICF-01 + AICF-02)
expected: Otevri AI Chat (Ctrl+Alt+A). Zacni psat text do TextEdit. Prejizdi mysi pres terminaly — text stale jde do AI Chatu, NE do terminalu. Fokus zustava v AI Chatu.
result: pass

### 5. Terminal funguje normalne bez modalu
expected: Zavri vsechny modaly a AI Chat. Klikni do terminalu. Terminal PREVEZME fokus normalne — klavesovy vstup jde do terminalu. Ctrl+Alt+B/A prepina fokus spravne.
result: pass

### 6. Settings — discard potvrzeni pri neulozenych zmenach
expected: Otevri Settings, zmen neco (napr. font size). Klikni na X nebo mimo modal (na overlay). Zobrazi se potvrzeni "zahodit zmeny?" misto okamziteho zavreni.
result: pass

## Summary

total: 6
passed: 5
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Interaktivni modaly (Settings) by se nemely zavirat kliknutim mimo modal — pouze pres Ulozit/Storno"
  status: failed
  reason: "User reported: Pri kliknuti u nastaveni mimo modal, ktery neni editovany, se modal zavre, aniz bych mel moznost v nem neco psat. chce to interaktivny modaly nechat reagovat jenom na Ulozit, Storno"
  severity: major
  test: 2
  root_cause: "StandardModal::show() radky 99-111 v modal.rs — kliknuti mimo okno vzdy zavre modal. Chybi rozliseni mezi interaktivnimi modaly (Settings, Plugins) a informacnimi (About). Interaktivni modaly by nemely reagovat na click-outside."
  artifacts:
    - path: "src/app/ui/widgets/modal.rs"
      issue: "show() metoda ma hardcoded click-outside-to-close pro vsechny modaly"
  missing:
    - "Pridat builder metodu close_on_click_outside(bool) na StandardModal"
    - "Settings modal pouzije close_on_click_outside(false)"
    - "About a dalsi info modaly zustanou s close_on_click_outside(true)"
