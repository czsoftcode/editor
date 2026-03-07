pub mod frontmatter;
pub mod config;
pub mod paths;

use std::path::Path;

use super::slash::SlashResult;
use crate::app::ui::workspace::state::WorkspaceState;

struct GsdSubcommand {
    name: &'static str,
    description: &'static str,
}

const GSD_SUBCOMMANDS: &[GsdSubcommand] = &[
    GsdSubcommand { name: "state", description: "Show or update project state" },
    GsdSubcommand { name: "progress", description: "Show progress bar and phase summary" },
    GsdSubcommand { name: "config", description: "Get or set GSD configuration" },
    GsdSubcommand { name: "help", description: "Show GSD subcommands" },
];

/// Main GSD dispatch — called from slash.rs with everything after `/gsd `.
pub fn cmd_gsd(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    todo!()
}

/// Returns matching GSD subcommands for autocomplete.
pub fn matching_subcommands(filter: &str) -> Vec<(&'static str, &'static str)> {
    todo!()
}

fn check_planning_dir(root: &Path) -> Option<SlashResult> {
    todo!()
}

fn cmd_gsd_help() -> SlashResult {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_gsd_help_empty_args() {
        // cmd_gsd("", ws) should return help text
        let result = cmd_gsd_help();
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("state"), "Help should list 'state' subcommand");
                assert!(text.contains("progress"), "Help should list 'progress' subcommand");
                assert!(text.contains("config"), "Help should list 'config' subcommand");
                assert!(text.contains("help"), "Help should list 'help' subcommand");
            }
            _ => panic!("Expected Immediate result from gsd help"),
        }
    }

    #[test]
    fn test_check_planning_dir_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let result = check_planning_dir(tmp.path());
        assert!(result.is_some(), "Should return error when .planning/ missing");
        match result.unwrap() {
            SlashResult::Immediate(text) => {
                assert!(text.contains(".planning"), "Should mention .planning directory");
            }
            _ => panic!("Expected Immediate result"),
        }
    }

    #[test]
    fn test_check_planning_dir_exists() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".planning")).unwrap();
        let result = check_planning_dir(tmp.path());
        assert!(result.is_none(), "Should return None when .planning/ exists");
    }

    #[test]
    fn test_matching_subcommands_all() {
        let all = matching_subcommands("");
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_matching_subcommands_filter() {
        let filtered = matching_subcommands("st");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "state");
    }

    #[test]
    fn test_matching_subcommands_no_match() {
        let none = matching_subcommands("xyz");
        assert!(none.is_empty());
    }
}
