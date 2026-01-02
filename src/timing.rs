use crate::types::Token;

/// Calculate display duration for a token at given WPM.
/// Returns duration in milliseconds.
///
/// The duration is calculated as:
/// - Base: 60,000ms / WPM (e.g., 300 WPM = 200ms per word)
/// - Plus timing hint modifiers for word length, punctuation, structure
/// - Minimum duration is 50ms to prevent too-fast display
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000 / wpm as u64;
    let modifiers = token.timing_hint.word_length_modifier
        + token.timing_hint.punctuation_modifier
        + token.timing_hint.structure_modifier;

    (base_ms as i64 + modifiers as i64).max(50) as u64
}
