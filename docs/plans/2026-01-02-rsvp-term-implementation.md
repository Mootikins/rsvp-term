# rsvp-term Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a TUI for RSVP reading of markdown prose with ORP-centered display and document context.

**Architecture:** SOLID separation - Parser (markdown → tokens), Timing (tokens → timed tokens), Renderer (state → UI). Trait-based parser abstraction enables future format support (EPUB).

**Tech Stack:** Rust, Ratatui, Crossterm, markdown-it, clap, insta

---

## Phase 1: Project Setup

### Task 1.1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `.gitignore`

**Step 1: Initialize cargo project**

Run:
```bash
cargo init
```

**Step 2: Update Cargo.toml with dependencies**

```toml
[package]
name = "rsvp-term"
version = "0.1.0"
edition = "2021"
description = "TUI for RSVP reading of markdown prose"
license = "MIT"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
markdown-it = "0.6"
clap = { version = "4", features = ["derive"] }

[dev-dependencies]
insta = "1.41"
```

**Step 3: Create minimal lib.rs**

```rust
pub mod parser;
pub mod timing;
pub mod orp;
pub mod ui;
pub mod app;
```

**Step 4: Create placeholder modules**

Create empty module files:
- `src/parser.rs`
- `src/timing.rs`
- `src/orp.rs`
- `src/ui.rs`
- `src/app.rs`

Each containing just: `// TODO`

**Step 5: Create minimal main.rs**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "rsvp-term")]
#[command(about = "TUI for RSVP reading of markdown prose")]
struct Cli {
    /// Markdown file to read
    file: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    println!("Reading: {:?}", cli.file);
}
```

**Step 6: Add .gitignore**

```
/target
Cargo.lock
```

**Step 7: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 8: Commit**

```bash
git add -A
git commit -m "feat: initialize rust project with dependencies"
```

---

## Phase 2: Core Types

### Task 2.1: Define Token Types

**Files:**
- Create: `src/types.rs`
- Modify: `src/lib.rs`

**Step 1: Write test for token creation**

Create `tests/types_test.rs`:
```rust
use rsvp_term::types::{Token, TokenStyle, BlockContext, TimingHint};

