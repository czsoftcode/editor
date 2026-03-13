# Quick Plan: Fix Windows build (localtime) a warningy

## Cíl
Opravit chybu kompilace na Windows (`localtime_r` není dostupné) a odstranit varování o nepoužitých proměnných, aby GH Actions pro `.exe` a `.msi` prošly bez errorů.

## Tasky

1. Opravit timestamp v chat konverzaci pro Windows
- V `src/app/ui/widgets/ai/chat/conversation.rs` použít `localtime_s` na Windows a `localtime_r` na ostatních platformách pomocí `cfg`.

2. Uklidit varování `unused variable`
- V `src/app/fonts.rs` inicializovat `home` jen pro Linux/macOS, aby na Windows nebyl nepoužitý.
- V `src/app/ui/workspace/modal_dialogs/terminal.rs` změnit `terminal` na `_terminal` (nebo upravit `cfg`), aby nevznikalo warning na Windows.

3. Validace
- Spustit `cargo check` a `./check.sh`.
- Zapsat výsledky do quick summary.
