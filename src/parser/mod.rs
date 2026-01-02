pub mod markdown;
pub mod traits;

pub use markdown::MarkdownParser;
pub use traits::{DocumentParser, ParseError, ParsedDocument};
