---
created: 2026-03-12T10:32:39.169Z
title: Opravit ztratu obsahu pri kopii souboru
area: ui
files:
  - src/app/ui/file_tree/ops.rs
  - src/app/ui/file_tree/mod.rs
  - tests/
---

## Problem

Pri kopirovani souboru do jine slozky ve file tree se po vlozeni ztrati obsah souboru. Uzivatel vidi zkopirovany soubor, ale data nejsou zachovana.

## Solution

Prověřit copy/paste tok v `FileTree::handle_action` (vetve `Copy`/`Paste`) a doplnit regresni test, ktery overi, ze zkopirovany soubor zachova puvodni obsah bajtove shodne. Pri oprave zachovat existujici chovani pro adresare a error toast pipeline.
