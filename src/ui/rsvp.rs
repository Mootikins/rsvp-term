use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let token = match app.current_token() {
        Some(t) => t,
        None => {
            let block = Block::default().borders(Borders::ALL);
            frame.render_widget(block, area);
            return;
        }
    };

    let word = &token.token.word;
    let orp_pos = token.orp_position;

    // Build styled word with ORP highlight
    let chars: Vec<char> = word.chars().collect();
    let mut spans = Vec::new();

    for (i, c) in chars.iter().enumerate() {
        let style = if i == orp_pos {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    // Calculate padding for ORP centering
    let center = area.width as usize / 2;
    let left_padding = center.saturating_sub(orp_pos);
    let padding = " ".repeat(left_padding);

    let mut line_spans = vec![Span::raw(padding)];
    line_spans.extend(spans);

    let paragraph = Paragraph::new(Line::from(line_spans))
        .block(Block::default())
        .alignment(Alignment::Left);

    // Center vertically
    let vertical_center = area.height / 2;
    let rsvp_area = Rect {
        x: area.x,
        y: area.y + vertical_center,
        width: area.width,
        height: 1,
    };

    frame.render_widget(paragraph, rsvp_area);
}
