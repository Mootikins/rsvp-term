use crate::app::App;
use crate::types::{BlockContext, TimedToken};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Left padding for context lines
const LEFT_PADDING: usize = 4;

/// Render context above the RSVP word
pub fn render_before(frame: &mut Frame, app: &App, area: Rect) {
    let lines = compute_document_lines(app, area.width as usize, app.context_width());
    let current_pos = app.position();

    // Find which line contains the current word
    let (line_idx, _) = find_position_in_lines(&lines, current_pos);

    // Render lines up to and including current line (words before current_pos shown, rest blanked)
    render_lines_before(frame, &lines, line_idx, current_pos, area);
}

/// Render context below the RSVP word
pub fn render_after(frame: &mut Frame, app: &App, area: Rect) {
    let lines = compute_document_lines(app, area.width as usize, app.context_width());
    let current_pos = app.position();

    // Find which line contains the current word
    let (line_idx, _) = find_position_in_lines(&lines, current_pos);

    // Render lines from current line onward (words after current_pos shown, rest blanked)
    render_lines_after(frame, &lines, line_idx, current_pos, area);
}

/// A line with its tokens and their global indices
struct DocLine<'a> {
    tokens: Vec<(usize, &'a TimedToken)>, // (global_index, token)
    is_blank: bool,                       // True for separator lines between blocks
}

/// Compute document lines from tokens around current position
fn compute_document_lines(app: &App, width: usize, max_line_chars: usize) -> Vec<DocLine<'_>> {
    let tokens = app.tokens();
    let pos = app.position();

    // Always start from 0 to ensure consistent line breaks (no reflow)
    let start = 0;
    let end = (pos + 500).min(tokens.len());

    // Use configurable max chars to prevent reflow on wide terminals
    let max_chars = width.saturating_sub(LEFT_PADDING + 4).min(max_line_chars);

    let mut lines: Vec<DocLine> = Vec::new();
    let mut current_line: Vec<(usize, &TimedToken)> = Vec::new();
    let mut current_width = 0;
    let mut last_block: Option<&BlockContext> = None;
    let mut last_table_row: Option<usize> = None;

    for (idx, token) in tokens.iter().enumerate().skip(start).take(end - start) {
        let current_table_row = table_row(&token.token.block);
        let is_table_cell = current_table_row.is_some();
        let was_in_table = last_table_row.is_some();

        // Detect block transitions
        let block_changed = last_block.is_some_and(|b| {
            if is_table_cell && was_in_table {
                current_table_row != last_table_row
            } else {
                b != &token.token.block
            }
        });

        let table_transition = was_in_table != is_table_cell;
        let word_width = token.token.word.chars().count() + 1;
        let would_overflow = current_width + word_width > max_chars;

        if (block_changed || table_transition || would_overflow) && !current_line.is_empty() {
            lines.push(DocLine {
                tokens: current_line,
                is_blank: false,
            });
            current_line = Vec::new();
            current_width = 0;

            // Insert blank line on block transitions (not just overflow)
            if block_changed || table_transition {
                lines.push(DocLine {
                    tokens: Vec::new(),
                    is_blank: true,
                });
            }
        }

        current_line.push((idx, token));
        current_width += word_width;
        last_block = Some(&token.token.block);
        last_table_row = current_table_row;
    }

    if !current_line.is_empty() {
        lines.push(DocLine {
            tokens: current_line,
            is_blank: false,
        });
    }

    lines
}

/// Find which line and word index contains the given global position
fn find_position_in_lines(lines: &[DocLine], pos: usize) -> (usize, usize) {
    for (line_idx, line) in lines.iter().enumerate() {
        for (word_idx, (global_idx, _)) in line.tokens.iter().enumerate() {
            if *global_idx == pos {
                return (line_idx, word_idx);
            }
        }
    }
    (0, 0)
}

/// Extract row number from a table cell block context
const fn table_row(block: &BlockContext) -> Option<usize> {
    match block {
        BlockContext::TableCell(row) => Some(*row),
        _ => None,
    }
}

/// Get block prefix for visual indication
const fn block_prefix(block: &BlockContext) -> &'static str {
    match block {
        BlockContext::ListItem(_) => "* ",
        BlockContext::Quote(_) | BlockContext::TableCell(_) => "| ",
        BlockContext::Heading(_) | BlockContext::Paragraph => "",
        BlockContext::Callout(_) => "[i] ",
    }
}

