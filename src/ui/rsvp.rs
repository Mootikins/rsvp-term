use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Guide line color - slightly lighter than context text
const GUIDE_COLOR: Color = Color::Rgb(120, 120, 120);

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let Some(token) = app.current_token() else {
        return;
    };

    let word = &token.token.word;
    let orp_pos = token.orp_position;

    // Calculate ORP center position
    let center = area.width as usize / 2;
    let left_padding = center.saturating_sub(orp_pos);

    // Build guide line with tick mark at ORP position
    let guide_style = Style::default().fg(GUIDE_COLOR);

    // Top guide: ───┬───
    let top_line = build_guide_line(area.width as usize, left_padding + orp_pos, '┬');
    let top_para = Paragraph::new(Line::from(Span::styled(top_line, guide_style)));

    // Bottom guide: ───┴───
    let bottom_line = build_guide_line(area.width as usize, left_padding + orp_pos, '┴');
    let bottom_para = Paragraph::new(Line::from(Span::styled(bottom_line, guide_style)));

    // Build styled word with ORP highlight
    let chars: Vec<char> = word.chars().collect();
    let mut spans = Vec::with_capacity(chars.len() + 1);

    spans.push(Span::raw(" ".repeat(left_padding)));

    for (i, c) in chars.iter().enumerate() {
        let style = if i == orp_pos {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    let word_para = Paragraph::new(Line::from(spans));

    // Center vertically - need 3 lines: guide, word, guide
    let vertical_center = area.height / 2;

    // Top guide line
    if vertical_center > 0 {
        let top_area = Rect {
            x: area.x,
            y: area.y + vertical_center - 1,
            width: area.width,
            height: 1,
        };
        frame.render_widget(top_para, top_area);
    }

    // Word line
    let word_area = Rect {
        x: area.x,
        y: area.y + vertical_center,
        width: area.width,
        height: 1,
    };
    frame.render_widget(word_para, word_area);

    // Bottom guide line
    if vertical_center + 1 < area.height {
        let bottom_area = Rect {
            x: area.x,
            y: area.y + vertical_center + 1,
            width: area.width,
            height: 1,
        };
        frame.render_widget(bottom_para, bottom_area);
    }
}

/// Build a guide line with a tick mark at the specified position
fn build_guide_line(width: usize, tick_pos: usize, tick_char: char) -> String {
    let mut line = String::with_capacity(width);
    for i in 0..width {
        if i == tick_pos {
            line.push(tick_char);
        } else {
            line.push('─');
        }
    }
    line
}
