# Pitfalls Research

**Domain:** Save behavior and unsaved-change protection in desktop editor
**Researched:** 2026-03-09
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Silent data loss on close

**What goes wrong:**
Tab/aplikace se zavře bez potvrzení i když jsou změny v paměti.

**Why it happens:**
Close path neprovádí centralizovanou dirty kontrolu.

**How to avoid:**
Jednotný close guard se Save/Discard/Cancel a testy pro tab + app cesty.

**Warning signs:**
Uživatel hlásí „soubor se po restartu vrátil“.

**Phase to address:**
Phase 24 (Save Mode Foundation)

---

### Pitfall 2: Nekonzistentní chování auto/manual režimu

**What goes wrong:**
Některé cesty ukládají automaticky, jiné čekají na Ctrl+S bez jasné logiky.

**Why it happens:**
Save režim není single source of truth a je roztroušen v UI podmínkách.

**How to avoid:**
Persistovaný `save_mode` + centralizované rozhodnutí v save dispatcheru.

**Warning signs:**
Stejná akce vede v různých panelech k odlišnému výsledku.

**Phase to address:**
Phase 24

---

### Pitfall 3: Ctrl+S nefunguje spolehlivě při fokus změnách

**What goes wrong:**
Shortcut se ztratí při aktivním modalu nebo jiném panelu a uživatel má pocit, že save nefunguje.

**Why it happens:**
Nejednotné focus guardy nebo kolize shortcut handlerů.

**How to avoid:**
Jasná precedence shortcut handleru + explicitní testy pro modal/focus edge cases.

**Warning signs:**
Intermittent bug reporty „někdy Ctrl+S nic neudělá“.

**Phase to address:**
Phase 25 (Close Guard + UX hardening)

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Ignore save errors during close | Méně UI větvení | Tichá ztráta dat | Never |
| Duplicate close dialog code | Rychlé dodání | Regrese a nekonzistence | Never |
| Runtime-only save mode (bez persistence) | Rychlý prototyp | Po restartu jiné chování | Only for throwaway prototype |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Settings persistence | Missing serde default for new field | Add explicit default and migration-safe load |
| Tab close events | Handle only mouse close, not programmatic close | Route all close paths through same guard |
| App close hook | Ignore multi-tab dirty aggregation | Aggregate dirty state across all open tabs |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full-file rescans on each frame for dirty detection | UI micro-stutters | Maintain incremental dirty flag | Medium-large files, frequent typing |
| Save-all synchronous on app close without progress | Freeze on shutdown | Batch write + clear feedback | Many dirty tabs |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting stale path when saving tab | Write to wrong location | Validate current tab path before write |
| Ignoring I/O errors on final close | User believes save succeeded | Surface error and keep close cancellable |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Vague dialog text on close | User neví, co ztratí | Show affected file names/count |
| Hidden save mode | Uživatel nerozumí chování | Visible toggle + status indicator |

## "Looks Done But Isn't" Checklist

- [ ] **Ctrl+S:** funguje i po focus přepnutí mezi panely — verify shortcut tests.
- [ ] **Manual mode:** žádný implicitní autosave side-effect — verify save trigger matrix.
- [ ] **Close guard:** pokrývá tab close i app close — verify both paths.
- [ ] **Error path:** save failure při close nezavře app bez potvrzení — verify failure flow.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Lost unsaved changes | HIGH | Add guard immediately, add regression test, communicate behavior change |
| Inconsistent mode behavior | MEDIUM | Refactor to central save decision point and backfill tests |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Silent data loss on close | Phase 24 | Manual + unit tests for dirty close matrix |
| Inconsistent auto/manual logic | Phase 24 | Mode-switch behavioral tests |
| Ctrl+S focus regressions | Phase 25 | Shortcut integration tests |

## Sources

- Existing PolyCredo editor behavior and milestone goals
- Internal architecture constraints from `.planning/PROJECT.md`

---

# Pitfalls: Adding Theme Variants (v1.3.0)

**Domain:** Rust/eframe/egui desktop editor theme system
**Researched:** 2026-03-10
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Syntect Theme Not Mapped to Light Variants

**What goes wrong:** `syntect_theme_name()` returns `"Solarized (light)"` for ALL light variants. Syntax highlighting ignores theme choice.

**Why it happens:** In `src/settings.rs:292-298`, method only checks `dark_theme` boolean, not `light_variant`.

**Consequences:**
- CoolGray theme gets warm-toned syntax colors → visual mismatch
- Sepia theme gets yellow-heavy highlighting → poor contrast
- User perceives incomplete theme implementation

**Prevention:** Extend `syntect_theme_name()` to return variant-appropriate themes:

| LightVariant | Recommended Syntect Theme |
|--------------|---------------------------|
| WarmIvory | `"Solarized (light)"` or `"InspiredGitHub"` |
| CoolGray | `"base16-ocean.light"` |
| Sepia | `"Solarized (light)"` |

