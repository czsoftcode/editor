use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GitVisualStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}

pub fn parse_porcelain_status(x: char, y: char) -> GitVisualStatus {
    if x == '?' && y == '?' {
        GitVisualStatus::Untracked
    } else if x == 'D' || y == 'D' {
        GitVisualStatus::Deleted
    } else if matches!(x, 'A' | 'C') || y == 'A' {
        GitVisualStatus::Added
    } else {
        GitVisualStatus::Modified
    }
}

pub fn git_color_for_mode(status: GitVisualStatus, dark_mode: bool) -> egui::Color32 {
    match (status, dark_mode) {
        (GitVisualStatus::Modified, true) => egui::Color32::from_rgb(220, 180, 60),
        (GitVisualStatus::Added, true) => egui::Color32::from_rgb(100, 200, 110),
        (GitVisualStatus::Deleted, true) => egui::Color32::from_rgb(210, 80, 80),
        (GitVisualStatus::Untracked, true) => egui::Color32::from_rgb(120, 190, 255),
        (GitVisualStatus::Modified, false) => egui::Color32::from_rgb(144, 104, 20),
        (GitVisualStatus::Added, false) => egui::Color32::from_rgb(28, 128, 58),
        (GitVisualStatus::Deleted, false) => egui::Color32::from_rgb(165, 42, 42),
        (GitVisualStatus::Untracked, false) => egui::Color32::from_rgb(0, 118, 172),
    }
}

fn mix_channel(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    ((a as f32) * (1.0 - t) + (b as f32) * t).round() as u8
}

fn mix_color(base: egui::Color32, target: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        mix_channel(base.r(), target.r(), t),
        mix_channel(base.g(), target.g(), t),
        mix_channel(base.b(), target.b(), t),
    )
}

pub fn git_color_for_visuals(status: GitVisualStatus, visuals: &egui::Visuals) -> egui::Color32 {
    if visuals.dark_mode {
        return git_color_for_mode(status, true);
    }

    let base = git_color_for_mode(status, false);
    let panel_toned = match status {
        GitVisualStatus::Modified => mix_color(base, visuals.panel_fill, 0.2),
        GitVisualStatus::Added => mix_color(base, visuals.panel_fill, 0.18),
        GitVisualStatus::Deleted => mix_color(base, visuals.panel_fill, 0.16),
        GitVisualStatus::Untracked => mix_color(base, visuals.panel_fill, 0.22),
    };
    mix_color(panel_toned, visuals.faint_bg_color, 0.06)
}

#[cfg(test)]
mod tests {
    use super::{
        GitVisualStatus, git_color_for_mode, git_color_for_visuals, parse_porcelain_status,
    };
    use crate::settings::{LightVariant, Settings};
    use eframe::egui::Visuals;
    use std::collections::HashSet;

    fn light_visuals(variant: LightVariant) -> eframe::egui::Visuals {
        Settings {
            dark_theme: false,
            light_variant: variant,
            ..Default::default()
        }
        .to_egui_visuals()
    }

    #[test]
    fn file_tree_git_parse_question_question_maps_to_untracked() {
        assert_eq!(parse_porcelain_status('?', '?'), GitVisualStatus::Untracked);
    }

    #[test]
    fn file_tree_git_parse_added_maps_to_added() {
        assert_eq!(parse_porcelain_status('A', ' '), GitVisualStatus::Added);
    }

    #[test]
    fn file_tree_git_parse_deleted_in_worktree_maps_to_deleted() {
        assert_eq!(parse_porcelain_status(' ', 'D'), GitVisualStatus::Deleted);
    }

    #[test]
    fn file_tree_git_parse_modified_maps_to_modified() {
        assert_eq!(parse_porcelain_status('M', ' '), GitVisualStatus::Modified);
    }

    #[test]
    fn file_tree_git_parse_rename_modified_maps_to_modified() {
        assert_eq!(parse_porcelain_status('R', 'M'), GitVisualStatus::Modified);
    }

    #[test]
    fn file_tree_git_light_untracked_is_distinct_from_modified_and_added() {
        let light_untracked = git_color_for_mode(GitVisualStatus::Untracked, false);
        let light_modified = git_color_for_mode(GitVisualStatus::Modified, false);
        let light_added = git_color_for_mode(GitVisualStatus::Added, false);

        assert_ne!(light_untracked, light_modified);
        assert_ne!(light_untracked, light_added);
    }

    #[test]
    fn file_tree_git_dark_light_palette_is_not_identical_for_statuses() {
        let statuses = [
            GitVisualStatus::Modified,
            GitVisualStatus::Added,
            GitVisualStatus::Deleted,
            GitVisualStatus::Untracked,
        ];

        for status in statuses {
            assert_ne!(
                git_color_for_mode(status, true),
                git_color_for_mode(status, false)
            );
        }
    }

    #[test]
    fn file_tree_git_light_variant_modified_tone_differs_between_warm_and_cool() {
        let warm = git_color_for_visuals(
            GitVisualStatus::Modified,
            &light_visuals(LightVariant::WarmIvory),
        );
        let cool = git_color_for_visuals(
            GitVisualStatus::Modified,
            &light_visuals(LightVariant::CoolGray),
        );

        assert_ne!(
            warm, cool,
            "light variants must produce different modified tone"
        );
    }

    #[test]
    fn file_tree_git_light_variant_untracked_tone_differs_between_cool_and_sepia() {
        let cool = git_color_for_visuals(
            GitVisualStatus::Untracked,
            &light_visuals(LightVariant::CoolGray),
        );
        let sepia = git_color_for_visuals(
            GitVisualStatus::Untracked,
            &light_visuals(LightVariant::Sepia),
        );

        assert_ne!(
            cool, sepia,
            "light variants must produce different untracked tone"
        );
    }

    #[test]
    fn file_tree_git_light_variant_modified_tone_differs_across_all_three() {
        let colors: HashSet<eframe::egui::Color32> = [
            LightVariant::WarmIvory,
            LightVariant::CoolGray,
            LightVariant::Sepia,
        ]
        .into_iter()
        .map(|variant| git_color_for_visuals(GitVisualStatus::Modified, &light_visuals(variant)))
        .collect();

        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn file_tree_git_dark_visuals_match_legacy_dark_palette() {
        let dark_visuals = Visuals::dark();
        let statuses = [
            GitVisualStatus::Modified,
            GitVisualStatus::Added,
            GitVisualStatus::Deleted,
            GitVisualStatus::Untracked,
        ];

        for status in statuses {
            assert_eq!(
                git_color_for_visuals(status, &dark_visuals),
                git_color_for_mode(status, true)
            );
        }
    }
}
