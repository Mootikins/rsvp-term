use rsvp_term::app::{App, ViewMode};
use rsvp_term::types::{BlockContext, TimedToken, TimingHint, Token, TokenStyle};

fn make_timed_token(word: &str) -> TimedToken {
    TimedToken {
        token: Token {
            word: word.to_string(),
            style: TokenStyle::Normal,
            block: BlockContext::Paragraph,
            parent_context: None,
            timing_hint: TimingHint::default(),
        },
        duration_ms: 200,
        orp_position: 1,
    }
}

#[test]
fn test_app_initial_state() {
    let tokens = vec![make_timed_token("hello"), make_timed_token("world")];
    let app = App::new(tokens, vec![]);

    assert_eq!(app.position(), 0);
    assert_eq!(app.wpm(), 300);
    assert!(!app.is_paused());
}

#[test]
fn test_app_pause_toggle() {
    let tokens = vec![make_timed_token("hello")];
    let mut app = App::new(tokens, vec![]);

    assert!(!app.is_paused());
    app.toggle_pause();
    assert!(app.is_paused());
    app.toggle_pause();
    assert!(!app.is_paused());
}

#[test]
fn test_app_wpm_adjustment() {
    let tokens = vec![make_timed_token("hello")];
    let mut app = App::new(tokens, vec![]);

    app.increase_wpm();
    assert_eq!(app.wpm(), 325);

    app.decrease_wpm();
    assert_eq!(app.wpm(), 300);
}

#[test]
fn test_app_wpm_bounds() {
    let tokens = vec![make_timed_token("hello")];
    let mut app = App::new(tokens, vec![]);

    // Test upper bound
    for _ in 0..100 {
        app.increase_wpm();
    }
    assert_eq!(app.wpm(), 1000);

    // Test lower bound
    for _ in 0..100 {
        app.decrease_wpm();
    }
    assert_eq!(app.wpm(), 100);
}

#[test]
fn test_app_advance() {
    let tokens = vec![make_timed_token("hello"), make_timed_token("world")];
    let mut app = App::new(tokens, vec![]);

    assert_eq!(app.position(), 0);
    app.advance();
    assert_eq!(app.position(), 1);
}

#[test]
fn test_app_view_mode_toggle() {
    let tokens = vec![make_timed_token("hello")];
    let mut app = App::new(tokens, vec![]);

    assert_eq!(app.view_mode(), ViewMode::Reading);
    app.toggle_outline();
    assert_eq!(app.view_mode(), ViewMode::Outline);
    app.toggle_outline();
    assert_eq!(app.view_mode(), ViewMode::Reading);
}
