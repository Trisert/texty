#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_variants() {
        assert_eq!(Mode::Normal, Mode::Normal);
        assert_ne!(Mode::Normal, Mode::Insert);
        assert_eq!(Mode::Visual, Mode::Visual);
        assert_eq!(Mode::Command, Mode::Command);
    }

    #[test]
    fn test_mode_clone() {
        let mode = Mode::Insert;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }
}
