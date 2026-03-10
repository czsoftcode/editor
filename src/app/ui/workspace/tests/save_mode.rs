use crate::settings::SaveMode;
use crate::app::ui::editor::render::tabs::tab_label_with_mode_indicator_for_tests;
use crate::i18n::{I18n, SUPPORTED_LANGS};

use super::super::status_bar_runtime_mode_key;
use super::super::status_bar_save_mode_key_for_runtime;

#[test]
fn mode_04_runtime_visibility_manual_mode_key_is_explicit() {
    assert_eq!(
        status_bar_runtime_mode_key(&SaveMode::Manual),
        "statusbar-save-mode-manual"
    );
}

#[test]
fn mode_04_runtime_visibility_auto_mode_key_is_explicit() {
    assert_eq!(
        status_bar_runtime_mode_key(&SaveMode::Automatic),
        "statusbar-save-mode-automatic"
    );
}

#[test]
fn mode_04_runtime_visibility_settings_draft_does_not_override_runtime_key() {
    let runtime_mode = SaveMode::Manual;
    let settings_draft_mode = Some(&SaveMode::Automatic);
    assert_eq!(
        status_bar_save_mode_key_for_runtime(&runtime_mode, settings_draft_mode),
        "statusbar-save-mode-manual"
    );
}

#[test]
fn mode_04_runtime_visibility_updates_immediately_after_apply() {
    let runtime_mode_after_apply = SaveMode::Automatic;
    assert_eq!(
        status_bar_runtime_mode_key(&runtime_mode_after_apply),
        "statusbar-save-mode-automatic"
    );
}

#[test]
fn save_ux_contrast_regression_mode_key_branches_cover_manual_and_auto() {
    assert_eq!(
        status_bar_runtime_mode_key(&SaveMode::Manual),
        "statusbar-save-mode-manual"
    );
    assert_eq!(
        status_bar_runtime_mode_key(&SaveMode::Automatic),
        "statusbar-save-mode-automatic"
    );
}

#[test]
fn save_ux_contrast_regression_dirty_marker_stays_primary_over_mode_badge() {
    let label =
        tab_label_with_mode_indicator_for_tests("main.rs", true, false, true, &SaveMode::Manual);
    assert!(label.contains("● ·M"));
}

#[test]
fn save_ux_contrast_regression_mode_badge_is_hidden_for_inactive_tab() {
    let inactive_label =
        tab_label_with_mode_indicator_for_tests("lib.rs", true, false, false, &SaveMode::Manual);
    assert!(inactive_label.contains("●"));
    assert!(!inactive_label.contains("·M"));
}

#[test]
fn save_ux_i18n_smoke() {
    for &lang in SUPPORTED_LANGS {
        let i18n = I18n::new(lang);
        for &key in crate::i18n::phase_26_save_ux_keys() {
            assert_ne!(
                i18n.get(key),
                key,
                "chybi save UX i18n klic '{key}' v jazyce '{lang}'"
            );
        }
    }
}
