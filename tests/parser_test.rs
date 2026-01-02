use rsvp_term::parser::{MarkdownParser, DocumentParser};
use rsvp_term::types::TokenStyle;

#[test]
fn test_parse_simple_paragraph() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Hello world").unwrap();

    assert_eq!(result.tokens.len(), 2);
    assert_eq!(result.tokens[0].word, "Hello");
    assert_eq!(result.tokens[1].word, "world");
}

#[test]
fn test_parse_heading() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("# Title\n\nParagraph").unwrap();

    assert_eq!(result.sections.len(), 1);
    assert_eq!(result.sections[0].title, "Title");
    assert_eq!(result.sections[0].level, 1);
}

#[test]
fn test_parse_bold() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("This is **bold** text").unwrap();

    let bold_token = result.tokens.iter().find(|t| t.word == "bold").unwrap();
    assert_eq!(bold_token.style, TokenStyle::Bold);
}

#[test]
fn test_parse_italic() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("This is *italic* text").unwrap();

    let italic_token = result.tokens.iter().find(|t| t.word == "italic").unwrap();
    assert_eq!(italic_token.style, TokenStyle::Italic);
}

#[test]
fn test_skip_code_block() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Before\n\n```\ncode here\n```\n\nAfter").unwrap();

    let words: Vec<&str> = result.tokens.iter().map(|t| t.word.as_str()).collect();
    assert!(!words.contains(&"code"));
    assert!(words.contains(&"Before"));
    assert!(words.contains(&"After"));
}

#[test]
fn test_skip_image() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Before ![alt](image.png) After").unwrap();

    let words: Vec<&str> = result.tokens.iter().map(|t| t.word.as_str()).collect();
    assert!(!words.contains(&"alt"));
    assert!(!words.contains(&"image.png"));
}
