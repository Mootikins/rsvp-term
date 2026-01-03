# Code Review Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address all code review findings to achieve idiomatic Rust per the project standards.

**Architecture:** Incremental fixes organized by module, progressing from critical type safety issues through API polish to documentation improvements. Each task is independently testable and committable.

**Tech Stack:** Rust 2021 edition, Clippy pedantic/nursery lints

---

## Phase 1: Critical Type Safety Fixes

### Task 1: Add `Eq` Derivations to Types

**Files:**
- Modify: `src/types.rs:1,11,21`

**Step 1: Update TokenStyle derive**

```rust
// src/types.rs:1
// Change from:
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStyle {

// To:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStyle {
```

**Step 2: Update BlockContext derive**

```rust
// src/types.rs:11
// Change from:
#[derive(Debug, Clone, PartialEq)]
pub enum BlockContext {

// To:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockContext {
```

**Step 3: Update TimingHint derive**

```rust
// src/types.rs:21
// Change from:
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TimingHint {

// To:
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TimingHint {
```

**Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 6: Verify Clippy warnings resolved**

Run: `cargo clippy -- -W clippy::derive-partial-eq-without-eq 2>&1 | grep derive_partial`
Expected: No output (warnings resolved)

**Step 7: Commit**

```bash
git add src/types.rs
git commit -m "fix: add Eq derivation to PartialEq types

Clippy pedantic requires Eq when PartialEq is derived for types
that can implement it. All three types (TokenStyle, BlockContext,
TimingHint) have no floating-point or other non-Eq fields.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Fix Cast Safety in timing.rs

**Files:**
- Modify: `src/timing.rs:10-17`

**Step 1: Read current implementation**

Current code at lines 10-17:
```rust
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000 / wpm as u64;
    let modifiers = token.timing_hint.word_length_modifier
        + token.timing_hint.punctuation_modifier
        + token.timing_hint.structure_modifier;

    (base_ms as i64 + modifiers as i64).max(50) as u64
}
```

**Step 2: Replace with safe casts**

```rust
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
    let base_ms = 60_000_u64 / u64::from(wpm);
    let modifiers = i64::from(token.timing_hint.word_length_modifier)
        + i64::from(token.timing_hint.punctuation_modifier)
        + i64::from(token.timing_hint.structure_modifier);

    // Safe: base_ms is at most 60000, modifiers are small i32s
    // Result after max(50) is always positive
    #[allow(clippy::cast_sign_loss)]
    let result = (i64::try_from(base_ms).unwrap_or(i64::MAX) + modifiers).max(50) as u64;
    result
}
```

**Step 3: Run timing tests**

Run: `cargo test timing`
Expected: All 14 timing tests pass

**Step 4: Verify Clippy cast warnings resolved**

Run: `cargo clippy -- -W clippy::cast-lossless 2>&1 | grep -c "timing.rs"`
Expected: 0 (no more cast warnings in timing.rs for the fixed lines)

**Step 5: Commit**

```bash
git add src/timing.rs
git commit -m "fix: use safe casts in calculate_duration

Replace as-casts with From/TryFrom for type-safe conversions.
Add explicit allow for the final cast where we've proven safety.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Fix Cast Safety in generate_timing_hint

**Files:**
- Modify: `src/timing.rs:20-55`

**Step 1: Update word_length_modifier calculation**

```rust
// Lines 24-32, change from:
let word_length_modifier = if len > 10 {
    let base = (10 - 6) * 10; // 40ms for chars 7-10
    let extra = (len - 10) * 20; // 20ms per char over 10
    (base + extra) as i32
} else if len > 6 {
    ((len - 6) * 10) as i32
} else {
    0
};

// To (safe bounded conversion):
let word_length_modifier: i32 = if len > 10 {
    let base = (10 - 6) * 10; // 40ms for chars 7-10
    let extra = (len - 10) * 20; // 20ms per char over 10
    // Cap at reasonable max (words over ~100 chars are rare)
    i32::try_from(base + extra).unwrap_or(1000)
} else if len > 6 {
    i32::try_from((len - 6) * 10).unwrap_or(40)
} else {
    0
};
```

**Step 2: Run timing tests**

