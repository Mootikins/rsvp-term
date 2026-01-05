use crate::app::App;
use crate::types::{BlockContext, TokenStyle};
use crate::ui::GUTTER_WIDTH;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Guide line color - slightly lighter than context text
const GUIDE_COLOR: Color = Color::Rgb(120, 120, 120);


/// Fade zone characters: dotted (2) + dashed (2) + solid fade (2)
const FADE_DOTTED: usize = 2;
const FADE_DASHED: usize = 2;
const FADE_SOLID: usize = 2;
const FADE_TOTAL: usize = FADE_DOTTED + FADE_DASHED + FADE_SOLID;

/// Get hint characters for a block context
fn current_block_hint(block: &BlockContext) -> &'static str {
    match block {
        BlockContext::Heading(1) => "#",
        BlockContext::Heading(2) => "##",
        BlockContext::Heading(3) => "###",
        BlockContext::Heading(4) => "####",
        BlockContext::Heading(5) => "#####",
        BlockContext::Heading(6) => "######",
        BlockContext::Heading(_) => "#",
        BlockContext::ListItem(_) => "-",
        BlockContext::Quote(_) => ">",
        BlockContext::TableCell(_) => "|",
        BlockContext::Callout(_) => "[!]",
        BlockContext::Paragraph => "",
    }
}

pub fn render(frame: &mut Frame, app: &App, area: Rect, gutter_area: Option<Rect>) {
    let Some(token) = app.current_token() else {
        return;
    };

    let word = &token.token.word;
    let orp_pos = token.orp_position;

    // Calculate base style from token style (if styling enabled)
    let base_style = if app.styling_enabled {
        match &token.token.style {
            TokenStyle::Bold => Style::default().add_modifier(Modifier::BOLD),
            TokenStyle::Italic => Style::default().add_modifier(Modifier::ITALIC),
            TokenStyle::BoldItalic => Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
            TokenStyle::Code => Style::default().bg(Color::Rgb(60, 60, 60)),
            TokenStyle::Link(_) => Style::default().add_modifier(Modifier::UNDERLINED),
            TokenStyle::Normal => Style::default(),
        }
    } else {
        Style::default()
    };

    // Calculate ORP center position
    let center = area.width as usize / 2;
    let left_padding = center.saturating_sub(orp_pos);

    // Build guide line with tick mark at ORP position
    let guide_style = Style::default().fg(GUIDE_COLOR);

    // Build guide lines - with fade effect if hint_chars enabled
    let (top_line, bottom_line) = if app.hint_chars_enabled {
        // Build faded guide lines
        let top_spans = build_faded_guide_line(area.width as usize, left_padding + orp_pos, '┬');
        let bottom_spans =
            build_faded_guide_line(area.width as usize, left_padding + orp_pos, '┴');
        (top_spans, bottom_spans)
    } else {
        // Simple guide lines
        let top = build_guide_line(area.width as usize, left_padding + orp_pos, '┬');
        let bottom = build_guide_line(area.width as usize, left_padding + orp_pos, '┴');
        (
            vec![Span::styled(top, guide_style)],
            vec![Span::styled(bottom, guide_style)],
        )
    };
    let top_para = Paragraph::new(Line::from(top_line));
    let bottom_para = Paragraph::new(Line::from(bottom_line));

    // Build styled word with ORP highlight
    let chars: Vec<char> = word.chars().collect();
    let mut spans = Vec::with_capacity(chars.len() + 1);

    spans.push(Span::raw(" ".repeat(left_padding)));

    for (i, c) in chars.iter().enumerate() {
        let char_style = if i == orp_pos {
            base_style.fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            base_style.fg(Color::White)
        };
        spans.push(Span::styled(c.to_string(), char_style));
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

    // Render gutter hints if enabled
    if let Some(gutter) = gutter_area {
        let gutter_style = Style::default().fg(GUIDE_COLOR);

        // Current block hint at word line
        let block_hint = current_block_hint(&token.token.block);
        if !block_hint.is_empty() {
            let hint_text = format!("{:>width$}", block_hint, width = GUTTER_WIDTH as usize);
            let hint_para = Paragraph::new(Line::from(Span::styled(hint_text, gutter_style)));
            let hint_area = Rect {
                x: gutter.x,
                y: gutter.y + vertical_center,
                width: GUTTER_WIDTH,
                height: 1,
            };
            frame.render_widget(hint_para, hint_area);
        }

        // Parent context hint at guide lines
        if let Some(parent) = &token.token.parent_context {
            let parent_hint = parent.hint_chars();
            if !parent_hint.is_empty() {
                let hint_text = format!("{:>width$}", parent_hint, width = GUTTER_WIDTH as usize);

                // Top guide line gutter
                if vertical_center > 0 {
                    let top_hint_para =
                        Paragraph::new(Line::from(Span::styled(&hint_text, gutter_style)));
                    let top_hint_area = Rect {
                        x: gutter.x,
                        y: gutter.y + vertical_center - 1,
                        width: GUTTER_WIDTH,
                        height: 1,
                    };
                    frame.render_widget(top_hint_para, top_hint_area);
                }

                // Bottom guide line gutter
                if vertical_center + 1 < area.height {
                    let bottom_hint_para =
                        Paragraph::new(Line::from(Span::styled(&hint_text, gutter_style)));
                    let bottom_hint_area = Rect {
                        x: gutter.x,
                        y: gutter.y + vertical_center + 1,
                        width: GUTTER_WIDTH,
                        height: 1,
                    };
                    frame.render_widget(bottom_hint_para, bottom_hint_area);
                }
            }
        }
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

/// Build a guide line with fade effect on the left side
/// Pattern: dotted (┄) → dashed (╌) → solid (─) with increasing brightness
fn build_faded_guide_line<'a>(width: usize, tick_pos: usize, tick_char: char) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let fade_end = FADE_TOTAL.min(width);

    for i in 0..width {
        let (c, brightness) = if i < FADE_DOTTED.min(fade_end) {
            // Dotted zone (dimmest): ┄
            let b = 40 + (i * 20 / FADE_DOTTED.max(1)) as u8;
            ('┄', b)
        } else if i < (FADE_DOTTED + FADE_DASHED).min(fade_end) {
            // Dashed zone (medium): ╌
            let progress = i - FADE_DOTTED;
            let b = 60 + (progress * 20 / FADE_DASHED.max(1)) as u8;
            ('╌', b)
        } else if i < fade_end {
            // Solid fade zone (brightening): ─
            let progress = i - FADE_DOTTED - FADE_DASHED;
            let b = 80 + (progress * 40 / FADE_SOLID.max(1)) as u8;
            ('─', b)
        } else {
            // Full brightness solid
            ('─', 120)
        };

        // Use tick char at tick position
        let display_char = if i == tick_pos { tick_char } else { c };
        let style = Style::default().fg(Color::Rgb(brightness, brightness, brightness));
        spans.push(Span::styled(display_char.to_string(), style));
    }

    spans
}