#[test]
fn test_token_creation() {
    let token = Token {
        word: "hello".to_string(),
        style: TokenStyle::Normal,
        block: BlockContext::Paragraph,
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test types`
Expected: FAIL - module not found

**Step 3: Implement types module**

Create `src/types.rs`:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Link(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockContext {
    Paragraph,
    ListItem(usize),    // depth
    Quote(usize),       // depth
    Callout(String),    // type
    Heading(u8),        // level 1-6
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TimingHint {
    pub word_length_modifier: i32,
    pub punctuation_modifier: i32,
    pub structure_modifier: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub word: String,
    pub style: TokenStyle,
    pub block: BlockContext,
    pub timing_hint: TimingHint,
}

#[derive(Debug, Clone)]
pub struct TimedToken {
    pub token: Token,
    pub duration_ms: u64,
    pub orp_position: usize,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub level: u8,
    pub token_start: usize,
    pub token_end: usize,
}
```

**Step 4: Update lib.rs**

```rust
pub mod types;
pub mod parser;
pub mod timing;
pub mod orp;
pub mod ui;
pub mod app;
```

**Step 5: Run test to verify it passes**

Run: `cargo test types`
Expected: PASS

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add core token types"
```

---

## Phase 3: ORP Calculation

### Task 3.1: ORP Position Logic

**Files:**
- Modify: `src/orp.rs`
- Create: `tests/orp_test.rs`

**Step 1: Write ORP tests**

Create `tests/orp_test.rs`:
```rust
use rsvp_term::orp::calculate_orp;

#[test]
fn test_orp_short_words() {
    assert_eq!(calculate_orp("a"), 0);      // 1 char
    assert_eq!(calculate_orp("to"), 0);     // 2 chars
    assert_eq!(calculate_orp("the"), 0);    // 3 chars
}

#[test]
fn test_orp_medium_words() {
    assert_eq!(calculate_orp("word"), 1);   // 4 chars
    assert_eq!(calculate_orp("quick"), 1);  // 5 chars
    assert_eq!(calculate_orp("jumped"), 1); // 6 chars
}

#[test]
fn test_orp_longer_words() {
    assert_eq!(calculate_orp("quickly"), 2);    // 7 chars
    assert_eq!(calculate_orp("beautiful"), 2);  // 9 chars
}

#[test]
fn test_orp_very_long_words() {
    assert_eq!(calculate_orp("ということ"), 3);  // 10+ chars
    assert_eq!(calculate_orp("extraordinary"), 3);
}

#[test]
fn test_orp_empty_string() {
    assert_eq!(calculate_orp(""), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test orp`
Expected: FAIL - function not found

**Step 3: Implement ORP calculation**

Update `src/orp.rs`:
```rust
/// Calculate the Optimal Recognition Point for a word.
/// Returns the 0-indexed position of the character to highlight.
pub fn calculate_orp(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        0..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orp_boundary_cases() {
        assert_eq!(calculate_orp("abc"), 0);   // 3 -> 0
        assert_eq!(calculate_orp("abcd"), 1);  // 4 -> 1
        assert_eq!(calculate_orp("abcdef"), 1); // 6 -> 1
        assert_eq!(calculate_orp("abcdefg"), 2); // 7 -> 2
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test orp`
Expected: PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add ORP position calculation"
```

---

## Phase 4: Timing Logic

### Task 4.1: Base Timing Calculation

**Files:**
- Modify: `src/timing.rs`
- Create: `tests/timing_test.rs`

**Step 1: Write timing tests**

Create `tests/timing_test.rs`:
```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test timing`
Expected: FAIL - function not found

**Step 3: Implement timing calculation**

Update `src/timing.rs`:
```rust
use crate::types::Token;

/// Calculate display duration for a token at given WPM.
/// Returns duration in milliseconds.
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000 / wpm as u64;
    let modifiers = token.timing_hint.word_length_modifier
        + token.timing_hint.punctuation_modifier
        + token.timing_hint.structure_modifier;

    (base_ms as i64 + modifiers as i64).max(50) as u64
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test timing`
Expected: PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add timing calculation with modifiers"
```

### Task 4.2: Timing Hint Generation

**Files:**
- Modify: `src/timing.rs`
- Modify: `tests/timing_test.rs`

**Step 1: Write timing hint tests**

Add to `tests/timing_test.rs`:
```rust
use rsvp_term::timing::generate_timing_hint;

#[test]
fn test_hint_short_word() {
    let hint = generate_timing_hint("the", false, false);
    assert_eq!(hint.word_length_modifier, 0);
}

#[test]
fn test_hint_long_word() {
    // "beautiful" = 9 chars, 3 extra over 6 = 60ms
    let hint = generate_timing_hint("beautiful", false, false);
    assert_eq!(hint.word_length_modifier, 60);
}

#[test]
fn test_hint_very_long_word() {
    // "extraordinary" = 13 chars
    // 6-10: (13-6)*20 = 140
    // 10+: (13-10)*40 = 120 (additional)
    // total = 140 + 120 = 260... wait let me recalculate
    // chars over 6: 7 extra chars
    // first 4 (7-10): 4 * 20 = 80
    // remaining 3 (10+): 3 * 40 = 120
    // Actually simpler:
    // > 6: (min(len,10) - 6) * 20
    // > 10: (len - 10) * 40
    // For 13: (10-6)*20 + (13-10)*40 = 80 + 120 = 200
    let hint = generate_timing_hint("extraordinary", false, false);
    assert_eq!(hint.word_length_modifier, 200);
}

#[test]
fn test_hint_comma() {
    let hint = generate_timing_hint("word,", false, false);
    assert_eq!(hint.punctuation_modifier, 150);
}

#[test]
fn test_hint_period() {
    let hint = generate_timing_hint("end.", false, false);
    assert_eq!(hint.punctuation_modifier, 200);
}

#[test]
fn test_hint_question() {
    let hint = generate_timing_hint("why?", false, false);
    assert_eq!(hint.punctuation_modifier, 200);
}

#[test]
fn test_hint_paragraph_break() {
    let hint = generate_timing_hint("word", true, false);
    assert_eq!(hint.structure_modifier, 300);
}

#[test]
fn test_hint_new_block() {
    let hint = generate_timing_hint("word", false, true);
    assert_eq!(hint.structure_modifier, 150);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test hint`
Expected: FAIL - function not found

**Step 3: Implement timing hint generation**

Add to `src/timing.rs`:
```rust
use crate::types::TimingHint;

/// Generate timing hints based on word characteristics.
pub fn generate_timing_hint(word: &str, is_paragraph_end: bool, is_new_block: bool) -> TimingHint {
    let len = word.chars().count();

    // Word length modifier
    let word_length_modifier = if len > 10 {
        let base = (10 - 6) * 20; // 80ms for chars 7-10
        let extra = (len - 10) * 40; // 40ms per char over 10
        (base + extra) as i32
    } else if len > 6 {
        ((len - 6) * 20) as i32
    } else {
        0
    };

    // Punctuation modifier (check last char)
    let punctuation_modifier = word.chars().last().map_or(0, |c| match c {
        '.' | '!' | '?' => 200,
        ',' | ':' | ';' => 150,
        _ => 0,
    });

    // Structure modifier
    let structure_modifier = if is_paragraph_end {
        300
    } else if is_new_block {
        150
    } else {
        0
    };

    TimingHint {
        word_length_modifier,
        punctuation_modifier,
        structure_modifier,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test hint`
Expected: PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add timing hint generation"
```

---

## Phase 5: Markdown Parser

### Task 5.1: Parser Trait Definition

**Files:**
- Create: `src/parser/mod.rs`
- Create: `src/parser/traits.rs`
- Modify: `src/lib.rs`

**Step 1: Define parser trait**

Create `src/parser/mod.rs`:
```rust
pub mod traits;
pub mod markdown;

pub use traits::DocumentParser;
pub use markdown::MarkdownParser;
```

Create `src/parser/traits.rs`:
```rust
use crate::types::{Token, Section};
use std::path::Path;

/// Trait for document parsers (enables future EPUB support)
pub trait DocumentParser {
    /// Parse document from file path
    fn parse_file(&self, path: &Path) -> Result<ParsedDocument, ParseError>;

    /// Parse document from string content
    fn parse_str(&self, content: &str) -> Result<ParsedDocument, ParseError>;
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub tokens: Vec<Token>,
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}
```

**Step 2: Update lib.rs**

```rust
pub mod types;
pub mod parser;
pub mod timing;
pub mod orp;
pub mod ui;
pub mod app;
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles (with warnings about unused)

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add parser trait for SOLID extensibility"
```

### Task 5.2: Basic Markdown Parser

**Files:**
- Create: `src/parser/markdown.rs`
- Create: `tests/parser_test.rs`
- Create: `tests/fixtures/simple.md`

**Step 1: Create test fixture**

Create `tests/fixtures/simple.md`:
```markdown
# Hello World

This is a simple paragraph.

## Second Section

Another paragraph here.
```

**Step 2: Write parser tests**

Create `tests/parser_test.rs`:
```rust
use rsvp_term::parser::{MarkdownParser, DocumentParser};
use rsvp_term::types::{TokenStyle, BlockContext};
use std::path::Path;

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
```

**Step 3: Run test to verify it fails**

Run: `cargo test parser`
Expected: FAIL - MarkdownParser not found

**Step 4: Implement basic markdown parser**

Create `src/parser/markdown.rs`:
```rust
use crate::parser::traits::{DocumentParser, ParsedDocument, ParseError};
use crate::types::{Token, TokenStyle, BlockContext, TimingHint, Section};
use crate::timing::generate_timing_hint;
use std::path::Path;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentParser for MarkdownParser {
    fn parse_file(&self, path: &Path) -> Result<ParsedDocument, ParseError> {
        let content = std::fs::read_to_string(path)?;
        self.parse_str(&content)
    }

    fn parse_str(&self, content: &str) -> Result<ParsedDocument, ParseError> {
        let md = &mut markdown_it::MarkdownIt::new();
        markdown_it::plugins::cmark::add(md);
        markdown_it::plugins::extra::add(md);

        let ast = md.parse(content);

        let mut tokens = Vec::new();
        let mut sections = Vec::new();
        let mut current_style = TokenStyle::Normal;
        let mut current_block = BlockContext::Paragraph;
        let mut in_code_block = false;
        let mut skip_content = false;

        self.walk_ast(&ast, &mut tokens, &mut sections, &mut current_style,
                      &mut current_block, &mut in_code_block, &mut skip_content);

        Ok(ParsedDocument { tokens, sections })
    }
}

impl MarkdownParser {
    fn walk_ast(
        &self,
        node: &markdown_it::Node,
        tokens: &mut Vec<Token>,
        sections: &mut Vec<Section>,
        current_style: &mut TokenStyle,
        current_block: &mut BlockContext,
        in_code_block: &mut bool,
        skip_content: &mut bool,
    ) {
        use markdown_it::NodeValue;

        match &node.value {
            NodeValue::CodeBlock(_) | NodeValue::CodeFence(_) => {
                *in_code_block = true;
            }
            NodeValue::Image(_) => {
                *skip_content = true;
            }
            NodeValue::Heading(h) => {
                let level = h.level;
                *current_block = BlockContext::Heading(level);

                // Extract heading text for sections
                let title = self.extract_text(node);
                if !title.is_empty() {
                    sections.push(Section {
                        title,
                        level,
                        token_start: tokens.len(),
                        token_end: tokens.len(), // Updated after processing
                    });
                }
            }
            NodeValue::Paragraph => {
                if !*in_code_block {
                    *current_block = BlockContext::Paragraph;
                }
            }
            NodeValue::BulletList(_) | NodeValue::OrderedList(_) => {
                // Handle in children
            }
            NodeValue::ListItem(_) => {
                *current_block = BlockContext::ListItem(0);
            }
            NodeValue::Blockquote => {
                *current_block = BlockContext::Quote(0);
            }
            NodeValue::Strong => {
                let prev_style = current_style.clone();
                *current_style = match &prev_style {
                    TokenStyle::Italic => TokenStyle::BoldItalic,
                    _ => TokenStyle::Bold,
                };
                for child in node.children.iter() {
                    self.walk_ast(child, tokens, sections, current_style,
                                  current_block, in_code_block, skip_content);
                }
                *current_style = prev_style;
                return;
            }
            NodeValue::Emphasis => {
                let prev_style = current_style.clone();
                *current_style = match &prev_style {
                    TokenStyle::Bold => TokenStyle::BoldItalic,
                    _ => TokenStyle::Italic,
                };
                for child in node.children.iter() {
                    self.walk_ast(child, tokens, sections, current_style,
                                  current_block, in_code_block, skip_content);
                }
                *current_style = prev_style;
                return;
            }
            NodeValue::CodeInline(_) => {
                let prev_style = current_style.clone();
                *current_style = TokenStyle::Code;
                for child in node.children.iter() {
                    self.walk_ast(child, tokens, sections, current_style,
                                  current_block, in_code_block, skip_content);
                }
                *current_style = prev_style;
                return;
            }
            NodeValue::Link(link) => {
                let prev_style = current_style.clone();
                *current_style = TokenStyle::Link(link.url.clone());
                for child in node.children.iter() {
                    self.walk_ast(child, tokens, sections, current_style,
                                  current_block, in_code_block, skip_content);
                }
                *current_style = prev_style;
                return;
            }
            NodeValue::Text(text) => {
                if !*in_code_block && !*skip_content {
                    for word in text.split_whitespace() {
                        let hint = generate_timing_hint(word, false, false);
                        tokens.push(Token {
                            word: word.to_string(),
                            style: current_style.clone(),
                            block: current_block.clone(),
                            timing_hint: hint,
                        });
                    }
                }
            }
            _ => {}
        }

        // Reset flags after processing container
        if matches!(&node.value, NodeValue::CodeBlock(_) | NodeValue::CodeFence(_)) {
            for child in node.children.iter() {
                self.walk_ast(child, tokens, sections, current_style,
                              current_block, in_code_block, skip_content);
            }
            *in_code_block = false;
            return;
        }

        if matches!(&node.value, NodeValue::Image(_)) {
            *skip_content = false;
            return;
        }

        // Update section end after processing heading
        if matches!(&node.value, NodeValue::Heading(_)) {
            if let Some(section) = sections.last_mut() {
                section.token_end = tokens.len();
            }
        }

        for child in node.children.iter() {
            self.walk_ast(child, tokens, sections, current_style,
                          current_block, in_code_block, skip_content);
        }
    }

    fn extract_text(&self, node: &markdown_it::Node) -> String {
        let mut text = String::new();
        self.collect_text(node, &mut text);
        text.trim().to_string()
    }

    fn collect_text(&self, node: &markdown_it::Node, text: &mut String) {
        if let markdown_it::NodeValue::Text(t) = &node.value {
            text.push_str(t);
        }
        for child in node.children.iter() {
            self.collect_text(child, text);
        }
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test parser`
Expected: PASS (or may need adjustments based on markdown-it API)

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add markdown parser with styling support"
```

### Task 5.3: Parser Snapshot Tests

**Files:**
- Create: `tests/snapshots/`
- Modify: `tests/parser_test.rs`

**Step 1: Add snapshot tests**

Add to `tests/parser_test.rs`:
```rust
use insta::assert_debug_snapshot;

#[test]
fn test_snapshot_simple_doc() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str(
        "# Hello\n\nThis is **bold** and *italic* text.\n\n## World\n\nAnother paragraph."
    ).unwrap();

    assert_debug_snapshot!(result.tokens);
    assert_debug_snapshot!(result.sections);
}

#[test]
fn test_snapshot_list() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str(
        "- First item\n- Second item\n- Third item"
    ).unwrap();

    assert_debug_snapshot!(result.tokens);
}

#[test]
fn test_snapshot_quote() {
    let parser = MarkdownParser::new();
    let result = parser.parse_str(
        "> This is a quote\n> with multiple lines"
    ).unwrap();

    assert_debug_snapshot!(result.tokens);
}
```

**Step 2: Run snapshot tests (creates snapshots)**

Run: `cargo insta test`

**Step 3: Review and accept snapshots**

Run: `cargo insta review`

**Step 4: Commit**

```bash
git add -A
git commit -m "test: add parser snapshot tests"
```

---

## Phase 6: Basic TUI

### Task 6.1: App State

**Files:**
- Modify: `src/app.rs`
- Create: `tests/app_test.rs`

**Step 1: Write app state tests**

Create `tests/app_test.rs`:
```rust
use rsvp_term::app::{App, ViewMode};
use rsvp_term::types::{Token, TokenStyle, BlockContext, TimingHint, TimedToken};

fn make_timed_token(word: &str) -> TimedToken {
    TimedToken {
        token: Token {
            word: word.to_string(),
            style: TokenStyle::Normal,
            block: BlockContext::Paragraph,
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
    assert_eq!(app.wpm(), 800);

    // Test lower bound
    for _ in 0..100 {
        app.decrease_wpm();
    }
    assert_eq!(app.wpm(), 100);
}

#[test]
fn test_app_advance() {
    let tokens = vec![
        make_timed_token("hello"),
        make_timed_token("world"),
    ];
    let mut app = App::new(tokens, vec![]);

    assert_eq!(app.position(), 0);
    app.advance();
    assert_eq!(app.position(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test app`
Expected: FAIL - App not found

**Step 3: Implement App state**

Update `src/app.rs`:
```rust
use crate::types::{TimedToken, Section};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Reading,
    Outline,
}

pub struct App {
    tokens: Vec<TimedToken>,
    sections: Vec<Section>,
    position: usize,
    wpm: u16,
    paused: bool,
    view_mode: ViewMode,
    outline_selection: usize,
}

impl App {
    pub fn new(tokens: Vec<TimedToken>, sections: Vec<Section>) -> Self {
        Self {
            tokens,
            sections,
            position: 0,
            wpm: 300,
            paused: false,
            view_mode: ViewMode::Reading,
            outline_selection: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn wpm(&self) -> u16 {
        self.wpm
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn current_token(&self) -> Option<&TimedToken> {
        self.tokens.get(self.position)
    }

    pub fn tokens(&self) -> &[TimedToken] {
        &self.tokens
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn outline_selection(&self) -> usize {
        self.outline_selection
    }

    pub fn progress(&self) -> f64 {
        if self.tokens.is_empty() {
            0.0
        } else {
            self.position as f64 / self.tokens.len() as f64
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn increase_wpm(&mut self) {
        self.wpm = (self.wpm + 25).min(800);
    }

    pub fn decrease_wpm(&mut self) {
        self.wpm = self.wpm.saturating_sub(25).max(100);
    }

    pub fn advance(&mut self) {
        if self.position < self.tokens.len().saturating_sub(1) {
            self.position += 1;
        }
    }

    pub fn rewind_sentence(&mut self) {
        // Simple: go back ~10 words or to start
        self.position = self.position.saturating_sub(10);
    }

    pub fn skip_sentence(&mut self) {
        // Simple: go forward ~10 words
        self.position = (self.position + 10).min(self.tokens.len().saturating_sub(1));
    }

    pub fn toggle_outline(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Reading => ViewMode::Outline,
            ViewMode::Outline => ViewMode::Reading,
        };
    }

    pub fn outline_up(&mut self) {
        self.outline_selection = self.outline_selection.saturating_sub(1);
    }

    pub fn outline_down(&mut self) {
        if !self.sections.is_empty() {
            self.outline_selection = (self.outline_selection + 1).min(self.sections.len() - 1);
        }
    }

    pub fn jump_to_section(&mut self) {
        if let Some(section) = self.sections.get(self.outline_selection) {
            self.position = section.token_start;
            self.view_mode = ViewMode::Reading;
        }
    }

    pub fn current_section_title(&self) -> Option<&str> {
        for section in self.sections.iter().rev() {
            if self.position >= section.token_start {
                return Some(&section.title);
            }
        }
        None
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test app`
Expected: PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add App state management"
```

### Task 6.2: Basic UI Rendering

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/rsvp.rs`
- Create: `src/ui/status.rs`

**Step 1: Create UI module structure**

Create `src/ui/mod.rs`:
```rust
pub mod rsvp;
pub mod status;
pub mod outline;
pub mod context;

use ratatui::Frame;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::ViewMode;
    use ratatui::layout::{Layout, Direction, Constraint};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Main content
            Constraint::Length(2),   // Status bar
        ])
        .split(frame.area());

    match app.view_mode() {
        ViewMode::Reading => {
            rsvp::render(frame, app, chunks[0]);
        }
        ViewMode::Outline => {
            outline::render(frame, app, chunks[0]);
        }
    }

    status::render(frame, app, chunks[1]);
}
```

**Step 2: Create RSVP widget**

Create `src/ui/rsvp.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;
use crate::orp::calculate_orp;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let token = match app.current_token() {
        Some(t) => t,
        None => {
            let block = Block::default().borders(Borders::ALL);
            frame.render_widget(block, area);
            return;
        }
    };

    let word = &token.token.word;
    let orp_pos = calculate_orp(word);

    // Build styled word with ORP highlight
    let chars: Vec<char> = word.chars().collect();
    let mut spans = Vec::new();

    for (i, c) in chars.iter().enumerate() {
        let style = if i == orp_pos {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    // Calculate padding for ORP centering
    let center = area.width as usize / 2;
    let left_padding = center.saturating_sub(orp_pos);
    let padding = " ".repeat(left_padding);

    let mut line_spans = vec![Span::raw(padding)];
    line_spans.extend(spans);

    let paragraph = Paragraph::new(Line::from(line_spans))
        .block(Block::default())
        .alignment(Alignment::Left);

    // Center vertically
    let vertical_center = area.height / 2;
    let rsvp_area = Rect {
        x: area.x,
        y: area.y + vertical_center,
        width: area.width,
        height: 1,
    };

    frame.render_widget(paragraph, rsvp_area);
}
```

**Step 3: Create status bar widget**

Create `src/ui/status.rs`:
```rust
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Gauge},
};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::layout::{Layout, Direction, Constraint};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Section title + progress %
            Constraint::Length(1),  // Progress bar + WPM + state
        ])
        .split(area);

    // Top line: section title and percentage
    let section_title = app.current_section_title().unwrap_or("Document");
    let progress_pct = (app.progress() * 100.0) as u16;
    let top_line = Line::from(vec![
        Span::raw("▸ "),
        Span::styled(section_title, Style::default().fg(Color::Cyan)),
        Span::raw(format!(" {:>3}%", progress_pct)),
    ]);
    frame.render_widget(Paragraph::new(top_line), chunks[0]);

    // Bottom line: progress bar, WPM, pause state
    let pause_indicator = if app.is_paused() { "⏸" } else { "▶" };
    let label = format!("  {} WPM  {}", app.wpm(), pause_indicator);

    let gauge = Gauge::default()
        .ratio(app.progress())
        .label(label)
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray));

    frame.render_widget(gauge, chunks[1]);
}
```

**Step 4: Create outline widget**

Create `src/ui/outline.rs`:
```rust
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.sections()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let prefix = "#".repeat(section.level as usize);
            let text = format!("{} {}", prefix, section.title);

            let style = if i == app.outline_selection() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" OUTLINE ")
            .borders(Borders::ALL));

    frame.render_widget(list, area);
}
```

**Step 5: Create context widget (placeholder)**

Create `src/ui/context.rs`:
```rust
// Context lines rendering - will be implemented in Phase 7
use ratatui::{Frame, layout::Rect};
use crate::app::App;

pub fn render(_frame: &mut Frame, _app: &App, _area: Rect) {
    // TODO: Implement faded context lines
}
```

**Step 6: Update lib.rs**

Remove the placeholder `pub mod ui;` line and ensure proper module structure.

**Step 7: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 8: Commit**

```bash
git add -A
git commit -m "feat: add basic UI rendering (RSVP, status, outline)"
```

### Task 6.3: Event Loop and Input Handling

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement main event loop**

Update `src/main.rs`:
```rust
use clap::Parser as ClapParser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{io::stdout, time::{Duration, Instant}};

use rsvp_term::{
    app::{App, ViewMode},
    parser::{MarkdownParser, DocumentParser},
    timing::calculate_duration,
    orp::calculate_orp,
    types::TimedToken,
    ui,
};

#[derive(ClapParser)]
#[command(name = "rsvp-term")]
#[command(about = "TUI for RSVP reading of markdown prose")]
struct Cli {
    /// Markdown file to read
    file: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Parse document
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&cli.file)?;

    // Convert to timed tokens
    let wpm = 300u16;
    let timed_tokens: Vec<TimedToken> = doc.tokens
        .into_iter()
        .map(|token| {
            let duration = calculate_duration(&token, wpm);
            let orp = calculate_orp(&token.word);
            TimedToken {
                token,
                duration_ms: duration,
                orp_position: orp,
            }
        })
        .collect();

    // Initialize app
    let mut app = App::new(timed_tokens, doc.sections);

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main loop
    let mut last_advance = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Calculate time until next word
        let next_duration = app.current_token()
            .map(|t| Duration::from_millis(t.duration_ms))
            .unwrap_or(Duration::from_millis(200));

        // Handle input with timeout
        let timeout = if app.is_paused() || app.view_mode() == ViewMode::Outline {
            Duration::from_millis(100)
        } else {
            let elapsed = last_advance.elapsed();
            next_duration.saturating_sub(elapsed)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (app.view_mode(), key.code) {
                        // Global
                        (_, KeyCode::Char('q')) => break,
                        (_, KeyCode::Char('?')) => {} // TODO: help overlay

                        // Reading mode
                        (ViewMode::Reading, KeyCode::Char(' ')) => app.toggle_pause(),
                        (ViewMode::Reading, KeyCode::Char('j') | KeyCode::Down) => app.decrease_wpm(),
                        (ViewMode::Reading, KeyCode::Char('k') | KeyCode::Up) => app.increase_wpm(),
                        (ViewMode::Reading, KeyCode::Char('h') | KeyCode::Left) => app.rewind_sentence(),
                        (ViewMode::Reading, KeyCode::Char('l') | KeyCode::Right) => app.skip_sentence(),
                        (ViewMode::Reading, KeyCode::Char('o')) => app.toggle_outline(),

                        // Outline mode
                        (ViewMode::Outline, KeyCode::Char('j') | KeyCode::Down) => app.outline_down(),
                        (ViewMode::Outline, KeyCode::Char('k') | KeyCode::Up) => app.outline_up(),
                        (ViewMode::Outline, KeyCode::Enter) => app.jump_to_section(),
                        (ViewMode::Outline, KeyCode::Esc | KeyCode::Char('o')) => app.toggle_outline(),

                        _ => {}
                    }
                }
            }
        }

        // Advance word if not paused and in reading mode
        if !app.is_paused() && app.view_mode() == ViewMode::Reading {
            if last_advance.elapsed() >= next_duration {
                app.advance();
                last_advance = Instant::now();
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
```

**Step 2: Verify build and manual test**

Run: `cargo build`

Create test file `test.md`:
```markdown
# Hello World

This is a **test** document with *italic* and `code`.

## Second Section

More text here.
```

Run: `cargo run -- test.md`
Expected: TUI appears, words display one at a time

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add event loop and input handling"
```

---

## Phase 7: Context Lines (Faded Document View)

### Task 7.1: Context Line Rendering

**Files:**
- Modify: `src/ui/context.rs`
- Modify: `src/ui/rsvp.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

**Step 1: Add context tracking to App**

Add to `src/app.rs`:
```rust
impl App {
    // ... existing methods ...

    /// Get tokens around current position for context display
    pub fn context_tokens(&self, before: usize, after: usize) -> (&[TimedToken], &[TimedToken]) {
        let start = self.position.saturating_sub(before);
        let end = (self.position + after + 1).min(self.tokens.len());

        let before_slice = &self.tokens[start..self.position];
        let after_slice = if self.position + 1 < end {
            &self.tokens[self.position + 1..end]
        } else {
            &[]
        };

        (before_slice, after_slice)
    }
}
```

**Step 2: Implement context rendering**

Update `src/ui/context.rs`:
```rust
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::App;
use crate::types::TokenStyle;

pub fn render_before(frame: &mut Frame, app: &App, area: Rect) {
    let (before, _) = app.context_tokens(50, 0);
    render_context_lines(frame, before, area, true);
}

pub fn render_after(frame: &mut Frame, app: &App, area: Rect) {
    let (_, after) = app.context_tokens(0, 50);
    render_context_lines(frame, after, area, false);
}

fn render_context_lines(
    frame: &mut Frame,
    tokens: &[crate::types::TimedToken],
    area: Rect,
    fade_up: bool,
) {
    if tokens.is_empty() || area.height == 0 {
        return;
    }

    // Group tokens into lines (rough: ~10 words per line)
    let words_per_line = (area.width as usize / 8).max(5);
    let lines: Vec<Vec<&crate::types::TimedToken>> = tokens
        .chunks(words_per_line)
        .map(|chunk| chunk.iter().collect())
        .collect();

    let num_lines = area.height as usize;
    let display_lines: Vec<_> = if fade_up {
        lines.iter().rev().take(num_lines).collect()
    } else {
        lines.iter().take(num_lines).collect()
    };

    for (i, line_tokens) in display_lines.iter().enumerate() {
        // Calculate fade level (0 = brightest, further = dimmer)
        let distance = if fade_up {
            i
        } else {
            i
        };

        let gray = match distance {
            0 => Color::Rgb(180, 180, 180),
            1 => Color::Rgb(120, 120, 120),
            2 => Color::Rgb(80, 80, 80),
            _ => Color::Rgb(50, 50, 50),
        };

        let spans: Vec<Span> = line_tokens
            .iter()
            .map(|t| {
                let style = Style::default().fg(gray);
                Span::styled(format!("{} ", t.token.word), style)
            })
            .collect();

        let y = if fade_up {
            area.y + area.height - 1 - i as u16
        } else {
            area.y + i as u16
        };

        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };

        frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
    }
}
```

**Step 3: Update main UI to include context**

Update `src/ui/mod.rs`:
```rust
pub mod rsvp;
pub mod status;
pub mod outline;
pub mod context;

use ratatui::Frame;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::ViewMode;
    use ratatui::layout::{Layout, Direction, Constraint};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Main content
            Constraint::Length(2),   // Status bar
        ])
        .split(frame.area());

    match app.view_mode() {
        ViewMode::Reading => {
            render_reading_view(frame, app, chunks[0]);
        }
        ViewMode::Outline => {
            outline::render(frame, app, chunks[0]);
        }
    }

    status::render(frame, app, chunks[1]);
}

fn render_reading_view(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::layout::{Layout, Direction, Constraint};

    // Split into: context above, RSVP line, context below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Context above
            Constraint::Length(3),        // RSVP line (with padding)
            Constraint::Percentage(40),  // Context below
        ])
        .split(area);

    context::render_before(frame, app, chunks[0]);
    rsvp::render(frame, app, chunks[1]);
    context::render_after(frame, app, chunks[2]);
}
```

**Step 4: Verify visually**

Run: `cargo run -- test.md`
Expected: Faded context lines appear above and below the RSVP word

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add faded context lines around RSVP word"
```

