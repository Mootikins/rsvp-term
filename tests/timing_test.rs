use rsvp_term::timing::{calculate_duration, generate_timing_hint};
use rsvp_term::types::{BlockContext, TimingHint, Token, TokenStyle};

fn make_token(word: &str, hint: TimingHint) -> Token {
    Token {
        word: word.to_string(),
        style: TokenStyle::Normal,
        block: BlockContext::Paragraph,
        parent_context: None,
        timing_hint: hint,
    }
}

#[test]
fn test_base_timing_300_wpm() {
    let token = make_token("hello", TimingHint::default());
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 200); // 60000 / 300 = 200ms
}

#[test]
fn test_base_timing_600_wpm() {
    let token = make_token("hello", TimingHint::default());
    let duration = calculate_duration(&token, 600);
    assert_eq!(duration, 100); // 60000 / 600 = 100ms
}

#[test]
fn test_long_word_modifier() {
    let hint = TimingHint {
        word_length_modifier: 30, // 3 extra chars * 10ms
        ..Default::default()
    };
    let token = make_token("extraordinary", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 230); // 200 + 30
}

#[test]
fn test_punctuation_modifier() {
    let hint = TimingHint {
        punctuation_modifier: 200, // period
        ..Default::default()
    };
    let token = make_token("end.", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 400); // 200 + 200
}

#[test]
fn test_structure_modifier() {
    let hint = TimingHint {
        structure_modifier: 300, // paragraph break
        ..Default::default()
    };
    let token = make_token("paragraph", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 500); // 200 + 300
}

#[test]
fn test_combined_modifiers() {
    let hint = TimingHint {
        word_length_modifier: 20,
        punctuation_modifier: 150,
        structure_modifier: 0,
    };
    let token = make_token("sentence,", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 370); // 200 + 20 + 150
}

#[test]
fn test_hint_short_word() {
    let hint = generate_timing_hint("the", false, false, false);
    assert_eq!(hint.word_length_modifier, 0);
}

#[test]
fn test_hint_long_word() {
    // "beautiful" = 9 chars, 3 extra over 6 = 60ms
    let hint = generate_timing_hint("beautiful", false, false, false);
    assert_eq!(hint.word_length_modifier, 60);
}

#[test]
fn test_hint_very_long_word() {
    // "extraordinary" = 13 chars
    // > 6: (min(len,10) - 6) * 20 = (10-6)*20 = 80
    // > 10: (len - 10) * 40 = (13-10)*40 = 120
    // total = 80 + 120 = 200
    let hint = generate_timing_hint("extraordinary", false, false, false);
    assert_eq!(hint.word_length_modifier, 200);
}

#[test]
fn test_hint_comma() {
    let hint = generate_timing_hint("word,", false, false, false);
    assert_eq!(hint.punctuation_modifier, 150);
}

#[test]
fn test_hint_period() {
    let hint = generate_timing_hint("end.", false, false, false);
    assert_eq!(hint.punctuation_modifier, 200);
}

#[test]
fn test_hint_question() {
    let hint = generate_timing_hint("why?", false, false, false);
    assert_eq!(hint.punctuation_modifier, 200);
}

#[test]
fn test_hint_paragraph_break() {
    let hint = generate_timing_hint("word", true, false, false);
    assert_eq!(hint.structure_modifier, 300);
}

#[test]
fn test_hint_new_block() {
    let hint = generate_timing_hint("word", false, true, false);
    assert_eq!(hint.structure_modifier, 150);
}

#[test]
fn test_hint_last_table_cell() {
    let hint = generate_timing_hint("word", false, false, true);
    assert_eq!(hint.structure_modifier, 300); // Last table cell gets 300ms
}
