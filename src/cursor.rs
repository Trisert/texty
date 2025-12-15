pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub desired_col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            line: 0,
            col: 0,
            desired_col: 0,
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.desired_col, 0);
    }

    proptest! {
        #[test]
        fn cursor_invariants(line in 0..1000usize, col in 0..1000usize, desired_col in 0..1000usize) {
            let cursor = Cursor { line, col, desired_col };
            prop_assert_eq!(cursor.line, line);
            prop_assert_eq!(cursor.col, col);
            prop_assert_eq!(cursor.desired_col, desired_col);
        }
    }
}
