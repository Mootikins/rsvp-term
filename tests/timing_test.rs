use rsvp_term::timing::calculate_duration;
use rsvp_term::types::{Token, TokenStyle, BlockContext, TimingHint};

fn make_token(word: &str, hint: TimingHint) -> Token {
    Token {
        word: word.to_string(),
        style: TokenStyle::Normal,
        block: BlockContext::Paragraph,
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
        word_length_modifier: 60, // 3 extra chars * 20ms
        ..Default::default()
    };
    let token = make_token("extraordinary", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 260); // 200 + 60
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
        word_length_modifier: 40,
        punctuation_modifier: 150,
        structure_modifier: 0,
    };
    let token = make_token("sentence,", hint);
    let duration = calculate_duration(&token, 300);
    assert_eq!(duration, 390); // 200 + 40 + 150
}