Run: `cargo test timing`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/timing.rs
git commit -m "fix: use safe casts in generate_timing_hint

Use try_from with fallback for usize to i32 conversion.
Values are bounded by realistic word lengths.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 2: API Polish - #[must_use] Attributes

### Task 4: Add #[must_use] to Parser Constructors

**Files:**
- Modify: `src/parser/epub.rs:15-20`
- Modify: `src/parser/markdown.rs:30-36`

**Step 1: Add to EpubParser::new**

```rust
// src/parser/epub.rs, before line 16:
#[must_use]
pub fn new() -> Self {
```

**Step 2: Add to MarkdownParser::new**

```rust
// src/parser/markdown.rs, before line 31:
#[must_use]
pub fn new() -> Self {
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/parser/epub.rs src/parser/markdown.rs
git commit -m "fix: add #[must_use] to parser constructors

Pure constructor functions should have must_use to catch
accidental discarding of the return value.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Add #[must_use] to Pure Functions

**Files:**
- Modify: `src/timing.rs:10,20`
- Modify: `src/orp.rs:16`

**Step 1: Add to calculate_duration**

```rust
// src/timing.rs, before line 10:
#[must_use]
pub fn calculate_duration(token: &Token, wpm: u16) -> u64 {
```

**Step 2: Add to generate_timing_hint**

```rust
// src/timing.rs, before line 20 (adjusted for earlier edits):
#[must_use]
pub fn generate_timing_hint(word: &str, is_paragraph_end: bool, is_new_block: bool) -> TimingHint {
```

**Step 3: Add to calculate_orp**

```rust
// src/orp.rs, before line 16:
#[must_use]
pub fn calculate_orp(word: &str) -> usize {
```

**Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/timing.rs src/orp.rs
git commit -m "fix: add #[must_use] to pure calculation functions

These functions have no side effects; discarding their result
is always a bug.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 3: Error Documentation

### Task 6: Document Errors on Trait Methods

**Files:**
- Modify: `src/parser/traits.rs:4-11`

**Step 1: Add error documentation to parse_file**

```rust
// src/parser/traits.rs, replace lines 4-7:
/// Trait for document parsers (enables future EPUB support)
pub trait DocumentParser {
    /// Parse document from file path.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::IoError`] if the file cannot be read.
    /// Returns [`ParseError::ParseError`] if the content is malformed.
    fn parse_file(&self, path: &Path) -> Result<ParsedDocument, ParseError>;

    /// Parse document from string content.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::ParseError`] if the content is malformed.
    fn parse_str(&self, content: &str) -> Result<ParsedDocument, ParseError>;
}
```

**Step 2: Verify doc tests compile**

Run: `cargo test --doc`
Expected: Passes (no doc tests to run, but validates doc syntax)

**Step 3: Commit**

```bash
git add src/parser/traits.rs
git commit -m "docs: add Errors section to DocumentParser trait methods

Per Rust API Guidelines, Result-returning functions must document
their error conditions.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Document Errors on export_chapters

**Files:**
- Modify: `src/parser/epub.rs:62-66`

**Step 1: Add error documentation**

```rust
// src/parser/epub.rs, before export_chapters function:
/// Export each chapter as a separate markdown file.
///
/// Creates a directory named after the book title and writes each
/// chapter as a numbered markdown file.
///
/// # Errors
///
/// Returns [`ParseError::ParseError`] if the EPUB cannot be opened.
/// Returns [`ParseError::IoError`] if directory creation or file writing fails.
pub fn export_chapters(&self, path: &Path) -> Result<(String, usize), ParseError> {
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/parser/epub.rs
git commit -m "docs: add Errors section to export_chapters

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 4: Idiomatic Pattern Fixes

### Task 8: Use let...else in rsvp.rs

**Files:**
- Modify: `src/ui/rsvp.rs:10-18`

**Step 1: Replace match with let...else**

```rust
// src/ui/rsvp.rs, replace lines 10-18:
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let Some(token) = app.current_token() else {
        let block = Block::default().borders(Borders::ALL);
        frame.render_widget(block, area);
        return;
    };
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/ui/rsvp.rs
git commit -m "refactor: use let...else pattern in rsvp render

More idiomatic Rust for early returns with pattern matching.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Use let...else in epub.rs (two locations)

**Files:**
- Modify: `src/parser/epub.rs:81-84,130-133`

**Step 1: Fix first location (export_chapters loop)**

```rust
// src/parser/epub.rs, replace lines 81-84:
            let Some((content, _mime)) = doc.get_current_str() else { continue };
```

**Step 2: Fix second location (parse_file loop)**

```rust
// src/parser/epub.rs, replace lines 130-133:
            let Some((content, _mime)) = doc.get_current_str() else { continue };
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Run epub tests**

Run: `cargo test epub`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/parser/epub.rs
git commit -m "refactor: use let...else pattern in epub parser

Replace match-with-continue patterns with idiomatic let...else.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 10: Use Self in ParseError Pattern Matching

**Files:**
- Modify: `src/parser/traits.rs:26-45`

**Step 1: Update Display impl**

```rust
// src/parser/traits.rs, replace lines 26-31:
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}
```

**Step 2: Update Error impl**

```rust
// src/parser/traits.rs, replace lines 34-41:
impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::ParseError(_) => None,
        }
    }
}
```

**Step 3: Update From impl**

```rust
// src/parser/traits.rs, replace lines 43-46:
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
```

**Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/parser/traits.rs
git commit -m "refactor: use Self instead of type name in ParseError

More idiomatic and maintainable pattern matching.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 5: Format String Improvements

### Task 11: Inline Format Arguments (epub.rs)

**Files:**
- Modify: `src/parser/epub.rs` (multiple locations)

**Step 1: Fix line 65**

```rust
// Change:
ParseError::ParseError(format!("Failed to open EPUB: {}", e))
// To:
ParseError::ParseError(format!("Failed to open EPUB: {e}"))
```

**Step 2: Fix line 98**

```rust
// Change:
.unwrap_or_else(|| format!("chapter-{:02}", exported_count))
// To:
.unwrap_or_else(|| format!("chapter-{exported_count:02}"))
```

**Step 3: Fix line 101**

```rust
// Change:
let filename = format!("{:02}-{}.md", exported_count, chapter_title);
// To:
let filename = format!("{exported_count:02}-{chapter_title}.md");
```

**Step 4: Fix line 120**

```rust
// Change:
ParseError::ParseError(format!("Failed to open EPUB: {}", e))
// To:
ParseError::ParseError(format!("Failed to open EPUB: {e}"))
```

**Step 5: Fix lines 139-142**

```rust
// Change:
format!("Failed to parse chapter {}: malformed XHTML", title)
// To:
format!("Failed to parse chapter {title}: malformed XHTML")
```

**Step 6: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 7: Commit**

```bash
git add src/parser/epub.rs
git commit -m "style: inline format arguments in epub.rs

