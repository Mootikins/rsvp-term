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
    TableCell,          // table cell (cells separated by |, rows by newline)
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
