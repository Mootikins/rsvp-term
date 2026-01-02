use crate::types::{TimingHint, Token};

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

/// Generate timing hints based on word characteristics.
pub fn generate_timing_hint(word: &str, is_paragraph_end: bool, is_new_block: bool) -> TimingHint {
    let len = word.chars().count();

    // Word length modifier
    let word_length_modifier = if len > 10 {
        let base = (10 - 6) * 20; // 80ms for chars 7-10
        let extra = (len - 10) * 40; // 40ms per char over 10
        (base + extra) as i32
    } else if len > 6 {
        ((len - 6) * 20) as i32
    } else {
        0
    };

    // Punctuation modifier (check last char)
    let punctuation_modifier = word.chars().last().map_or(0, |c| match c {
        '.' | '!' | '?' => 200,
        ',' | ':' | ';' => 150,
        _ => 0,
    });

    // Structure modifier
    let structure_modifier = if is_paragraph_end {
        300
    } else if is_new_block {
        150
    } else {
        0
    };

    TimingHint {
        word_length_modifier,
        punctuation_modifier,
        structure_modifier,
    }
}
