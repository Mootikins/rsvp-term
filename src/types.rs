#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockHint {
    Heading(u8),
    Quote,
    BulletList,
    OrderedList,
    Table,
    Callout(String),
}

impl BlockHint {
    /// Returns the hint characters for display in the gutter
    #[must_use]
    pub fn hint_chars(&self) -> &str {
        match self {
            BlockHint::Heading(1) => "#",
            BlockHint::Heading(2) => "##",
            BlockHint::Heading(3) => "###",
            BlockHint::Heading(4) => "####",
            BlockHint::Heading(5) => "#####",
            BlockHint::Heading(6) => "######",
            BlockHint::Heading(_) => "#",
            BlockHint::Quote => ">",
            BlockHint::BulletList => "-",
            BlockHint::OrderedList => "1.",
            BlockHint::Table => "|",
            BlockHint::Callout(_) => "[!]",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Link(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockContext {
    Paragraph,
    ListItem(usize),  // depth
    Quote(usize),     // depth
    Callout(String),  // type
    Heading(u8),      // level 1-6
    TableCell(usize), // table cell with row number (0-indexed)
}

impl BlockContext {
    /// Returns the hint characters for display in the gutter
    #[must_use]
    pub fn hint_chars(&self) -> &'static str {
        match self {
            BlockContext::Heading(1) => "#",
            BlockContext::Heading(2) => "##",
            BlockContext::Heading(3) => "###",
            BlockContext::Heading(4) => "####",
            BlockContext::Heading(5) => "#####",
            BlockContext::Heading(6) => "######",
            BlockContext::Heading(_) => "#",
            BlockContext::ListItem(_) => "",
            BlockContext::Quote(_) => ">",
            BlockContext::TableCell(_) => "|",
            BlockContext::Callout(_) => "[!]",
            BlockContext::Paragraph => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TimingHint {
    pub word_length_modifier: i32,
    pub punctuation_modifier: i32,
    pub structure_modifier: i32,
    /// True if this is the first word of a table cell (for rendering separators)
    pub is_cell_start: bool,
    /// Column index for table cells (0-indexed), None if not in a table
    pub table_column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub word: String,
    pub style: TokenStyle,
    pub block: BlockContext,
    pub parent_context: Option<BlockHint>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_hint_display() {
        assert_eq!(BlockHint::Heading(1).hint_chars(), "#");
        assert_eq!(BlockHint::Heading(2).hint_chars(), "##");
        assert_eq!(BlockHint::Quote.hint_chars(), ">");
        assert_eq!(BlockHint::BulletList.hint_chars(), "-");
        assert_eq!(BlockHint::OrderedList.hint_chars(), "1.");
        assert_eq!(BlockHint::Table.hint_chars(), "|");
        assert_eq!(BlockHint::Callout("note".into()).hint_chars(), "[!]");
    }
}
