use insta::assert_debug_snapshot;
use rsvp_term::parser::{DocumentParser, MarkdownParser};
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
    let result = parser
        .parse_str("Before\n\n```\ncode here\n```\n\nAfter")
        .unwrap();

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

// Snapshot tests for capturing full token/section structure

#[test]
fn test_snapshot_simple_doc() {
    let parser = MarkdownParser::new();
    let result = parser
        .parse_str(
            "# Hello\n\nThis is **bold** and *italic* text.\n\n## World\n\nAnother paragraph.",
        )
        .unwrap();

    assert_debug_snapshot!(result.tokens);
    assert_debug_snapshot!(result.sections);
}

#[test]
fn test_snapshot_list() {
    let parser = MarkdownParser::new();
    let result = parser
        .parse_str("- First item\n- Second item\n- Third item")
        .unwrap();

    assert_debug_snapshot!(result.tokens);
}

#[test]
fn test_snapshot_quote() {
    let parser = MarkdownParser::new();
    let result = parser
        .parse_str("> This is a quote\n> with multiple lines")
        .unwrap();

    assert_debug_snapshot!(result.tokens);
}

#[test]
fn test_parse_callout_with_folder_emoji() {
    let parser = MarkdownParser::new();
    let result = parser
        .parse_str("> [!folder] File: example.txt")
        .unwrap();

    // Should parse as Callout with folder type
    assert!(!result.tokens.is_empty());
    if let rsvp_term::types::BlockContext::Callout(callout_type) = &result.tokens[0].block {
        assert_eq!(callout_type, "folder");
    } else {
        panic!("Expected Callout block context, got {:?}", result.tokens[0].block);
    }
}

#[test]
fn test_inline_code_with_underscores_preserved() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Use `my_variable_name` for x").unwrap();

    // Code with underscores should be preserved as a single word
    assert_eq!(result.tokens.len(), 4);
    assert_eq!(result.tokens[1].word, "my_variable_name");
    assert_eq!(result.tokens[1].style, rsvp_term::types::TokenStyle::Code);
}

#[test]
fn test_inline_code_with_hyphens_preserved() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Use `my-function-name` for x").unwrap();

    // Code with hyphens should be preserved as a single word
    assert_eq!(result.tokens.len(), 4);
    assert_eq!(result.tokens[1].word, "my-function-name");
    assert_eq!(result.tokens[1].style, rsvp_term::types::TokenStyle::Code);
}

#[test]
fn test_inline_code_with_multiple_words_preserved() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Use `my function name` for x").unwrap();

    // Inline code with multiple words should NOT be split
    assert_eq!(result.tokens.len(), 4);
    assert_eq!(result.tokens[1].word, "my function name");
    assert_eq!(result.tokens[1].style, rsvp_term::types::TokenStyle::Code);
}

#[test]
fn test_hyphenated_words_split_when_long() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("This is well-known text").unwrap();
    // "well-known" should be split into "well-" and "known" (both > 3 chars)
    assert_eq!(result.tokens.len(), 5);
    assert_eq!(result.tokens[2].word, "well-");
    assert_eq!(result.tokens[3].word, "known");
}

#[test]
fn test_short_hyphenated_words_preserved() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("This is co-op text").unwrap();
    // "co-op" should be preserved as single word (portions ≤ 3 chars)
    assert_eq!(result.tokens.len(), 4);
    assert_eq!(result.tokens[2].word, "co-op");
}

#[test]
fn test_multiple_hyphens_split() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str("Check mother-in-law").unwrap();
    // "mother-in-law" → "mother-", "in-", "law"
    assert_eq!(result.tokens.len(), 4);
    assert_eq!(result.tokens[1].word, "mother-");
    assert_eq!(result.tokens[2].word, "in-");
    assert_eq!(result.tokens[3].word, "law");
}

#[test]
fn test_table_last_column_longer_pause() {
    let parser = MarkdownParser::new();
    let result = parser
        .parse_str(
            "| Col1 | Col2 | Col3 |\n|------|------|------|\n| Data | Data | Data |",
        )
        .unwrap();

    // Find tokens from each column
    let col1_tokens: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.word == "Col1" || t.word == "Data")
        .take(2)
        .collect();

    let col3_tokens: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.word == "Col3")
        .collect();

    // Column 3 tokens should have longer structure modifier than column 1
    // Column 1: structure_modifier should be 150 (standard new cell)
    // Column 3: structure_modifier should be > 150 (last cell special handling)
    assert!(col3_tokens[0].timing_hint.structure_modifier > col1_tokens[0].timing_hint.structure_modifier,
        "Last column should have longer pause: col3={:?} vs col1={:?}",
        col3_tokens[0].timing_hint.structure_modifier,
        col1_tokens[0].timing_hint.structure_modifier
    );
}