---

## Phase 8: Polish and Error Handling

### Task 8.1: Error Handling

**Files:**
- Modify: `src/main.rs`
- Modify: `src/parser/traits.rs`

**Step 1: Improve error messages**

Update main.rs error handling:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Validate file exists
    if !cli.file.exists() {
        eprintln!("Error: File not found: {}", cli.file.display());
        std::process::exit(1);
    }

    // Validate file extension
    let ext = cli.file.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "md" && ext != "markdown" {
        eprintln!("Warning: File may not be markdown: {}", cli.file.display());
    }

    // ... rest of main
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add input validation and error handling"
```

### Task 8.2: Help Overlay

**Files:**
- Create: `src/ui/help.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

**Step 1: Add help state to App**

Add to `src/app.rs`:
```rust
pub struct App {
    // ... existing fields ...
    show_help: bool,
}

impl App {
    pub fn show_help(&self) -> bool {
        self.show_help
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
```

**Step 2: Create help overlay**

Create `src/ui/help.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
};

pub fn render(frame: &mut Frame, area: Rect) {
    // Center the help box
    let width = 50.min(area.width - 4);
    let height = 16.min(area.height - 4);
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    let help_area = Rect { x, y, width, height };

    // Clear background
    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled("CONTROLS", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("Space     Pause/Resume"),
        Line::from("j/↓       Slower (-25 WPM)"),
        Line::from("k/↑       Faster (+25 WPM)"),
        Line::from("h/←       Rewind sentence"),
        Line::from("l/→       Skip sentence"),
        Line::from("H         Rewind paragraph"),
        Line::from("L         Skip paragraph"),
        Line::from("o         Toggle outline"),
        Line::from("q         Quit"),
        Line::from("?         Toggle help"),
        Line::from(""),
        Line::from(Span::styled("Press ? to close", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, help_area);
}
```

