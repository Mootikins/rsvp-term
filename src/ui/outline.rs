use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .sections()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let prefix = "#".repeat(section.level as usize);
            let text = format!("{} {}", prefix, section.title);

            let style = if i == app.outline_selection() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items).block(Block::default().title(" OUTLINE ").borders(Borders::ALL));

    frame.render_widget(list, area);
}
