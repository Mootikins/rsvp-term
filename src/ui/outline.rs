use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Minimum left padding for outline items
const MIN_PADDING: usize = 2;

/// Threshold for centering: if content uses less than this fraction of width, center it
const CENTER_THRESHOLD: f32 = 0.6;

/// Hint color - matches context gutter
const HINT_COLOR: Color = Color::Rgb(120, 120, 120);

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let sections = app.sections();
    if sections.is_empty() {
        return;
    }

    let selected = app.outline_selection();
    // Reserve bottom line for hint
    let content_height = area.height.saturating_sub(1) as usize;
    if content_height == 0 {
        return;
    }

    // Calculate which items to show (selected centered)
    let half = content_height / 2;
    let start = selected.saturating_sub(half);
    let end = (start + content_height).min(sections.len());
    let start = end.saturating_sub(content_height);

    // Render items with scroll (selected stays centered)
    for (i, idx) in (start..end).enumerate() {
        let section = &sections[idx];
        let distance = selected.abs_diff(idx);

        let y = area.y + i as u16;
        if y >= area.y + area.height.saturating_sub(1) {
            break;
        }

        // Color based on distance (matches context view)
        let gray = match distance {
            0 => Color::Rgb(200, 200, 200),
            1 => Color::Rgb(150, 150, 150),
            2 => Color::Rgb(110, 110, 110),
            3 => Color::Rgb(80, 80, 80),
            _ => Color::Rgb(60, 60, 60),
        };

        let mut style = Style::default().fg(gray);
        if idx == selected {
            style = style.add_modifier(Modifier::BOLD);
        }

        // Calculate centering (same logic as context view)
        let content_width = section.title.chars().count();
        let padding = calculate_padding(content_width, area.width as usize);

        let text = format!("{}{}", " ".repeat(padding), section.title);
        let para = Paragraph::new(Line::from(Span::styled(text, style)));

        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(para, line_area);
    }

    // Render hint at bottom (centered)
    if selected < sections.len() {
        let section = &sections[selected];
        let hint = "#".repeat(section.level as usize);
        let hint_style = Style::default().fg(HINT_COLOR);

        let hint_y = area.y + area.height.saturating_sub(1);
        let hint_padding = (area.width as usize).saturating_sub(hint.len()) / 2;
        let hint_text = format!("{}{}", " ".repeat(hint_padding), hint);

        let hint_para = Paragraph::new(Line::from(Span::styled(hint_text, hint_style)));
        let hint_area = Rect {
            x: area.x,
            y: hint_y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(hint_para, hint_area);
    }
}

/// Calculate left padding for a line - centers short lines, left-aligns long ones
fn calculate_padding(content_width: usize, available_width: usize) -> usize {
    if available_width == 0 {
        return MIN_PADDING;
    }
    let ratio = content_width as f32 / available_width as f32;

    if ratio < CENTER_THRESHOLD {
        (available_width.saturating_sub(content_width)) / 2
    } else {
        MIN_PADDING
    }
    .max(MIN_PADDING)
}