**Step 3: Integrate help into main render**

Update `src/ui/mod.rs` render function to call help overlay when active.

**Step 4: Add help toggle to main.rs input handling**

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add help overlay"
```

---

## Phase 9: Final Testing

### Task 9.1: Integration Tests

**Files:**
- Create: `tests/integration_test.rs`

**Step 1: Write integration tests**

```rust
use rsvp_term::parser::{MarkdownParser, DocumentParser};
use rsvp_term::timing::calculate_duration;
use rsvp_term::orp::calculate_orp;
use rsvp_term::types::TimedToken;
use rsvp_term::app::App;

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
    assert!(doc.sections.len() >= 2);

    let timed: Vec<TimedToken> = doc.tokens.into_iter().map(|t| {
        TimedToken {
            duration_ms: calculate_duration(&t, 300),
            orp_position: calculate_orp(&t.word),
            token: t,
        }
    }).collect();

    let mut app = App::new(timed, doc.sections);

    // Test navigation
    assert_eq!(app.position(), 0);
    app.advance();
    assert_eq!(app.position(), 1);

    // Test WPM
    app.increase_wpm();
    assert_eq!(app.wpm(), 325);
}
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Commit**

```bash
git add -A
git commit -m "test: add integration tests"
```

### Task 9.2: README

**Files:**
- Create: `README.md`

**Step 1: Write README**

```markdown
# rsvp-term

A TUI for RSVP (Rapid Serial Visual Presentation) reading of markdown prose.

## Features

- ORP-centered word display (Spritz-style)
- Markdown styling preserved (bold, italic, code, links)
- Faded document context around current word
- Heading outline navigation
- Vim-style controls

## Installation

```bash
cargo install --path .
```

## Usage

```bash
rsvp-term document.md
```

## Controls

| Key | Action |
|-----|--------|
| Space | Pause/Resume |
| j/↓ | Slower (-25 WPM) |
| k/↑ | Faster (+25 WPM) |
| h/← | Rewind sentence |
| l/→ | Skip sentence |
| o | Toggle outline |
| q | Quit |
| ? | Help |

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README"
```

---

## Execution Summary

**Total Tasks:** 20+
**Phases:** 9

1. Project Setup (1 task)
2. Core Types (1 task)
3. ORP Calculation (1 task)
4. Timing Logic (2 tasks)
5. Markdown Parser (3 tasks)
6. Basic TUI (3 tasks)
7. Context Lines (1 task)
8. Polish (2 tasks)
9. Final Testing (2 tasks)