Use Rust 2021 format string syntax for cleaner code.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 12: Inline Format Arguments (traits.rs)

**Files:**
- Modify: `src/parser/traits.rs:28-29`

**Step 1: Already fixed in Task 10**

These were already updated to use `{e}` and `{s}` syntax.

**Step 2: Verify with Clippy**

Run: `cargo clippy -- -W clippy::uninlined-format-args 2>&1 | grep traits.rs`
Expected: No output

---

### Task 13: Inline Format Arguments (status.rs)

**Files:**
- Modify: `src/ui/status.rs:27`

**Step 1: Fix format string**

```rust
// Change:
Span::raw(format!(" {:>3}%", progress_pct)),
// To:
Span::raw(format!(" {progress_pct:>3}%")),
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/ui/status.rs
git commit -m "style: inline format argument in status.rs

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 14: Use write! Instead of format_push_string (epub.rs)

**Files:**
- Modify: `src/parser/epub.rs:153`

**Step 1: Add use statement if not present**

```rust
// At top of file, add:
use std::fmt::Write;
```

**Step 2: Replace push_str with write!**

```rust
// Change line 153:
combined_markdown.push_str(&format!("# {}\n\n", title));
// To:
let _ = write!(combined_markdown, "# {title}\n\n");
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/parser/epub.rs
git commit -m "perf: use write! instead of format! + push_str

Avoids intermediate String allocation.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 6: Const Function Improvements

### Task 15: Add const fn to markdown.rs

