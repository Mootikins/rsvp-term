use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
};

pub fn render(frame: &mut Frame, area: Rect) {
    // Center the help box
    let width = 50.min(area.width.saturating_sub(4));
    let height = 16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let help_area = Rect { x, y, width, height };

    // Clear background
    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled("CONTROLS", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("Space     Pause/Resume"),
        Line::from("j/Down    Slower (-25 WPM)"),
        Line::from("k/Up      Faster (+25 WPM)"),
        Line::from("h/Left    Rewind sentence"),
        Line::from("l/Right   Skip sentence"),
        Line::from("o         Toggle outline"),
        Line::from("q         Quit"),
        Line::from("?         Toggle help"),
        Line::from(""),
        Line::from(Span::styled("Press ? to close", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, help_area);
}
