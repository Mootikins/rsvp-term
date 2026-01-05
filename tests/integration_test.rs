use rsvp_term::app::App;
use rsvp_term::orp::calculate_orp;
use rsvp_term::parser::{DocumentParser, MarkdownParser};
use rsvp_term::timing::calculate_duration;
use rsvp_term::types::TimedToken;

#[test]
fn test_full_document_flow() {
    let content = r#"
# Test Document

This is a **bold** test with *italic* text.

## Second Section

- List item one
- List item two

> A quote here
"#;

    let parser = MarkdownParser::new();
    let doc = parser.parse_str(content).unwrap();

    assert!(!doc.tokens.is_empty());
    assert_eq!(doc.sections.len(), 2);

    let timed: Vec<TimedToken> = doc
        .tokens
        .into_iter()
        .map(|t| TimedToken {
            duration_ms: calculate_duration(&t, 300),
            orp_position: calculate_orp(&t.word),
            token: t,
        })
        .collect();

    let mut app = App::new(timed, doc.sections);

    // Test navigation
    assert_eq!(app.position(), 0);
    app.advance();
    assert_eq!(app.position(), 1);

    // Test WPM
    app.increase_wpm();
    assert_eq!(app.wpm(), 325);
}
