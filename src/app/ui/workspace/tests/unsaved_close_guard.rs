use std::path::PathBuf;

use crate::app::ui::dialogs::confirm::UnsavedGuardDecision;

use super::super::{PendingCloseFlow, PendingCloseMode, UnsavedCloseOutcome};
use super::super::apply_unsaved_close_decision;
use super::super::consume_close_tab_shortcut;

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
    let outcome =
        apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Cancel, Ok(()));
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
    let mut flow = make_flow(vec![path1.clone(), path2.clone()]);

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
