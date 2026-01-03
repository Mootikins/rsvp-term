/// Calculate the Optimal Recognition Point for a word.
/// Returns the 0-indexed position of the character to highlight.
///
/// ORP is typically about 1/3 into the word, where the eye naturally focuses.
/// For Spritz-style RSVP display, this letter is highlighted and the word
/// is centered around it.
///
/// Algorithm (based on alphabetic characters only):
/// - 1-3 chars: position 0 (first letter)
/// - 4-6 chars: position 1 (second letter)
/// - 7-9 chars: position 2 (third letter)
/// - 10+ chars: position 3 (fourth letter)
///
/// Leading punctuation is skipped so the ORP falls on actual letters.
/// Uses `.chars().count()` for correct Unicode handling.
pub fn calculate_orp(word: &str) -> usize {
    // Find leading punctuation to skip
    let leading_punct: usize = word
        .chars()
        .take_while(|c| !c.is_alphabetic())
        .count();

    // Calculate ORP based on alphabetic content length
    let alpha_len: usize = word.chars().filter(|c| c.is_alphabetic()).count();

    let orp_offset = match alpha_len {
        0..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    };

    // Return position accounting for leading punctuation
    leading_punct + orp_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orp_boundary_cases() {
        assert_eq!(calculate_orp("abc"), 0);    // 3 -> 0
        assert_eq!(calculate_orp("abcd"), 1);   // 4 -> 1
        assert_eq!(calculate_orp("abcdef"), 1); // 6 -> 1
        assert_eq!(calculate_orp("abcdefg"), 2); // 7 -> 2
    }

    #[test]
    fn test_orp_with_leading_punctuation() {
        assert_eq!(calculate_orp("(as"), 1);     // skip '(', 'as' is 2 chars -> offset 0, result 1
        assert_eq!(calculate_orp("\"hello"), 2); // skip '"', 'hello' is 5 chars -> offset 1, result 2
        assert_eq!(calculate_orp("(test)"), 2);  // skip '(', 'test' is 4 chars -> offset 1, result 2
        assert_eq!(calculate_orp("...word"), 4); // skip '...', 'word' is 4 chars -> offset 1, result 4
    }
}
