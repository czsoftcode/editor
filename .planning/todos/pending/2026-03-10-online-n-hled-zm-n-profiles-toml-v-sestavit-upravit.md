---
created: 2026-03-10T23:48:01.666Z
title: Online náhled změn profiles.toml v Sestavit > Upravit
area: ui
files:
  - .polycredo/profiles.toml
  - src/app/ui
---

## Problem

V workflow "Sestavit > Upravit" chybí online náhled změn, které se následně zapisují do `.polycredo/profiles.toml`. Uživatel nevidí dopad úprav před uložením, což zvyšuje riziko chybné konfigurace.

## Solution

Doplnit do UI "Sestavit > Upravit" průběžný (live) preview diff/obsah hodnot, které budou zapsány do `.polycredo/profiles.toml`.

Požadované chování:
- během editace je vidět, co se změní,
- před uložením je jasné, jaký bude výsledný zápis,
- zachovat konzistentní chování se současným save flow.
