pub mod epub;
pub mod markdown;
pub mod traits;

pub use epub::EpubParser;
pub use markdown::MarkdownParser;
pub use traits::{DocumentParser, ParseError, ParsedDocument};
