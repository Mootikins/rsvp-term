/// Calculate the Optimal Recognition Point for a word.
/// Returns the 0-indexed position of the character to highlight.
///
/// ORP is typically about 1/3 into the word, where the eye naturally focuses.
/// For Spritz-style RSVP display, this letter is highlighted and the word
/// is centered around it.
///
/// Algorithm:
/// - 1-3 chars: position 0 (first letter)
/// - 4-6 chars: position 1 (second letter)
/// - 7-9 chars: position 2 (third letter)
/// - 10+ chars: position 3 (fourth letter)
///
/// Uses `.chars().count()` for correct Unicode handling.
pub fn calculate_orp(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        0..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    }
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
}
