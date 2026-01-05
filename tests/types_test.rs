use rsvp_term::types::{BlockContext, TimingHint, Token, TokenStyle};

#[test]
fn test_token_creation() {
    let token = Token {
        word: "hello".to_string(),
        style: TokenStyle::Normal,
        block: BlockContext::Paragraph,
        parent_context: None,
        timing_hint: TimingHint::default(),
    };
    assert_eq!(token.word, "hello");
}

#[test]
fn test_token_style_variants() {
    let styles = vec![
        TokenStyle::Normal,
        TokenStyle::Bold,
        TokenStyle::Italic,
        TokenStyle::BoldItalic,
        TokenStyle::Code,
        TokenStyle::Link("https://example.com".to_string()),
    ];
    assert_eq!(styles.len(), 6);
}
