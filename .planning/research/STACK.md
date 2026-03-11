# Milestone v1.3.0 Research - Stack

## Scope
Odebrani `src/app/cli/*` se zachovanim AI terminalu.

## Existing stack to keep
- Rust 2024
- eframe/egui UI stack
- ureq networking
- fluent i18n
- stavajici test harness (`cargo test`, `./check.sh`)

## Stack changes needed
- Zadna nova knihovna neni nutna.
- Potrebne je presunout/renamovat moduly a typy, ktere AI terminal pouziva z `app::cli` namespace.

## Integration notes
- Zachovat stejna rozhrani tam, kde jsou pouzita ve `src/app/ui/terminal/ai_chat/*`.
- Omezit zmeny na namespace + file layout + importy.

## What not to add
- Zadny async runtime navic.
- Zadny novy provider framework.
- Zadny feature expansion mimo cleanup.
