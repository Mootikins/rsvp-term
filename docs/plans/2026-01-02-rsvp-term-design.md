# rsvp-term Design

A TUI for RSVP (Rapid Serial Visual Presentation) reading of markdown prose.

## Overview

Display markdown documents one word at a time using ORP-centered (Spritz-style) presentation. The current word appears on a full-width line with surrounding document context faded, providing spatial awareness while maintaining focus.

## Tech Stack

- **Language**: Rust
- **TUI**: Ratatui + Crossterm
- **Markdown**: markdown-it (Rust port) for extensibility
- **Testing**: insta for snapshot testing, TDD approach
- **CLI**: clap

## Architecture (SOLID)

```
┌──────────────────────────────────────────────────────────────┐
│                        rsvp-term                             │
├──────────────────────────────────────────────────────────────┤
│  main.rs           CLI entry, arg parsing                    │
│  app.rs            App state, event loop orchestration       │
│  parser.rs         Markdown → Token stream (Single Resp.)    │
│  timing.rs         Tokens → Timed tokens (Single Resp.)      │
│  orp.rs            ORP calculation logic                     │
│  ui/mod.rs         Rendering coordination                    │
│  ui/rsvp.rs        RSVP line widget                          │
│  ui/context.rs     Faded context lines widget                │
│  ui/outline.rs     Heading outline widget                    │
│  ui/status.rs      Status bar widget                         │
└──────────────────────────────────────────────────────────────┘
```

**Data flow:**
1. Load markdown file from CLI arg
2. Parse to token stream (words with styling + block context)
3. Calculate timing for each token
4. Build heading outline for navigation
5. Enter TUI event loop: render current state, handle input, advance on timer

## Core Types

```rust
enum TokenStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Code,           // inline `code`
    Link(String),   // underlined, URL stored
}

enum BlockContext {
    Paragraph,
    ListItem(usize),    // depth
    Quote(usize),       // depth
    Callout(String),    // type: note, warning, etc.
    Heading(u8),        // level 1-6
}

struct TimingHint {
    base_modifier: i32,     // extra ms from word length
    punctuation: i32,       // extra ms from punctuation
    structure: i32,         // extra ms from paragraph/block breaks
}

struct Token {
    word: String,
    style: TokenStyle,
    block: BlockContext,
    timing_hint: TimingHint,
}

struct TimedToken {
    token: Token,
    duration_ms: u64,
    orp_position: usize,
}

struct Section {
    title: String,
    level: u8,
    token_range: Range<usize>,
}

struct AppState {
    tokens: Vec<TimedToken>,
    sections: Vec<Section>,
    position: usize,
    wpm: u16,
    paused: bool,
    view_mode: ViewMode,  // Reading | Outline
}
```

## UI Layout

### Reading Mode

```
┌─────────────────────────────────────────────────────────┐
│ ░░░░░░░░░░ faded context line above ░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░░ faded context line above ░░░░░░░░░░░░░░░░░░░ │
│                                                         │
│              the qui|c|k brown fox                      │  ← RSVP line (full width)
│                     ─┬─                                 │    ORP letter highlighted
│                                                         │
│ ░░░░░░░░░░ faded context line below ░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░░ faded context line below ░░░░░░░░░░░░░░░░░░░ │
├─────────────────────────────────────────────────────────┤
│ ▸ Introduction                                     12%  │
│ ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  350 WPM  ⏸/▶  │
└─────────────────────────────────────────────────────────┘
```

### Outline Mode

```
┌─────────────────────────────────────────────────────────┐
│  OUTLINE                                                │
│                                                         │
│  # Introduction                                         │  ← H1
│  ## Background                                          │  ← H2
│  ## Methodology                                         │
│  ### Data Collection                                    │  ← H3
│  ## Results                                             │
│  # Conclusion                                           │
│                                                         │
│  [Enter] Jump  [Esc] Back  [j/k] Navigate              │
└─────────────────────────────────────────────────────────┘
```

Heading levels shown with markdown symbols (`#`, `##`, etc.).

### Fading

Context lines use diminishing intensity:
- Adjacent lines: 70% brightness
- 2 lines away: 40% brightness
- 3+ lines away: 20% brightness

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause/resume |
| `j` / `↓` | Decrease WPM (slower, -25) |
| `k` / `↑` | Increase WPM (faster, +25) |
| `h` / `←` | Rewind one sentence |
| `l` / `→` | Skip forward one sentence |
| `H` | Rewind one paragraph |
| `L` | Skip forward one paragraph |
| `o` | Toggle outline view |
| `Enter` | (in outline) Jump to heading |
| `Esc` | Exit outline / quit confirmation |
| `q` | Quit |
| `?` | Show help overlay |

**WPM range**: 100–800 WPM, steps of 25.

## Timing Logic

**Base timing**: `60_000ms / WPM` per word

**Modifiers** (additive):

| Condition | Extra time |
|-----------|------------|
| Word length > 6 chars | +20ms per extra char |
| Word length > 10 chars | +40ms per extra char (stacks) |
| Comma, colon, semicolon | +150ms |
| Period, question, exclamation | +200ms |
| Paragraph break | +300ms |
| New block element (list, quote) | +150ms |
| Start of new heading section | +400ms |

*Values to be validated against existing implementations during build.*

## ORP Calculation

Optimal Recognition Point by word length:

| Word length | ORP position (0-indexed) |
|-------------|--------------------------|
| 1-3 chars | 0 (first letter) |
| 4-6 chars | 1 (second letter) |
| 7-9 chars | 2 (third letter) |
| 10+ chars | 3 (fourth letter) |

ORP letter rendered in accent color (red/orange). Word aligned so ORP stays at screen center.

## Markdown Handling

### Preserved (inline styling)
- **Bold** → bold terminal attribute
- *Italic* → italic terminal attribute
- `code` → distinct color/background
- [links](url) → underlined, URL stored

### Block prefixes
- List items: `•` (unordered) or `1.` (ordered)
- Quotes: `│` (dimmed)
- Callouts: `ℹ` / `⚠` / etc. based on type

### Skipped
- Code blocks (fenced + indented)
- Images
- HTML blocks
- Horizontal rules (timing break only)

## Testing Strategy

### Unit Tests
- **Parser**: Markdown → Token stream, style preservation, block detection, skip logic
- **ORP**: Position calculation for various word lengths
- **Timing**: Base timing, modifier stacking, structure breaks

### Integration Tests
- Full document → token stream → timing sequence
- Snapshot entire sequences with insta

### TUI Tests
- Ratatui TestBackend for render verification
- Snapshot terminal output at various states

### Test Files
- `tests/fixtures/*.md` - sample documents
- `tests/snapshots/*.snap` - insta snapshots

## Scope (v1)

**In scope:**
- CLI argument file loading
- ORP-centered RSVP display
- Markdown inline styling
- Block element handling (lists, quotes, callouts)
- Vim-style controls
- Heading outline navigation
- Progress bar

**Out of scope (future):**
- Config file persistence
- Bookmarks / position saving
- File picker TUI
- Multiple file support
- Custom themes
