use std::path::PathBuf;

use crate::app::ui::dialogs::confirm::UnsavedGuardDecision;
use crate::app::ui::editor::Editor;
use crate::app::ui::workspace::state::{DirtyCloseQueueMode, build_dirty_close_queue_for_mode};

use super::super::apply_unsaved_close_decision;
use super::super::consume_close_tab_shortcut;
use super::super::editor_input_locked;
use super::super::open_guard_queue_item_without_focus;
use super::super::process_guard_save_failure_feedback;
use super::super::should_close_tabs_after_guard_decision;
use super::super::tabbar_close_target_path;
use super::super::{PendingCloseFlow, PendingCloseMode, UnsavedCloseOutcome};

// Helper to construct a simple PendingCloseFlow for testing.
fn make_flow(queue: Vec<PathBuf>) -> PendingCloseFlow {
    PendingCloseFlow {
        mode: PendingCloseMode::SingleTab,
        queue,
        current_index: 0,
        inline_error: None,
    }
}

#[test]
fn unsaved_close_guard_modal_actions() {
    let path1 = PathBuf::from("/project/a.txt");
    let path2 = PathBuf::from("/project/b.txt");

    // Start with two items in the queue.
    let mut flow = make_flow(vec![path1.clone(), path2.clone()]);

    // Discard should advance to the next item without error.
    let outcome = apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Discard, Ok(()));
    assert_eq!(flow.current_index, 1);
    assert!(flow.inline_error.is_none());
    assert_eq!(outcome, UnsavedCloseOutcome::Continue);

    // Cancel should stop the flow without modifying the queue.
    let outcome = apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Cancel, Ok(()));
    assert_eq!(flow.current_index, 1);
    assert!(flow.inline_error.is_none());
    assert_eq!(outcome, UnsavedCloseOutcome::Cancelled);
}

#[test]
fn unsaved_close_guard_save_fail() {
    let path = PathBuf::from("/project/a.txt");
    let mut flow = make_flow(vec![path.clone()]);

    let err_msg = "save failed".to_string();

    // Save decision with an error should keep the same item and set inline error.
    let outcome =
        apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Save, Err(err_msg.clone()));

    assert_eq!(flow.current_index, 0);
    assert_eq!(flow.queue.len(), 1);
    assert_eq!(flow.queue[0], path);
    assert_eq!(flow.inline_error.as_deref(), Some(err_msg.as_str()));
    assert_eq!(outcome, UnsavedCloseOutcome::Continue);
}

#[test]
fn unsaved_close_guard_tab_triggers() {
    let path1 = PathBuf::from("/project/a.txt");
    let path2 = PathBuf::from("/project/b.txt");

    // Workspace-level guard queue should process all items until finished
    // when the user consistently chooses Save/Discard.
    let mut flow = PendingCloseFlow {
        mode: PendingCloseMode::WorkspaceClose,
        queue: vec![path1.clone(), path2.clone()],
        current_index: 0,
        inline_error: None,
    };

    // First item: successful save advances the queue.
    let outcome_1 = apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Save, Ok(()));
    assert_eq!(flow.current_index, 1);
    assert!(flow.inline_error.is_none());
    assert_eq!(outcome_1, UnsavedCloseOutcome::Continue);

    // Second item: Discard finishes the flow.
    let outcome_2 = apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Discard, Ok(()));
    assert_eq!(flow.current_index, 1);
    assert!(flow.inline_error.is_none());
    assert_eq!(outcome_2, UnsavedCloseOutcome::Finished);

    // A fresh flow: Cancel on the first item should stop the guard without
    // touching the queue indices, mirroring a user pressing Cancel in the
    // tab-close guard dialog.
    let mut flow_cancel = make_flow(vec![path1, path2]);
    let outcome_cancel =
        apply_unsaved_close_decision(&mut flow_cancel, UnsavedGuardDecision::Cancel, Ok(()));
    assert_eq!(flow_cancel.current_index, 0);
    assert!(flow_cancel.inline_error.is_none());
    assert_eq!(outcome_cancel, UnsavedCloseOutcome::Cancelled);
}

