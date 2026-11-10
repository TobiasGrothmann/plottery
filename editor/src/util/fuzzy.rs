/// Whether the query's characters appear in `text` in order, ignoring case and query whitespace.
pub fn fuzzy_matches(text: &str, query: &str) -> bool {
    let mut text_chars = text.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .all(|q| text_chars.any(|t| t == q))
}
