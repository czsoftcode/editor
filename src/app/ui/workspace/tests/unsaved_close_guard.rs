use std::path::PathBuf;

use crate::app::ui::dialogs::confirm::UnsavedGuardDecision;

use super::super::{PendingCloseFlow, PendingCloseMode, UnsavedCloseOutcome};
use super::super::apply_unsaved_close_decision;

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
    let outcome =
        apply_unsaved_close_decision(&mut flow, UnsavedGuardDecision::Discard, Ok(()));
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
    let outcome = apply_unsaved_close_decision(
        &mut flow,
        UnsavedGuardDecision::Save,
        Err(err_msg.clone()),
    );

    assert_eq!(flow.current_index, 0);
    assert_eq!(flow.queue.len(), 1);
    assert_eq!(flow.queue[0], path);
    assert_eq!(flow.inline_error.as_deref(), Some(err_msg.as_str()));
    assert_eq!(outcome, UnsavedCloseOutcome::Continue);
}

