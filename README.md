# rsvp-term

A TUI for RSVP (Rapid Serial Visual Presentation) reading of markdown prose.

## Features

- ORP-centered word display (Spritz-style)
- Markdown styling preserved (bold, italic, code, links)
- Faded document context around current word
- Heading outline navigation
- Vim-style controls

## Installation

```bash
cargo install --path .
```

## Usage

```bash
rsvp-term document.md
```

## Controls

| Key | Action |
|-----|--------|
| Space | Pause/Resume |
| j/↓ | Slower (-25 WPM) |
| k/↑ | Faster (+25 WPM) |
| h/← | Rewind sentence |
| l/→ | Skip sentence |
| o | Toggle outline |
| q | Quit |
| ? | Help |

## License

MIT
