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
fn split_into_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
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
        ctx.push_block(BlockContext::Paragraph);
        restore_block = true;
    } else if node.is::<Blockquote>() {
        ctx.quote_depth += 1;
        ctx.push_block(BlockContext::Quote(ctx.quote_depth));
        restore_block = true;
        restore_quote_depth = true;
    } else if node.is::<BulletList>() || node.is::<OrderedList>() {
        ctx.list_depth += 1;
        restore_list_depth = true;
    } else if node.is::<ListItem>() {
        ctx.push_block(BlockContext::ListItem(ctx.list_depth));
        restore_block = true;
    } else if node.is::<Table>() {
        // Reset row counter when entering a table
        ctx.table_row = 0;
    } else if node.is::<TableRow>() {
        // Increment row counter for each row
        ctx.table_row += 1;
    } else if node.is::<TableCell>() {
        // Each cell is a distinct block for timing and rendering
        ctx.push_block(BlockContext::TableCell(ctx.table_row));
        restore_block = true;
    }

    // Handle inline-level styling elements
    if node.is::<Strong>() {
        ctx.push_style(TokenStyle::Bold);
        restore_style = true;
    } else if node.is::<Em>() {
        ctx.push_style(TokenStyle::Italic);
        restore_style = true;
    } else if node.is::<CodeInline>() {
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
            let words = split_into_words(&text.content);
            let word_count = words.len();

            for (i, word) in words.into_iter().enumerate() {
                let is_last_word = i == word_count - 1;
                // Check if this might be a paragraph end
                // (simplified - we'd need more context for full accuracy)
                let is_paragraph_end = is_last_word && word.ends_with(|c: char| ".!?".contains(c));
                // First word of a new block gets the new_block timing modifier
                let is_new_block = ctx.new_block_entered || tokens.is_empty();

                let timing_hint = generate_timing_hint(&word, is_paragraph_end, is_new_block);

                tokens.push(Token {
                    word,
                    style: ctx.current_style(),
                    block: ctx.current_block(),
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
}
