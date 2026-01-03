use crate::types::{Section, Token};
use std::path::Path;

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

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::ParseError(_) => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