/// Render lines before the current line (above context)
fn render_lines_before(
    frame: &mut Frame,
    lines: &[DocLine],
    current_line_idx: usize,
    current_pos: usize,
    area: Rect,
) {
    if area.height == 0 {
        return;
    }

    let num_lines = area.height as usize;
    // Include current line (it will show words before current_pos)
    let end_line = (current_line_idx + 1).min(lines.len());
    let start_line = end_line.saturating_sub(num_lines);
    let lines_to_show: Vec<_> = lines[start_line..end_line].iter().collect();

    // Render from top to bottom, with fading (farther = dimmer)
    for (i, line) in lines_to_show.iter().enumerate() {
        let distance_from_bottom = lines_to_show.len() - 1 - i;
        let y = area.y + (area.height as usize - lines_to_show.len() + i) as u16;

        if y < area.y || y >= area.y + area.height {
            continue;
        }

        render_line(
            frame,
            line,
            y,
            area.width,
            distance_from_bottom,
            current_pos,
            ContextType::Before,
        );
    }
}

/// Render lines after the current word (below context)
fn render_lines_after(
    frame: &mut Frame,
    lines: &[DocLine],
    current_line_idx: usize,
    current_pos: usize,
    area: Rect,
) {
    if area.height == 0 || current_line_idx >= lines.len() {
        return;
    }

    let num_lines = area.height as usize;
    let end_line = (current_line_idx + num_lines).min(lines.len());
    let lines_to_show = &lines[current_line_idx..end_line];

    // Render lines, with fading (farther = dimmer)
    for (i, line) in lines_to_show.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        render_line(
            frame,
            line,
            y,
            area.width,
            i,
            current_pos,
            ContextType::After,
        );
    }
}

/// Mode for rendering words - either show text or blank spaces
#[derive(Clone, Copy, PartialEq)]
enum WordMode {
    Visible,
    Blank,
}

/// Render a single line at the given y position
/// Words are shown or blanked based on their position relative to current_pos
fn render_line(
    frame: &mut Frame,
    line: &DocLine,
    y: u16,
    width: u16,
    distance: usize,
    current_pos: usize,
    context_type: ContextType,
) {
    // Blank separator lines - just skip (renders as empty space)
    if line.is_blank || line.tokens.is_empty() {
        return;
    }

    let gray = match distance {
        0 => Color::Rgb(200, 200, 200),
        1 => Color::Rgb(150, 150, 150),
        2 => Color::Rgb(110, 110, 110),
        3 => Color::Rgb(80, 80, 80),
        _ => Color::Rgb(60, 60, 60),
    };
    let style = Style::default().fg(gray);

    let first_token = &line.tokens[0].1;
    let prefix = block_prefix(&first_token.token.block);

    let padding = " ".repeat(LEFT_PADDING);
    let mut spans = vec![Span::raw(padding), Span::styled(prefix, style)];

    // Add words - visible or blank depending on position
    let mut prev_table_row: Option<usize> = None;
    for (j, (global_idx, token)) in line.tokens.iter().enumerate() {
        let current_row = table_row(&token.token.block);
        let is_new_cell = current_row.is_some() && token.token.timing_hint.structure_modifier > 0;

        // Add cell separator between cells in same row
        if is_new_cell && prev_table_row.is_some() && j > 0 {
            spans.push(Span::styled(" | ", style));
        }

        // Determine if this word should be visible or blank
        let mode = match context_type {
            ContextType::Before => {
                // In "before" context: show words before current_pos, blank others
                if *global_idx < current_pos {
                    WordMode::Visible
                } else {
                    WordMode::Blank
                }
            }
            ContextType::After => {
                // In "after" context: show words after current_pos, blank others
                if *global_idx > current_pos {
                    WordMode::Visible
                } else {
                    WordMode::Blank
                }
            }
        };

        let word_text = format!("{} ", token.token.word);
        let display_text = match mode {
            WordMode::Visible => word_text,
            WordMode::Blank => " ".repeat(token.token.word.chars().count() + 1),
        };

        spans.push(Span::styled(display_text, style));
        prev_table_row = current_row;
    }

    // Add trailing | for table rows
    if prev_table_row.is_some() {
        spans.push(Span::styled("|", style));
    }

    let line_area = Rect {
        x: 0,
        y,
        width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
}

#[derive(Clone, Copy)]
enum ContextType {
    Before,
    After,
}
