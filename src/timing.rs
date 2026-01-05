use crate::types::{TimingHint, Token};

/// Calculate display duration for a token at given WPM.
/// Returns duration in milliseconds.
///
/// The duration is calculated as:
/// - Base: 60,000ms / WPM (e.g., 300 WPM = 200ms per word)
/// - Plus timing hint modifiers scaled by WPM (modifiers are calibrated for 300 WPM)
/// - Minimum duration is 50ms to prevent too-fast display
#[must_use]
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000_u64 / u64::from(wpm);

    // Scale modifiers by WPM ratio (modifiers were designed for 300 WPM)
    // At 600 WPM, modifiers should be halved; at 150 WPM, doubled
    let scale = 300.0 / f64::from(wpm);
    let modifiers = (f64::from(token.timing_hint.word_length_modifier)
        + f64::from(token.timing_hint.punctuation_modifier)
        + f64::from(token.timing_hint.structure_modifier))
        * scale;

    // Safe: base_ms is at most 60000, scaled modifiers are bounded
    // Result after max(50) is always positive and fits in u64
    #[allow(clippy::cast_sign_loss)]
    {
        ((base_ms as f64 + modifiers).round().max(50.0)) as u64
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
    table_column: Option<usize>,
) -> TimingHint {
    let len = word.chars().count();

    // Word length modifier - gentle increase for longer words
    // Old values were too aggressive (40ms/char over 10 made long words 2x slower)
    let word_length_modifier: i32 = if len > 10 {
        let base = (10 - 6) * 10; // 40ms for chars 7-10
        let extra = (len - 10) * 15; // 15ms per char over 10 (was 40)
        i32::try_from(base + extra).unwrap_or(500)
    } else if len > 6 {
        i32::try_from((len - 6) * 10).unwrap_or(40) // 10ms per char (was 20)
    } else {
        0
    };

    // Punctuation modifier (check last char) - reduced from 200/150 to 100/75
    let punctuation_modifier = word.chars().last().map_or(0, |c| match c {
        '.' | '!' | '?' => 100,
        ',' | ':' | ';' => 75,
        _ => 0,
    });

    // Structure modifier - reduced from 300/150 to 150/75
    let structure_modifier = if is_paragraph_end || is_last_table_cell {
        150
    } else if is_new_block {
        75
    } else {
        0
    };

    TimingHint {
        word_length_modifier,
        punctuation_modifier,
        structure_modifier,
        is_cell_start,
        table_column,
        is_block_start: is_new_block,
    }
}
