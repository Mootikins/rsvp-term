# rsvp-term

A terminal UI for RSVP (Rapid Serial Visual Presentation) speed reading of Markdown and EPUB files.

![Demo](assets/demo.gif)

## What is RSVP?

RSVP displays text one word at a time at a fixed position, eliminating eye movement and enabling faster reading. This implementation uses ORP (Optimal Recognition Point) highlighting - the letter your eye naturally focuses on is highlighted in red and centered.

## Features

- **ORP-centered display** - Spritz-style word presentation with optimal recognition point
- **Markdown support** - Parses CommonMark with GFM tables
- **EPUB support** - Read EPUB books directly, or export chapters to Markdown
- **Context display** - Faded surrounding text above/below current word
- **Outline navigation** - Jump between sections via heading outline
- **Adaptive timing** - Longer words, punctuation, and paragraph breaks get extra display time
- **Vim-style controls** - Familiar keybindings for navigation

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/rsvp-term document.md
```

## Usage

```bash
# Read a Markdown file
rsvp-term document.md

# Read an EPUB book
rsvp-term book.epub

# Export EPUB chapters to Markdown files
rsvp-term book.epub --export-md
```

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume |
| `j` / `↓` | Slower (-25 WPM) |
| `k` / `↑` | Faster (+25 WPM) |
| `h` / `←` | Rewind ~10 words |
| `l` / `→` | Skip ~10 words |
| `o` | Toggle outline view |
| `Enter` | Jump to section (in outline) |
| `q` | Quit |
| `?` | Toggle help |
| `Ctrl+C` | Force quit |

## How It Works

1. **Parsing** - Markdown/EPUB is parsed into tokens with style (bold, italic, code, link) and block context (paragraph, list, quote, heading)

2. **Timing** - Each word gets a base duration (60000ms / WPM) plus modifiers:
   - Long words: +10-20ms per character over 6
   - Punctuation: +150-200ms for commas, periods, etc.
   - Structure: +150-300ms for new blocks/paragraphs

3. **ORP Calculation** - The optimal recognition point is ~1/3 into the word:
   - 1-3 chars: position 0
   - 4-6 chars: position 1
   - 7-9 chars: position 2
   - 10+ chars: position 3

4. **Display** - Word is centered on ORP position, with context lines fading by distance

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal handling
- [markdown-it](https://github.com/nickklemm/markdown-it) - CommonMark parser
- [epub](https://github.com/nickklemm/epub) - EPUB reader
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing

## License

MIT
