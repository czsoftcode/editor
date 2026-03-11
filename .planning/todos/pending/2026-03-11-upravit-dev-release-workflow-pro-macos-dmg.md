---
created: 2026-03-11T00:46:13.471Z
title: Upravit dev-release workflow pro macOS dmg
area: tooling
files:
  - .github/workflows/dev-release.yml
---

## Problem

Dev-release workflow na macOS zatim publikuje .tar.gz, ale potrebujeme DMG balicek pro uzivatele macOS.

## Solution

Upravit `.github/workflows/dev-release.yml` tak, aby macOS build artefakty byly DMG (ne tar.gz). Projit balici krok a vystupni artefakty, pripadne doplnit/změnit packaging krok pro DMG.
