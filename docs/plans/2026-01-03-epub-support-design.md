# EPUB Support Design

## Overview

Add EPUB file support to rsvp-term, enabling RSVP reading of ebooks and optional export to markdown.

## Architecture

```
EpubParser::parse_file(path)
  -> epub::EpubDoc::new(path)
  -> iterate chapters via spine
  -> html2text::from_read() each chapter -> markdown
  -> concatenate with heading separators
  -> MarkdownParser::parse_str(combined)
  -> return ParsedDocument
```

## Dependencies

```toml
epub = "2.1"
html2text = "0.12"
```

## File Changes

| File | Change |
|------|--------|
| `Cargo.toml` | Add epub, html2text dependencies |
| `src/parser/epub.rs` | New `EpubParser` implementing `DocumentParser` |
| `src/parser/mod.rs` | Export `EpubParser` |
| `src/main.rs` | Detect `.epub` extension, add `--export-md` flag |

## CLI Usage

```bash
rsvp-term book.epub              # RSVP reading mode
rsvp-term book.epub --export-md  # Export chapters to markdown
```

## Export Behavior

- Creates folder: `{book-title}/` (sanitized from metadata or filename)
- Writes files: `01-{chapter-name}.md`, `02-{chapter-name}.md`, ...
- Chapter names from EPUB TOC, fallback to `chapter-NN`
- Exits after export, prints summary

## Error Handling

- Invalid/corrupt EPUB: `ParseError::ParseError("Invalid EPUB: ...")`
- XHTML parse failure: `ParseError::ParseError("Failed to parse chapter N: {title}")` - fail fast
- Empty EPUB: Return empty `ParsedDocument`
- DRM: Not handled (deferred)

## Content Handling

- Images: Omitted (already skipped by MarkdownParser)
- Chapter titles: Become `# Heading` markers
- XHTML structure: Preserved via html2text -> markdown conversion

## Implementation Notes

- `EpubParser` has two entry points:
  - `parse_file()` for RSVP mode (returns ParsedDocument)
  - `export_chapters()` for markdown export (writes files)
- Reuses existing MarkdownParser for token generation
- All timing, ORP, section logic unchanged
