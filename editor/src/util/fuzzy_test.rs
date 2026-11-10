#[cfg(test)]
mod tests {
    use crate::util::fuzzy::fuzzy_matches;

    #[test]
    fn subsequence() {
        assert!(fuzzy_matches("ka_poster", "kapo"));
        assert!(fuzzy_matches("ka_poster", "ka_poster"));
        assert!(fuzzy_matches("ka_poster", "kpr"));
        assert!(!fuzzy_matches("ka_poster", "kapox"));
    }

    #[test]
    fn case_insensitive() {
        assert!(fuzzy_matches("Ka_Poster", "KAPO"));
        assert!(fuzzy_matches("ka_poster", "KaPo"));
    }

    #[test]
    fn ignores_query_whitespace() {
        assert!(fuzzy_matches("ka_poster", "ka poster"));
        assert!(fuzzy_matches("ka_poster", "  k a  p "));
        assert!(fuzzy_matches("ka_poster", " "));
    }

    #[test]
    fn rejects_out_of_order() {
        assert!(!fuzzy_matches("ka_poster", "poka"));
        assert!(!fuzzy_matches("ab", "ba"));
        assert!(!fuzzy_matches("aa", "aaa"));
    }
}
