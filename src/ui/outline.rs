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

/// Guide line color
const GUIDE_COLOR: Color = Color::Rgb(120, 120, 120);

/// Fade zone: dotted (2) + dashed (2) + solid fade (2) on each side
const FADE_DOTTED: usize = 2;
const FADE_DASHED: usize = 2;
const FADE_SOLID: usize = 2;
const FADE_TOTAL: usize = FADE_DOTTED + FADE_DASHED + FADE_SOLID;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let sections = app.sections();
    if sections.is_empty() {
        return;
    }

    let selected = app.outline_selection();
    // Need 3 lines for selected (top bar, text, bottom bar)
    let content_height = area.height as usize;
    if content_height < 3 {
        return;
    }

    // Calculate center position for the selected item (with its guide bars)
    let center_y = area.height / 2;

    // Get selected section info for guide bars
    let selected_section = &sections[selected];
    let hint = "#".repeat(selected_section.level as usize);
    let title_width = selected_section.title.chars().count();
    let title_padding = calculate_padding(title_width, area.width as usize);
    let tick_pos = title_padding + title_width / 2;

    // Render top guide bar
    if center_y > 0 {
        let top_y = area.y + center_y - 1;
        let top_spans = build_faded_guide_line(area.width as usize, tick_pos, '┬', &hint);
        let top_para = Paragraph::new(Line::from(top_spans));
        frame.render_widget(
            top_para,
            Rect {
                x: area.x,
                y: top_y,
                width: area.width,
                height: 1,
            },
        );
    }

    // Render selected item
    {
        let style = Style::default()
            .fg(Color::Rgb(200, 200, 200))
            .add_modifier(Modifier::BOLD);
        let text = format!(
            "{}{}",
            " ".repeat(title_padding),
            selected_section.title
        );
        let para = Paragraph::new(Line::from(Span::styled(text, style)));
        frame.render_widget(
            para,
            Rect {
                x: area.x,
                y: area.y + center_y,
                width: area.width,
                height: 1,
            },
        );
    }

    // Render bottom guide bar
    if center_y + 1 < area.height {
        let bottom_y = area.y + center_y + 1;
        let bottom_spans = build_faded_guide_line(area.width as usize, tick_pos, '┴', &hint);
        let bottom_para = Paragraph::new(Line::from(bottom_spans));
        frame.render_widget(
            bottom_para,
            Rect {
                x: area.x,
                y: bottom_y,
                width: area.width,
                height: 1,
            },
        );
    }

    // Render items above selected (from center-2 upward)
    let mut above_y = center_y.saturating_sub(2);
    let mut above_idx = selected.saturating_sub(1);
    let mut distance = 1usize;
    while above_idx < sections.len() && above_y < area.height {
        let section = &sections[above_idx];
        render_item(frame, section, area.x, area.y + above_y, area.width, distance);

        if above_idx == 0 || above_y == 0 {
            break;
        }
        above_idx -= 1;
        above_y -= 1;
        distance += 1;
    }

    // Render items below selected (from center+2 downward)
    let mut below_y = center_y + 2;
    let mut below_idx = selected + 1;
    let mut distance = 1usize;
    while below_idx < sections.len() && below_y < area.height {
        let section = &sections[below_idx];
        render_item(frame, section, area.x, area.y + below_y, area.width, distance);

        below_idx += 1;
        below_y += 1;
        distance += 1;
    }
}

fn render_item(frame: &mut Frame, section: &crate::types::Section, x: u16, y: u16, width: u16, distance: usize) {
    let gray = match distance {
        1 => Color::Rgb(150, 150, 150),
        2 => Color::Rgb(110, 110, 110),
        3 => Color::Rgb(80, 80, 80),
        _ => Color::Rgb(60, 60, 60),
    };
    let style = Style::default().fg(gray);

    let content_width = section.title.chars().count();
    let padding = calculate_padding(content_width, width as usize);
    let text = format!("{}{}", " ".repeat(padding), section.title);

    let para = Paragraph::new(Line::from(Span::styled(text, style)));
    frame.render_widget(
        para,
        Rect {
            x,
            y,
            width,
            height: 1,
        },
    );
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

/// Build a guide line with fade effect on both sides
/// Pattern: dotted (┄) → dashed (╌) → solid (─) with increasing brightness
fn build_faded_guide_line<'a>(
    width: usize,
    tick_pos: usize,
    tick_char: char,
    hint: &str,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let hint_len = hint.len();

    // Add hint at start (right-aligned in first few chars)
    let hint_style = Style::default().fg(GUIDE_COLOR);
    if hint_len > 0 && hint_len < width {
        spans.push(Span::styled(format!("{:>4} ", hint), hint_style));
    }

    let start_col = if hint_len > 0 { 5 } else { 0 };
    let fade_end_left = (start_col + FADE_TOTAL).min(width);
    let fade_start_right = width.saturating_sub(FADE_TOTAL);

    for i in start_col..width {
        let (c, brightness) = if i < fade_end_left {
            // Left fade zone
            let progress = i - start_col;
            if progress < FADE_DOTTED {
                let b = 40 + (progress * 20 / FADE_DOTTED.max(1)) as u8;
                ('┄', b)
            } else if progress < FADE_DOTTED + FADE_DASHED {
                let p = progress - FADE_DOTTED;
                let b = 60 + (p * 20 / FADE_DASHED.max(1)) as u8;
                ('╌', b)
            } else {
                let p = progress - FADE_DOTTED - FADE_DASHED;
                let b = 80 + (p * 40 / FADE_SOLID.max(1)) as u8;
                ('─', b)
            }
        } else if i >= fade_start_right {
            // Right fade zone (mirror of left)
            let progress = width - 1 - i;
            if progress < FADE_DOTTED {
                let b = 40 + (progress * 20 / FADE_DOTTED.max(1)) as u8;
                ('┄', b)
            } else if progress < FADE_DOTTED + FADE_DASHED {
                let p = progress - FADE_DOTTED;
                let b = 60 + (p * 20 / FADE_DASHED.max(1)) as u8;
                ('╌', b)
            } else {
                let p = progress - FADE_DOTTED - FADE_DASHED;
                let b = 80 + (p * 40 / FADE_SOLID.max(1)) as u8;
                ('─', b)
            }
        } else {
            // Full brightness solid middle
            ('─', 120)
        };

        let display_char = if i == tick_pos { tick_char } else { c };
        let style = Style::default().fg(Color::Rgb(brightness, brightness, brightness));
        spans.push(Span::styled(display_char.to_string(), style));
    }

    spans
}
