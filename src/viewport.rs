pub struct Viewport {
    pub offset_line: usize,
    pub offset_col: usize,
    pub rows: usize,
    pub cols: usize,
}

impl Viewport {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            offset_line: 0,
            offset_col: 0,
            rows,
            cols,
        }
    }

    pub fn scroll_to_cursor(&mut self, cursor_line: usize, cursor_col: usize) {
        // Center the cursor
        self.offset_line = cursor_line.saturating_sub(self.rows / 2);
        self.offset_col = cursor_col.saturating_sub(self.cols / 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_viewport_new() {
        let viewport = Viewport::new(10, 20);
        assert_eq!(viewport.offset_line, 0);
        assert_eq!(viewport.offset_col, 0);
        assert_eq!(viewport.rows, 10);
        assert_eq!(viewport.cols, 20);
    }

    #[test]
    fn test_scroll_to_cursor_above() {
        let mut viewport = Viewport::new(10, 20);
        viewport.scroll_to_cursor(5, 10);
        assert_eq!(viewport.offset_line, 0);
        assert_eq!(viewport.offset_col, 0);
    }

    #[test]
    fn test_scroll_to_cursor_below() {
        let mut viewport = Viewport::new(10, 20);
        viewport.scroll_to_cursor(15, 10);
        assert_eq!(viewport.offset_line, 10);
        assert_eq!(viewport.offset_col, 0);
    }

    #[test]
    fn test_scroll_to_cursor_left() {
        let mut viewport = Viewport::new(10, 20);
        viewport.scroll_to_cursor(5, 5);
        assert_eq!(viewport.offset_line, 0);
        assert_eq!(viewport.offset_col, 0);
    }

    #[test]
    fn test_scroll_to_cursor_right() {
        let mut viewport = Viewport::new(10, 20);
        viewport.scroll_to_cursor(5, 25);
        assert_eq!(viewport.offset_line, 0);
        assert_eq!(viewport.offset_col, 15);
    }

    #[test]
    fn test_scroll_to_cursor_within_viewport() {
        let mut viewport = Viewport::new(10, 20);
        viewport.scroll_to_cursor(7, 10);
        assert_eq!(viewport.offset_line, 2);
        assert_eq!(viewport.offset_col, 0);
    }

    proptest! {
        #[test]
        fn viewport_scroll_invariants(rows in 1..100usize, cols in 1..100usize, cursor_line in 0..200usize, cursor_col in 0..200usize) {
            let mut viewport = Viewport::new(rows, cols);
            viewport.scroll_to_cursor(cursor_line, cursor_col);

            // Cursor should be visible
            prop_assert!(cursor_line >= viewport.offset_line && cursor_line < viewport.offset_line + viewport.rows);
            prop_assert!(cursor_col >= viewport.offset_col && cursor_col < viewport.offset_col + viewport.cols);
        }
    }
}
