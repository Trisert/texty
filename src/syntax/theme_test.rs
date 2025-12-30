#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        let color = Theme::hex_to_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }
}
