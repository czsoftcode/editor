---
created: 2026-03-11T00:01:32.176Z
title: Zprovoznit .polycredo/trash pro smazane soubory
area: general
files:
  - .polycredo/trash
---

## Problem

Pri mazani souboru se dnes zrejme provadi tvrde smazani. Potrebujeme zavadit interni kos, aby bylo mozne omylem smazane soubory snadno obnovit a aby se mazani nedelo destruktivne.

## Solution

Vytvorit a zprovoznit adresar `.polycredo/trash` a zmenit logiku mazani tak, aby se soubory presouvaly do tohoto adresare. Doplnit udrzbu trash (napr. limit nebo rucni vycisteni) podle existujicich konvenci v projektu.
