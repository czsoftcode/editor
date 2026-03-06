pub mod binary;
pub mod context;
pub mod helpers;
pub mod lsp;
pub mod markdown;
pub mod normal;
pub mod tabs;

pub use helpers::{editor_line_count, goto_centered_scroll_offset, restore_saved_cursor};

#[cfg(test)]
mod tests {
    use super::helpers::goto_centered_scroll_offset;

    #[test]
    fn goto_scroll_centers_when_possible() {
        let offset = goto_centered_scroll_offset(50, 200, 20.0, 200.0);
        assert!((offset - 890.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_top() {
        let offset = goto_centered_scroll_offset(1, 200, 20.0, 200.0);
        assert!((offset - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_bottom() {
        let offset = goto_centered_scroll_offset(200, 200, 20.0, 200.0);
        assert!((offset - 3800.0).abs() < f32::EPSILON);
    }
}
