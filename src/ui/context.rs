use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::App;
use crate::types::{BlockContext, TimedToken};

/// Left padding for context lines
const LEFT_PADDING: usize = 4;

pub fn render_before(frame: &mut Frame, app: &App, area: Rect) {
    let (before, _) = app.context_tokens(100, 0);
    render_context_lines(frame, before, area, true);
}

pub fn render_after(frame: &mut Frame, app: &App, area: Rect) {
    let (_, after) = app.context_tokens(0, 100);
    render_context_lines(frame, after, area, false);
}

/// Group tokens into logical lines based on block context and width
fn group_into_lines<'a>(tokens: &'a [TimedToken], max_width: usize) -> Vec<Vec<&'a TimedToken>> {
    let mut lines: Vec<Vec<&TimedToken>> = Vec::new();
    let mut current_line: Vec<&TimedToken> = Vec::new();
    let mut current_width = 0;
    let mut last_block: Option<&BlockContext> = None;

    for token in tokens {
        // Start new line on block change (paragraph boundary)
        let block_changed = last_block.map_or(false, |b| b != &token.token.block);

        // Also wrap if line is too long
        let word_width = token.token.word.chars().count() + 1; // +1 for space
        let would_overflow = current_width + word_width > max_width;

        if (block_changed || would_overflow) && !current_line.is_empty() {
            lines.push(current_line);
            current_line = Vec::new();
            current_width = 0;
        }

        current_line.push(token);
        current_width += word_width;
        last_block = Some(&token.token.block);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Get block prefix for visual indication
fn block_prefix(block: &BlockContext) -> &'static str {
    match block {
        BlockContext::ListItem(_) => "• ",
        BlockContext::Quote(_) => "│ ",
        BlockContext::Heading(_) => "",
        BlockContext::Callout(_) => "ℹ ",
        BlockContext::Paragraph => "",
    }
}

fn render_context_lines(
    frame: &mut Frame,
    tokens: &[TimedToken],
    area: Rect,
    fade_up: bool,
) {
    if tokens.is_empty() || area.height == 0 || area.width < 10 {
        return;
    }

    // Group tokens respecting paragraph boundaries
    let max_chars_per_line = (area.width as usize).saturating_sub(4); // margin
    let lines = group_into_lines(tokens, max_chars_per_line);

    let num_lines = area.height as usize;
    let display_lines: Vec<_> = if fade_up {
        // For "before" context: show most recent lines at bottom
        lines.into_iter().rev().take(num_lines).collect::<Vec<_>>()
    } else {
        // For "after" context: show next lines at top
        lines.into_iter().take(num_lines).collect()
    };

    // Reverse again for fade_up so index 0 = closest to RSVP word
    let display_lines: Vec<_> = if fade_up {
        display_lines.into_iter().rev().collect()
    } else {
        display_lines
    };

    for (i, line_tokens) in display_lines.iter().enumerate() {
        if line_tokens.is_empty() {
            continue;
        }

        // Distance from RSVP word determines brightness
        let distance = if fade_up {
            display_lines.len() - 1 - i // bottom = closest = brightest
        } else {
            i // top = closest = brightest
        };

        let gray = match distance {
            0 => Color::Rgb(200, 200, 200),
            1 => Color::Rgb(150, 150, 150),
            2 => Color::Rgb(110, 110, 110),
            3 => Color::Rgb(80, 80, 80),
            _ => Color::Rgb(60, 60, 60),
        };

        let style = Style::default().fg(gray);

        // Left padding + block prefix
        let prefix = block_prefix(&line_tokens[0].token.block);
        let padding = " ".repeat(LEFT_PADDING);
        let mut spans = vec![
            Span::raw(padding),
            Span::styled(prefix, style),
        ];

        // Add words
        for token in line_tokens.iter() {
            spans.push(Span::styled(format!("{} ", token.token.word), style));
        }

        let y = if fade_up {
            area.y + i as u16
        } else {
            area.y + i as u16
        };

        if y >= area.y + area.height {
            continue;
        }

        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };

        frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
    }
}
