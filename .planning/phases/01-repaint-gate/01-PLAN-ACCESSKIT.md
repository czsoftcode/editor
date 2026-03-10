# Plan 01: Disable Accesskit

**Objective:** Disable the `accesskit` feature in `eframe` to reduce unnecessary background processing and potential repaint triggers.

**Context:**
- `accesskit` is enabled by default in `eframe` 0.31.
- RPNT-02 directive requires disabling it.

**Tasks:**
1. Modify `Cargo.toml`:
   - Change `eframe` dependency to:
     ```toml
     eframe = { version = "0.31", default-features = false, features = ["persistence", "default_fonts", "glow", "wayland", "x11"] }
     ```
   - *Note:* Verified default features of `eframe` 0.31 include `glow`, `wayland`, `x11`, `default_fonts`, `accesskit`, and `web_screen_reader`. We are keeping everything except `accesskit` and `web_screen_reader`.

**Verification:**
- `cargo check` to ensure no compilation errors.
- `cargo tree -p eframe` to verify `accesskit` is no longer a feature.