**Detection:** Run editor with each light variant, inspect code blocks. Mismatch = bug.

---

### Pitfall 2: Hardcoded Colors Override Theme

**What goes wrong:** Custom UI components use `Color32::from_rgb(r,g,b)` directly instead of theme-aware colors.

**Why it happens:** Developers copy-paste colors without using `ui.visuals()` methods.

**Consequences:**
- New dark variant: hardcoded light colors become invisible
- New light variant: hardcoded dark colors create contrast issues
- Known tech debt: "Warning text kontrast v light mode" worsens

**Prevention:**
- Always use `ui.visuals().weak_text_color()` for secondary labels
- Use `ui.visuals().panel_fill()` for custom backgrounds
- Test each component with both dark AND all light variants

**Detection:** `grep -r "Color32::from_rgb" src/app/ui/` → replace with theme-aware alternatives.

---

### Pitfall 3: Terminal/FileTree Desync on Theme Change

**What goes wrong:** Panels don't update when theme changes at runtime.

**Why it happens:** Component caches `Visuals` at construction instead of reading `ui.visuals()` each frame.

**Consequences:**
- User changes theme → editor updates, terminals stay old colors
- "Flash" of wrong colors during theme switch

**Prevention:**
- Components must receive `&egui::Visuals` reference, not own a copy
- Use `ui.visuals()` inside `paint()` or `show()` callbacks, not in `new()`
- Already solved in v1.0.2 — verify still works

**Detection:** Open terminal + file tree, switch theme → both should update instantly.

---

## Moderate Pitfalls

### Pitfall 4: Missing serde Deserialize for New Enum Variants

**What goes wrong:** Adding new `LightVariant` breaks deserialization of existing `settings.toml`.

**Why it happens:** `serde` fails on unknown enum variant by default.

**Consequences:** Settings fail to load → app crashes or resets to defaults.

**Prevention:** Ensure `#[default]` is on existing variant, test roundtrip with old settings.toml.

---

### Pitfall 5: Missing UI Strings for New Variants

**What goes wrong:** Settings dialog shows enum variant name instead of i18n string.

**Why it happens:** New `LightVariant` added but `light_variant_label_key()` not updated.

**Consequences:** UI shows `"stone"` instead of localized name.

**Prevention:** Update `light_variant_label_key()` in `src/app/ui/workspace/modal_dialogs/settings.rs` and add i18n keys in all 5 languages.

---

### Pitfall 6: Inconsistent Accent Colors

**What goes wrong:** All light variants use same accent color (hyperlinks, selections).

**Why it happens:** `to_egui_visuals()` only sets `panel_fill`, `window_fill`, `faint_bg_color`.

**Consequences:** Sepia theme with blue selection looks wrong.

**Prevention:** Extend `to_egui_visuals()` for per-variant accents (hyperlink_color, selection.bg_fill).

---

## Minor Pitfalls

### Pitfall 7: No Contrast Testing for New Themes
**New theme passes `cargo check` but fails accessibility tests.**

Prevention: Use contrast ratio tools (WebAIM), target ≥4.5:1 for normal text.

### Pitfall 8: Theme Switch Causes Focus Loss
**Changing theme causes editor to lose focus or scroll position.**

Prevention: Already handled. Verify: scroll to middle, switch theme → position preserved.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Add 4th light variant | Pitfall #1 (syntect mismatch) | Map each variant to appropriate syntect theme |
| Add 2nd dark variant | Pitfall #2 (hardcoded colors) | Audit all `Color32::from_rgb` calls |
| i18n update | Pitfall #5 (missing strings) | Add keys for all 5 languages |
| Settings migration | Pitfall #4 (deserialization) | Test roundtrip with old settings.toml |

---

## Verification Checklist

Before declaring theme work complete:

- [ ] Each light variant renders distinct panel colors
- [ ] `syntect_theme_name()` returns variant-appropriate theme
- [ ] All UI components use `ui.visuals()` not hardcoded colors
- [ ] New variant appears in Settings dialog with i18n label
- [ ] Settings.toml roundtrip works with new variant
- [ ] Contrast ratio ≥ 4.5:1 for text on panels
- [ ] Theme switch propagates to all open windows instantly
- [ ] No "flash" or layout reset during theme change

---

## Sources

- **HIGH**: egui GitHub issue #4490 — custom theme support
- **HIGH**: egui GitHub PR #4744 — `set_dark_style`/`set_light_style` API
- **MEDIUM**: syntect docs — ThemeSet, theme loading
- **MEDIUM**: WebAIM contrast guidelines — WCAG 2.1
- **MEDIUM**: PolyCredo `src/settings.rs` — existing implementation
- **MEDIUM**: PolyCredo PROJECT.md — known tech debt

---

*Pitfalls research for: adding theme variants (v1.3.0)*
*Researched: 2026-03-10*
