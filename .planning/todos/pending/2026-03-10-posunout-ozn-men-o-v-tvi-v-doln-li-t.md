---
created: 2026-03-10T23:40:59.973Z
title: Posunout oznámení o větvi v dolní liště
area: ui
files:
  - src/app/ui/terminal/bottom/git_bar.rs
  - src/app/ui/terminal/bottom/mod.rs
---

## Problem

V dolní liště je oznámení o větvi špatně vyvážené v layoutu: label větve je potřeba posunout více doprava a text/indikaci UTF více doleva, aby rozložení nebylo opticky stlačené a bylo lépe čitelné.

## Solution

Upravit layout a zarovnání prvků v dolní liště terminálu:
- branch oznámení posunout doprava,
- UTF indikaci posunout více doleva,
- zachovat čitelnost v light/dark variantách.

Doplnit rychlou regresní kontrolu layoutu (pokud existuje test harness pro UI), jinak manuální ověření v obou režimech.
