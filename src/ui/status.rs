use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Section title + progress %
            Constraint::Length(1), // Progress bar + WPM + state
        ])
        .split(area);

    // Top line: section title and percentage
    let section_title = app.current_section_title().unwrap_or("Document");
    let progress_pct = (app.progress() * 100.0).round() as u16;
    let top_line = Line::from(vec![
        Span::raw("> "),
        Span::styled(section_title, Style::default().fg(Color::Cyan)),
        Span::raw(format!(" {progress_pct:>3}%")),
    ]);
    frame.render_widget(Paragraph::new(top_line), chunks[0]);

    // Bottom line: progress bar, WPM, pause state
    let pause_indicator = if app.is_paused() { "||" } else { ">" };
    let label = format!("  {} WPM  {}", app.wpm(), pause_indicator);

    let gauge = Gauge::default()
        .ratio(app.progress())
        .label(label)
        .gauge_style(Style::default().fg(Color::Rgb(80, 120, 80)).bg(Color::Rgb(40, 40, 40)));

    frame.render_widget(gauge, chunks[1]);
}
