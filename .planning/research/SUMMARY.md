# Research Summary - Milestone v1.3.0 (CLI Removal)

## Stack additions
- Zadna nova knihovna neni potreba.
- Prace je cisty codebase cleanup/refactor namespace.

## Feature table stakes
- Zachovat plne funkcni AI terminal (chat/stream/model picker/slash).
- Zachovat approval + security flow.

## Architecture direction
- Odpojit UI/state/settings od `app::cli` namespace.
- Presunout nutne casti pod AI terminal-oriented modul.

## Watch out for
- Rozbite importy v `settings.rs`, `app/types.rs`, `workspace/state` a `background.rs`.
- Regrese approval/tool flow po presunu executor logiky.
- Zapomenute testy/fixtures odkazujici na puvodni modul.