#[test]
fn unsaved_close_guard_ctrl_w_consumes_shortcut() {
    let ctx = eframe::egui::Context::default();

    ctx.begin_pass(eframe::egui::RawInput {
        events: vec![eframe::egui::Event::Key {
            key: eframe::egui::Key::W,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: eframe::egui::Modifiers::CTRL,
        }],
        ..Default::default()
    });

    // Prvni cteni shortcutu ma uspet.
    assert!(consume_close_tab_shortcut(&ctx));
    // Druhe cteni v tom samem frame uz musi byt false (shortcut je spotrebovany).
    assert!(!consume_close_tab_shortcut(&ctx));

    let _ = ctx.end_pass();
}

#[test]
fn unsaved_close_guard_input_lock() {
    assert!(editor_input_locked(false, true));
    assert!(editor_input_locked(true, false));
    assert!(editor_input_locked(true, true));
    assert!(!editor_input_locked(false, false));
}

#[test]
fn unsaved_close_guard_target_tab_from_tabbar_close() {
    let a = PathBuf::from("/project/a.txt");
    let b = PathBuf::from("/project/b.txt");
    let c = PathBuf::from("/project/c.txt");
    let tabs = vec![(a.clone(), true), (b.clone(), false), (c.clone(), true)];

    assert_eq!(tabbar_close_target_path(&tabs, 2), Some(c));
    assert_eq!(tabbar_close_target_path(&tabs, 99), None);
}

#[test]
fn unsaved_close_guard_single_tab_regressions() {
    let active = PathBuf::from("/project/active.txt");
    let non_active_dirty = PathBuf::from("/project/non-active.txt");
    let clean = PathBuf::from("/project/clean.txt");
    let tabs = vec![
        (active.clone(), true),
        (non_active_dirty.clone(), true),
        (clean, false),
    ];

    // Klik na X na neaktivnim dirty tabu musi cilit prave na vybrany tab.
    let target_from_tabbar = tabbar_close_target_path(&tabs, 1).expect("target tab");
    let queue_from_tabbar = build_dirty_close_queue_for_mode(
        DirtyCloseQueueMode::SingleTab(&target_from_tabbar),
        &tabs,
    );
    assert_eq!(queue_from_tabbar, vec![non_active_dirty.clone()]);

    // Ctrl+W nad aktivnim dirty tabem nesmi iterovat dalsi dirty taby.
    let queue_from_ctrl_w =
        build_dirty_close_queue_for_mode(DirtyCloseQueueMode::SingleTab(&active), &tabs);
    assert_eq!(queue_from_ctrl_w, vec![active.clone()]);

    let mut flow = make_flow(queue_from_ctrl_w);
    let outcome = apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Save, Ok(()));
    assert_eq!(outcome, UnsavedCloseOutcome::Finished);
}

#[test]
fn unsaved_close_guard_focus_handoff() {
    let mut editor = Editor::new();
    let first = std::env::temp_dir().join("polycredo_unsaved_guard_focus_first.txt");
    let second = std::env::temp_dir().join("polycredo_unsaved_guard_focus_second.txt");

    std::fs::write(&first, "first").expect("create first temp file");
    std::fs::write(&second, "second").expect("create second temp file");

    editor.open_file(&first);
    editor.focus_editor_requested = false;

    open_guard_queue_item_without_focus(&mut editor, &second);
    assert_eq!(editor.active_path(), Some(&second));
    assert!(
        !editor.focus_editor_requested,
        "guard flow must not request editor focus while modal is active"
    );

    let _ = std::fs::remove_file(first);
    let _ = std::fs::remove_file(second);
}

#[test]
fn unsaved_close_guard_save_failure_feedback() {
    let path = PathBuf::from("/project/save-fail.txt");
    let mut flow = make_flow(vec![path]);
    let mut toasts = Vec::new();
    let unique_err = format!(
        "save failed {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_nanos()
    );

    let outcome =
        process_guard_save_failure_feedback(&mut flow, &mut toasts, unique_err.as_str(), true);

    assert_eq!(outcome, UnsavedCloseOutcome::Continue);
    assert_eq!(flow.current_index, 0);
    assert_eq!(flow.inline_error.as_deref(), Some(unique_err.as_str()));
    assert_eq!(toasts.len(), 1);
    assert_eq!(toasts[0].message, unique_err);
    assert!(
        toasts[0].is_error,
        "save fail v guard flow musi byt surfacnuty jako error toast"
    );

    let save_result = Err(String::from("save failed"));
    assert!(
        !should_close_tabs_after_guard_decision(UnsavedGuardDecision::Save, &save_result),
        "save fail nesmi zavrit tab ani projekt bez dalsiho uzivatelskeho rozhodnuti"
    );
}
