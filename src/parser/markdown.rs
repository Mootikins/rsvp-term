use std::path::Path;

use markdown_it::parser::inline::Text;
use markdown_it::plugins::cmark::block::{
    blockquote::Blockquote,
    fence::CodeFence,
    heading::ATXHeading,
    list::{BulletList, ListItem, OrderedList},
    paragraph::Paragraph,
};
use markdown_it::plugins::cmark::inline::{
    backticks::CodeInline,
    emphasis::{Em, Strong},
    image::Image,
    link::Link,
};
use markdown_it::plugins::extra::tables::{Table, TableCell, TableRow};
use markdown_it::{plugins::cmark, plugins::extra, MarkdownIt, Node};

use crate::parser::traits::{DocumentParser, ParseError, ParsedDocument};
use crate::timing::generate_timing_hint;
use crate::types::{BlockContext, Section, Token, TokenStyle};

/// Markdown parser that extracts tokens for RSVP reading.
pub struct MarkdownParser {
    md: MarkdownIt,
}

impl MarkdownParser {
    /// Create a new markdown parser with `CommonMark` and GFM table support.
    #[must_use]
    pub fn new() -> Self {
        let mut md = MarkdownIt::new();
        cmark::add(&mut md);
        extra::tables::add(&mut md);
        Self { md }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for tracking current parsing state while walking the AST.
#[derive(Debug, Clone)]
struct ParserContext {
    /// Current text styling (bold, italic, etc.)
    style_stack: Vec<TokenStyle>,
    /// Current block context
    block_stack: Vec<BlockContext>,
    /// Quote depth tracking
    quote_depth: usize,
    /// List depth tracking
    list_depth: usize,
    /// Whether we're inside a skippable element (code block, image)
    skip_depth: usize,
    /// Flag set when entering a new block (cleared after first token)
    new_block_entered: bool,
    /// Current table row number (reset when exiting table)
    table_row: usize,
    /// Current table cell index within the row (0-indexed)
    table_cell_index: usize,
    /// Total number of cells in the current row
    table_cell_count: usize,
    /// Whether the current cell is the last in the row (for timing)
    is_last_table_cell: bool,
    /// Whether the current blockquote is a callout
    in_callout: bool,
    /// Whether we're inside inline code (preserves whitespace)
    in_inline_code: bool,
}

impl ParserContext {
    fn new() -> Self {
        Self {
            style_stack: vec![TokenStyle::Normal],
            block_stack: vec![BlockContext::Paragraph],
            quote_depth: 0,
            list_depth: 0,
            skip_depth: 0,
            new_block_entered: false,
            table_row: 0,
            table_cell_index: 0,
            table_cell_count: 0,
            is_last_table_cell: false,
            in_callout: false,
            in_inline_code: false,
        }
    }

    fn current_style(&self) -> TokenStyle {
        self.style_stack
            .last()
            .cloned()
            .unwrap_or(TokenStyle::Normal)
    }

    fn current_block(&self) -> BlockContext {
        self.block_stack
            .last()
            .cloned()
            .unwrap_or(BlockContext::Paragraph)
    }

    fn push_style(&mut self, style: TokenStyle) {
        // Handle style stacking (bold + italic = BoldItalic)
        let current = self.current_style();
        let new_style = match (&current, &style) {
            (TokenStyle::Bold, TokenStyle::Italic) | (TokenStyle::Italic, TokenStyle::Bold) => {
                TokenStyle::BoldItalic
            }
            (TokenStyle::BoldItalic, _) => TokenStyle::BoldItalic,
            _ => style,
        };
        self.style_stack.push(new_style);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn push_block(&mut self, block: BlockContext) {
        self.block_stack.push(block);
        self.new_block_entered = true;
    }

    fn pop_block(&mut self) {
        if self.block_stack.len() > 1 {
            self.block_stack.pop();
        }
    }

    const fn should_skip(&self) -> bool {
        self.skip_depth > 0
    }
}

/// Split text into words, respecting Unicode boundaries.
/// Em-dashes (—) and en-dashes (–) are treated as word separators.
/// Hyphenated words are split when portions are more than 3 characters long,
/// keeping the hyphen on the tail of the preceding portion.
fn split_into_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .flat_map(|part| {
            // Split on em-dash (—) and en-dash (–) as word separators
            part.split(['—', '–'])
        })
        .flat_map(|part| {
            // Handle hyphenated words
            if part.contains('-') {
                split_hyphenated_word(&part)
            } else {
                vec![part.to_string()]
            }
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Split hyphenated words when portions are > 3 characters.
/// Returns the split portions, keeping hyphens on the tail of preceding portions.
///
/// Examples:
/// - "well-known" → ["well-", "known"] (both > 3 chars)
/// - "co-op" → ["co-op"] (portions ≤ 3 chars, keep together)
/// - "mother-in-law" → ["mother-", "in-", "law"] (all > 3 except last)
fn split_hyphenated_word(word: &str) -> Vec<String> {
    let portions: Vec<&str> = word.split('-').collect();

    // If no hyphens or single portion, return as-is
    if portions.len() < 2 {
        return vec![word.to_string()];
    }

    // Check if any portion is > 3 chars
    let has_long_portion = portions.iter().any(|p| p.chars().count() > 3);

    // If all portions are ≤ 3 chars, keep the whole word together
    if !has_long_portion {
        return vec![word.to_string()];
    }

    // Split into individual portions, keeping hyphens on tails
    let mut result = Vec::new();
    for (i, portion) in portions.iter().enumerate() {
        let is_last = i == portions.len() - 1;

        if is_last {
            // Last portion never gets a trailing hyphen
            result.push(portion.to_string());
        } else {
            // All other portions keep their trailing hyphen
            result.push(format!("{}-", portion));
        }
    }

    result
}

/// Detect callout type from text like "[!folder]" or "[!note]"
/// Returns Some(callout_type) if found, None otherwise
fn detect_callout_type(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.starts_with("[!") {
        if let Some(end_pos) = trimmed.find(']') {
            let callout_type = trimmed[2..end_pos].to_lowercase();
            return Some(callout_type);
        }
    }
    None
}

impl DocumentParser for MarkdownParser {
    fn parse_file(&self, path: &Path) -> Result<ParsedDocument, ParseError> {
        let content = std::fs::read_to_string(path)?;
        self.parse_str(&content)
    }

    fn parse_str(&self, content: &str) -> Result<ParsedDocument, ParseError> {
        let ast = self.md.parse(content);

        let mut tokens = Vec::new();
        let mut sections = Vec::new();
        let mut ctx = ParserContext::new();

        walk_ast(&ast, &mut ctx, &mut tokens, &mut sections);

        // Update section token_end values
        for i in 0..sections.len() {
            if i + 1 < sections.len() {
                sections[i].token_end = sections[i + 1].token_start;
            } else {
                sections[i].token_end = tokens.len();
            }
        }

        Ok(ParsedDocument { tokens, sections })
    }
}

/// Recursively walk the AST and extract tokens.
fn walk_ast(
    node: &Node,
    ctx: &mut ParserContext,
    tokens: &mut Vec<Token>,
    sections: &mut Vec<Section>,
) {
    // Handle entering different node types
    let (restore_style, restore_block, restore_skip, restore_list_depth, restore_quote_depth) =
        enter_node(node, ctx, tokens, sections);

    // Process children
    for child in &node.children {
        walk_ast(child, ctx, tokens, sections);
    }

    // Handle exiting (restore context)
    if restore_style {
        ctx.pop_style();
        ctx.in_inline_code = false;
    }
    if restore_block {
        ctx.pop_block();
    }
    if restore_skip {
        ctx.skip_depth = ctx.skip_depth.saturating_sub(1);
    }
    if restore_list_depth {
        ctx.list_depth = ctx.list_depth.saturating_sub(1);
    }
    if restore_quote_depth {
        ctx.quote_depth = ctx.quote_depth.saturating_sub(1);
        ctx.in_callout = false;
    }
}

/// Handle entering a node. Returns flags for what to restore on exit.
/// Returns (`restore_style`, `restore_block`, `restore_skip`, `restore_list_depth`, `restore_quote_depth`)
fn enter_node(
    node: &Node,
    ctx: &mut ParserContext,
    tokens: &mut Vec<Token>,
    sections: &mut Vec<Section>,
) -> (bool, bool, bool, bool, bool) {
    let mut restore_style = false;
    let mut restore_block = false;
    let mut restore_skip = false;
    let mut restore_list_depth = false;
    let mut restore_quote_depth = false;

    // Skip code blocks and images entirely
    if node.is::<CodeFence>() || node.is::<Image>() {
        ctx.skip_depth += 1;
        restore_skip = true;
        return (
            restore_style,
            restore_block,
            restore_skip,
            restore_list_depth,
            restore_quote_depth,
        );
    }

    // Handle block-level elements
    if node.is::<ATXHeading>() {
        if let Some(heading) = node.cast::<ATXHeading>() {
            let level = heading.level;
            ctx.push_block(BlockContext::Heading(level));
            restore_block = true;

            // Extract section title and create section entry
            let title = node.collect_text();
            sections.push(Section {
                title,
                level,
                token_start: tokens.len(),
                token_end: 0, // Will be updated later
            });
        }
    } else if node.is::<Paragraph>() {
        // Don't push paragraph context if inside a callout (use callout context instead)
        if !ctx.in_callout {
            ctx.push_block(BlockContext::Paragraph);
            restore_block = true;
        }
    } else if node.is::<Blockquote>() {
        ctx.quote_depth += 1;

        // Check if this is a callout (e.g., "[!folder]", "[!note]")
        // We need to look at paragraph text nodes inside the blockquote
        let mut callout_type: Option<String> = None;
        for child in &node.children {
            // Check if this is a paragraph with callout syntax
            if child.is::<Paragraph>() {
                for grandchild in &child.children {
                    if let Some(text) = grandchild.cast::<Text>() {
                        callout_type = detect_callout_type(&text.content);
                        if callout_type.is_some() {
                            break;
                        }
                    }
                }
            }
            if callout_type.is_some() {
                break;
            }
        }

        if let Some(ct) = callout_type {
            ctx.in_callout = true;
            ctx.push_block(BlockContext::Callout(ct));
        } else {
            ctx.in_callout = false;
            ctx.push_block(BlockContext::Quote(ctx.quote_depth));
        }
        restore_block = true;
        restore_quote_depth = true;
    } else if node.is::<BulletList>() || node.is::<OrderedList>() {
        ctx.list_depth += 1;
        restore_list_depth = true;
    } else if node.is::<ListItem>() {
        ctx.push_block(BlockContext::ListItem(ctx.list_depth));
        restore_block = true;
    } else if node.is::<Table>() {
        // Reset counters when entering a table
        ctx.table_row = 0;
        ctx.table_cell_index = 0;
        ctx.table_cell_count = 0;
        ctx.is_last_table_cell = false;
    } else if node.is::<TableRow>() {
        // Increment row counter for each row
        ctx.table_row += 1;
        // Reset cell index for new row and count total cells
        ctx.table_cell_index = 0;
        ctx.table_cell_count = node.children.iter().filter(|c| c.is::<TableCell>()).count();
    } else if node.is::<TableCell>() {
        // Check if this is the last cell in the row
        ctx.is_last_table_cell = ctx.table_cell_index == ctx.table_cell_count - 1;

        // Each cell is a distinct block for timing and rendering
        ctx.push_block(BlockContext::TableCell(ctx.table_row));
        restore_block = true;

        // Move to next cell
        ctx.table_cell_index += 1;
    }

    // Handle inline-level styling elements
    if node.is::<Strong>() {
        ctx.push_style(TokenStyle::Bold);
        restore_style = true;
    } else if node.is::<Em>() {
        ctx.push_style(TokenStyle::Italic);
        restore_style = true;
    } else if node.is::<CodeInline>() {
        ctx.in_inline_code = true;
        ctx.push_style(TokenStyle::Code);
        restore_style = true;
    } else if node.is::<Link>() {
        if let Some(link) = node.cast::<Link>() {
            ctx.push_style(TokenStyle::Link(link.url.clone()));
            restore_style = true;
        }
    }

    // Handle text nodes - extract words
    if let Some(text) = node.cast::<Text>() {
        if !ctx.should_skip() {
            // If inside inline code, preserve the entire text as a single token
            let words = if ctx.in_inline_code {
                vec![text.content.clone()]
            } else {
                split_into_words(&text.content)
            };
            let word_count = words.len();

            for (i, word) in words.into_iter().enumerate() {
                let is_last_word = i == word_count - 1;
                // Check if this might be a paragraph end
                // (simplified - we'd need more context for full accuracy)
                let is_paragraph_end = is_last_word && word.ends_with(|c: char| ".!?".contains(c));
                // First word of a new block gets the new_block timing modifier
                let is_new_block = ctx.new_block_entered || tokens.is_empty();

                let timing_hint = generate_timing_hint(
                    &word,
                    is_paragraph_end,
                    is_new_block,
                    ctx.is_last_table_cell,
                );

                tokens.push(Token {
                    word,
                    style: ctx.current_style(),
                    block: ctx.current_block(),
                    parent_context: None,
                    timing_hint,
                });

                // Clear flag after first token in new block
                ctx.new_block_entered = false;
            }
        }
    }

    (
        restore_style,
        restore_block,
        restore_skip,
        restore_list_depth,
        restore_quote_depth,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parser() {
        let parser = MarkdownParser::new();
        let result = parser.parse_str("test").unwrap();
        assert_eq!(result.tokens.len(), 1);
    }

    #[test]
    fn test_word_splitting() {
        let words = split_into_words("Hello   world\ntest");
        assert_eq!(words, vec!["Hello", "world", "test"]);
    }

    #[test]
    fn test_empty_input() {
        let parser = MarkdownParser::new();
        let result = parser.parse_str("").unwrap();
        assert!(result.tokens.is_empty());
        assert!(result.sections.is_empty());
    }

    #[test]
    fn test_multiple_headings() {
        let parser = MarkdownParser::new();
        let result = parser
            .parse_str("# First\n\n## Second\n\n### Third")
            .unwrap();
        assert_eq!(result.sections.len(), 3);
        assert_eq!(result.sections[0].level, 1);
        assert_eq!(result.sections[1].level, 2);
        assert_eq!(result.sections[2].level, 3);
    }

    #[test]
    fn test_bold_italic_combined() {
        let parser = MarkdownParser::new();
        let result = parser.parse_str("***both***").unwrap();
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].style, TokenStyle::BoldItalic);
    }

    #[test]
    fn test_multiple_lists_depth_reset() {
        let parser = MarkdownParser::new();
        let result = parser
            .parse_str("- Item 1\n\n- Item 2\n\n- Item 3")
            .unwrap();
        // All should have depth 1, not 1, 2, 3 (depth must reset between lists)
        for token in &result.tokens {
            if let BlockContext::ListItem(depth) = &token.block {
                assert_eq!(*depth, 1, "List depth should be 1, not cumulative");
            }
        }
    }

    #[test]
    fn test_multiple_blockquotes_depth_reset() {
        let parser = MarkdownParser::new();
        let result = parser.parse_str("> Quote 1\n\n> Quote 2").unwrap();
        // Both should have depth 1 (depth must reset between quotes)
        for token in &result.tokens {
            if let BlockContext::Quote(depth) = &token.block {
                assert_eq!(*depth, 1, "Quote depth should be 1, not cumulative");
            }
        }
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
    fn test_em_dash_separates_words() {
        let parser = MarkdownParser::new();
        let result = parser.parse_str("Hello—world").unwrap();
        // Em-dash should separate words
        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0].word, "Hello");
        assert_eq!(result.tokens[1].word, "world");
    }
}
