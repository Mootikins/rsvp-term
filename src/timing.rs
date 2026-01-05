use crate::types::{TimingHint, Token};

/// Calculate display duration for a token at given WPM.
/// Returns duration in milliseconds.
///
/// The duration is calculated as:
/// - Base: 60,000ms / WPM (e.g., 300 WPM = 200ms per word)
/// - Plus timing hint modifiers for word length, punctuation, structure
/// - Minimum duration is 50ms to prevent too-fast display
#[must_use]
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000_u64 / u64::from(wpm);
    let modifiers = i64::from(token.timing_hint.word_length_modifier)
        + i64::from(token.timing_hint.punctuation_modifier)
        + i64::from(token.timing_hint.structure_modifier);

    // Safe: base_ms is at most 60000, modifiers are small i32s summing to < 1000
    // Result after max(50) is always positive and fits in u64
    #[allow(clippy::cast_sign_loss)]
    {
        (i64::try_from(base_ms).unwrap_or(i64::MAX) + modifiers).max(50) as u64
    }
}

/// Generate timing hints based on word characteristics.
#[must_use]
pub fn generate_timing_hint(
    word: &str,
    is_paragraph_end: bool,
    is_new_block: bool,
    is_last_table_cell: bool,
    is_cell_start: bool,
) -> TimingHint {
    let len = word.chars().count();

    // Word length modifier - safe conversion with bounded fallback
    // Realistic words are < 50 chars, so values stay well under i32::MAX
    let word_length_modifier: i32 = if len > 10 {
        let base = (10 - 6) * 20; // 80ms for chars 7-10
        let extra = (len - 10) * 40; // 40ms per char over 10
        i32::try_from(base + extra).unwrap_or(1000)
    } else if len > 6 {
        i32::try_from((len - 6) * 20).unwrap_or(80)
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
    // Priority: paragraph_end or last_table_cell (300ms) > new_block (150ms)
    let structure_modifier = if is_paragraph_end || is_last_table_cell {
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
        is_cell_start,
    }
}
