/// Check if `pos`ition is within first word in `text`.
pub fn in_first_word(pos: usize, text: &str) -> bool {
    if let Some(wpos) = text.find(char::is_whitespace) {
        return pos < wpos;
    }
    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_first_word_beginning() {
        assert!(in_first_word(0, "hello world"));
    }

    #[test]
    fn in_first_word_middle() {
        assert!(in_first_word(2, "hello world"));
    }

    #[test]
    fn in_first_word_end() {
        assert!(in_first_word(4, "hello world"));
    }

    #[test]
    fn in_first_word_on_boundary_whitespace() {
        assert!(!in_first_word(5, "hello world"));
    }

    #[test]
    fn in_first_word_next_word() {
        assert!(!in_first_word(6, "hello world"));
    }
}
