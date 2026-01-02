use crate::types::{Section, Token};
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