**Files:**
- Modify: `src/parser/markdown.rs:115-117`

**Step 1: Add const keyword**

```rust
// Change:
fn should_skip(&self) -> bool {
    self.skip_depth > 0
}
// To:
const fn should_skip(&self) -> bool {
    self.skip_depth > 0
}
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/parser/markdown.rs
git commit -m "perf: mark should_skip as const fn

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 16: Add const fn to context.rs

**Files:**
- Modify: `src/ui/context.rs:110-115,118-127`

**Step 1: Add const to table_row**

```rust
// Change line 110:
fn table_row(block: &BlockContext) -> Option<usize> {
// To:
const fn table_row(block: &BlockContext) -> Option<usize> {
```

**Step 2: Add const to block_prefix**

```rust
// Change line 118:
fn block_prefix(block: &BlockContext) -> &'static str {
// To:
const fn block_prefix(block: &BlockContext) -> &'static str {
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/ui/context.rs
git commit -m "perf: mark helper functions as const fn

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 7: Match Arm Consolidation

### Task 17: Merge Identical Match Arms in context.rs

**Files:**
- Modify: `src/ui/context.rs:118-127`

**Step 1: Consolidate block_prefix match arms**

```rust
// Change lines 118-127:
const fn block_prefix(block: &BlockContext) -> &'static str {
    match block {
        BlockContext::ListItem(_) => "* ",
        BlockContext::Quote(_) | BlockContext::TableCell(_) => "| ",
        BlockContext::Heading(_) | BlockContext::Paragraph => "",
        BlockContext::Callout(_) => "[i] ",
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/ui/context.rs
git commit -m "refactor: consolidate identical match arms

Merge Quote/TableCell (both '| ') and Heading/Paragraph (both '').

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 8: Documentation Polish

### Task 18: Add Backticks to Doc Comments

**Files:**
- Modify: `src/parser/markdown.rs:30,191`

**Step 1: Fix line 30**

```rust
// Change:
/// Create a new markdown parser with CommonMark and GFM table support.
// To:
/// Create a new markdown parser with `CommonMark` and GFM table support.
```

**Step 2: Fix line 191**

```rust
// Change:
/// Returns (restore_style, restore_block, restore_skip, restore_list_depth, restore_quote_depth)
// To:
/// Returns (`restore_style`, `restore_block`, `restore_skip`, `restore_list_depth`, `restore_quote_depth`)
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/parser/markdown.rs
git commit -m "docs: add backticks around code terms in doc comments

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 9: Final Verification

### Task 19: Full Clippy Check

**Step 1: Run full Clippy with pedantic**

Run: `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery 2>&1 | head -50`
Expected: Significantly fewer warnings than before (some may remain for casts in UI code)

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Build release**

Run: `cargo build --release`
Expected: Builds successfully

---

### Task 20: Create Summary Commit

**Step 1: Verify git log**

Run: `git log --oneline -20`
Expected: See all the individual fix commits

**Step 2: No squash needed**

Individual commits are preferred for this type of cleanup work - they make bisecting easier if any issue is discovered later.

---

## Appendix: Files Modified Summary

| File | Changes |
|------|---------|
| `src/types.rs` | Add `Eq` derivations |
| `src/timing.rs` | Safe casts, `#[must_use]` |
| `src/orp.rs` | `#[must_use]` |
| `src/parser/traits.rs` | Error docs, `Self` usage, inline format |
| `src/parser/epub.rs` | `#[must_use]`, `let...else`, inline format, `write!` |
| `src/parser/markdown.rs` | `#[must_use]`, `const fn`, doc backticks |
| `src/ui/rsvp.rs` | `let...else` |
| `src/ui/status.rs` | Inline format |
| `src/ui/context.rs` | `const fn`, merge match arms |

---

## Not Addressed in This Plan

The following items from the review are **intentionally deferred**:

1. **UI module tests** - Requires ratatui test harness setup, separate effort
2. **Builder pattern for App** - Larger refactor, not a code smell
3. **Newtype for Wpm** - API change, needs design discussion
4. **thiserror migration** - Works fine as-is, optional improvement
5. **Context line caching** - Performance optimization, needs profiling first
6. **Cast warnings in UI code** - Low risk, would require significant refactor
