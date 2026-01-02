use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::App;
use crate::types::TimedToken;

pub fn render_before(frame: &mut Frame, app: &App, area: Rect) {
    let (before, _) = app.context_tokens(50, 0);
    render_context_lines(frame, before, area, true);
}

pub fn render_after(frame: &mut Frame, app: &App, area: Rect) {
    let (_, after) = app.context_tokens(0, 50);
    render_context_lines(frame, after, area, false);
}

fn render_context_lines(
    frame: &mut Frame,
    tokens: &[TimedToken],
    area: Rect,
    fade_up: bool,
) {
    if tokens.is_empty() || area.height == 0 {
        return;
    }

    // Group tokens into lines (rough: ~10 words per line)
    let words_per_line = (area.width as usize / 8).max(5);
    let lines: Vec<Vec<&TimedToken>> = tokens
        .chunks(words_per_line)
        .map(|chunk| chunk.iter().collect())
        .collect();

    let num_lines = area.height as usize;
    let display_lines: Vec<_> = if fade_up {
        lines.iter().rev().take(num_lines).collect()
    } else {
        lines.iter().take(num_lines).collect()
    };

    for (i, line_tokens) in display_lines.iter().enumerate() {
        // Calculate fade level (0 = brightest, further = dimmer)
        let distance = i;

        let gray = match distance {
            0 => Color::Rgb(180, 180, 180),
            1 => Color::Rgb(120, 120, 120),
            2 => Color::Rgb(80, 80, 80),
            _ => Color::Rgb(50, 50, 50),
        };

        let spans: Vec<Span> = line_tokens
            .iter()
            .map(|t| {
                let style = Style::default().fg(gray);
                Span::styled(format!("{} ", t.token.word), style)
            })
            .collect();

        let y = if fade_up {
            area.y + area.height - 1 - i as u16
        } else {
            area.y + i as u16
        };

        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };

        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            line_area,
        );
    }
}
