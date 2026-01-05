use std::fmt::Write;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use epub::doc::EpubDoc;

use super::markdown::MarkdownParser;
use super::traits::{DocumentParser, ParseError, ParsedDocument};

/// EPUB parser that extracts content and converts to tokens via markdown.
pub struct EpubParser {
    md_parser: MarkdownParser,
}

impl EpubParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            md_parser: MarkdownParser::new(),
        }
    }

    /// Convert XHTML content to markdown using html2text.
    fn xhtml_to_markdown(xhtml: &str) -> String {
        html2text::from_read(Cursor::new(xhtml), 10000).unwrap_or_default()
    }

    /// Sanitize a string for use as a filename.
    fn sanitize_filename(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Get the book title from EPUB metadata or filename.
    fn get_book_title(doc: &EpubDoc<std::io::BufReader<std::fs::File>>, path: &Path) -> String {
        doc.mdata("title")
            .map(|item| Self::sanitize_filename(&item.value))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(Self::sanitize_filename)
                    .unwrap_or_else(|| "epub-export".to_string())
            })
    }

    /// Get chapter title from TOC for current chapter index.
    fn get_chapter_title(
        doc: &EpubDoc<std::io::BufReader<std::fs::File>>,
        chapter_idx: usize,
    ) -> Option<String> {
        // Get the current spine item's path
        let spine_item = doc.spine.get(chapter_idx)?;
        let resource = doc.resources.get(&spine_item.idref)?;
        let resource_path = resource.path.to_string_lossy();

        // Find matching TOC entry
        doc.toc
            .iter()
            .find(|nav| nav.content.ends_with(resource_path.as_ref()))
            .map(|nav| nav.label.clone())
    }

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
        let mut doc = EpubDoc::new(path)
            .map_err(|e| ParseError::ParseError(format!("Failed to open EPUB: {e}")))?;

        let book_title = Self::get_book_title(&doc, path);
        let output_dir = Path::new(&book_title);

        // Create output directory
        fs::create_dir_all(output_dir)?;

        let num_chapters = doc.get_num_chapters();
        let mut exported_count = 0;

        for i in 0..num_chapters {
            doc.set_current_chapter(i);

            // Get chapter content
            let Some((content, _mime)) = doc.get_current_str() else {
                continue; // Skip empty chapters
            };

            // Convert to markdown
            let markdown = Self::xhtml_to_markdown(&content);
            if markdown.trim().is_empty() {
                continue; // Skip empty content
            }

            exported_count += 1;

            // Get chapter title from TOC or use fallback
            let chapter_title = Self::get_chapter_title(&doc, i)
                .map(|t| Self::sanitize_filename(&t))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("chapter-{exported_count:02}"));

            // Write chapter file
            let filename = format!("{exported_count:02}-{chapter_title}.md");
            let filepath = output_dir.join(&filename);

            fs::write(&filepath, &markdown)?;
        }

        Ok((book_title, exported_count))
    }
}

impl Default for EpubParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentParser for EpubParser {
    fn parse_file(&self, path: &Path) -> Result<ParsedDocument, ParseError> {
        let mut doc = EpubDoc::new(path)
            .map_err(|e| ParseError::ParseError(format!("Failed to open EPUB: {e}")))?;

        let mut combined_markdown = String::new();
        let num_chapters = doc.get_num_chapters();

        for i in 0..num_chapters {
            doc.set_current_chapter(i);

            // Get chapter content
            let Some((content, _mime)) = doc.get_current_str() else {
                continue;
            };

            // Check for malformed XHTML - fail fast
            if content.contains("<parsererror") {
                let title = Self::get_chapter_title(&doc, i)
                    .unwrap_or_else(|| format!("chapter {}", i + 1));
                return Err(ParseError::ParseError(format!(
                    "Failed to parse chapter {title}: malformed XHTML"
                )));
            }

            // Try to get chapter title from TOC
            let chapter_title = Self::get_chapter_title(&doc, i);

            // Add chapter heading if we have a title
            if let Some(title) = chapter_title {
                if !combined_markdown.is_empty() {
                    combined_markdown.push_str("\n\n");
                }
                let _ = write!(combined_markdown, "# {title}\n\n");
            }

            // Convert XHTML to markdown
            let markdown = Self::xhtml_to_markdown(&content);
            if markdown.trim().is_empty() {
                continue;
            }

            combined_markdown.push_str(&markdown);
        }

        // Parse combined markdown through the markdown parser
        self.md_parser.parse_str(&combined_markdown)
    }

    fn parse_str(&self, _content: &str) -> Result<ParsedDocument, ParseError> {
        Err(ParseError::ParseError(
            "EPUB parser does not support parsing from string".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(EpubParser::sanitize_filename("Hello World"), "Hello World");
        assert_eq!(
            EpubParser::sanitize_filename("Book: A Story"),
            "Book_ A Story"
        );
        assert_eq!(
            EpubParser::sanitize_filename("Test/File\\Name"),
            "Test_File_Name"
        );
    }

    #[test]
    fn test_xhtml_to_markdown() {
        let xhtml = "<p>Hello <strong>world</strong>!</p>";
        let md = EpubParser::xhtml_to_markdown(xhtml);
        assert!(md.contains("Hello"));
        assert!(md.contains("world"));
    }
}
