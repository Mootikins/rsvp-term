use crate::app::App;
use crate::types::{BlockContext, TimedToken, TokenStyle};
use crate::ui::common::{calculate_padding, GUIDE_COLOR, MIN_PADDING};
use crate::ui::GUTTER_WIDTH;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render context above the RSVP word
pub fn render_before(frame: &mut Frame, app: &App, area: Rect, gutter_area: Option<Rect>) {
    let lines = compute_document_lines(app, area.width as usize, app.context_width());
    let current_pos = app.position();

    // Find which line contains the current word
    let (line_idx, _) = find_position_in_lines(&lines, current_pos);

    // Render lines up to and including current line (words before current_pos shown, rest blanked)
    render_lines_before(
        frame,
        &lines,
        line_idx,
        current_pos,
        area,
        app.styling_enabled,
        gutter_area,
    );
}

/// Render context below the RSVP word
pub fn render_after(frame: &mut Frame, app: &App, area: Rect, gutter_area: Option<Rect>) {
    let lines = compute_document_lines(app, area.width as usize, app.context_width());
    let current_pos = app.position();

    // Find which line contains the current word
    let (line_idx, _) = find_position_in_lines(&lines, current_pos);

    // Render lines from current line onward (words after current_pos shown, rest blanked)
    render_lines_after(
        frame,
        &lines,
        line_idx,
        current_pos,
        area,
        app.styling_enabled,
        gutter_area,
    );
}

/// A line with its tokens and their global indices
#[derive(Clone)]
struct DocLine<'a> {
    tokens: Vec<(usize, &'a TimedToken)>, // (global_index, token)
    is_blank: bool,                       // True for separator lines between blocks
}

/// Compute document lines from tokens around current position
fn compute_document_lines(app: &App, width: usize, max_line_chars: usize) -> Vec<DocLine<'_>> {
    let tokens = app.tokens();
    let pos = app.position();

    // Look back enough tokens to fill context, but not from the beginning
    // This prevents O(n) growth as position increases
    let start = pos.saturating_sub(500);
    let end = (pos + 500).min(tokens.len());

    // Use configurable max chars to prevent reflow on wide terminals
    let max_chars = width.saturating_sub(MIN_PADDING + 4).min(max_line_chars);

    let mut lines: Vec<DocLine> = Vec::new();
    let mut current_line: Vec<(usize, &TimedToken)> = Vec::new();
    let mut current_width = 0;
    let mut last_block: Option<&BlockContext> = None;
    let mut last_table_row: Option<usize> = None;

    for (idx, token) in (start..end).zip(&tokens[start..end]) {
        let current_table_row = table_row(&token.token.block);
        let is_table_cell = current_table_row.is_some();
        let was_in_table = last_table_row.is_some();

        // Detect block transitions
        let is_new_list_item = matches!(&token.token.block, BlockContext::ListItem(_))
            && token.token.timing_hint.structure_modifier > 0;

        let block_changed = is_new_list_item
            || last_block.is_some_and(|b| {
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

/// Compute column widths for table cells in a set of lines
/// Returns a map from (row, column) to max width needed for that column
fn compute_table_column_widths(lines: &[DocLine]) -> std::collections::HashMap<usize, usize> {
    let mut column_widths: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for line in lines {
        if line.is_blank || line.tokens.is_empty() {
            continue;
        }

        // Track cell content by column
        let mut current_col: Option<usize> = None;
        let mut cell_width = 0usize;

        for (_, token) in &line.tokens {
            if let Some(col) = token.token.timing_hint.table_column {
                // Starting a new cell
                if token.token.timing_hint.is_cell_start {
                    // Save previous cell's width
                    if let Some(prev_col) = current_col {
                        let entry = column_widths.entry(prev_col).or_insert(0);
                        *entry = (*entry).max(cell_width);
                    }
                    current_col = Some(col);
                    cell_width = token.token.word.chars().count();
                } else {
                    // Continue current cell
                    cell_width += 1 + token.token.word.chars().count(); // space + word
                }
            }
        }

        // Don't forget the last cell
        if let Some(col) = current_col {
            let entry = column_widths.entry(col).or_insert(0);
            *entry = (*entry).max(cell_width);
        }
    }

    column_widths
}

/// Calculate the display width of a line's content (including prefix and separators)
fn calculate_line_width(line: &DocLine) -> usize {
    if line.is_blank || line.tokens.is_empty() {
        return 0;
    }

    let first_token = &line.tokens[0].1;
    let prefix_width = block_prefix(&first_token.token.block).chars().count();

    let mut width = prefix_width;
    let mut prev_table_row: Option<usize> = None;

    for (j, (_, token)) in line.tokens.iter().enumerate() {
        let current_row = table_row(&token.token.block);
        let is_new_cell = current_row.is_some() && token.token.timing_hint.is_cell_start;

        // Cell separator
        if is_new_cell && prev_table_row.is_some() && j > 0 {
            width += 3; // " | "
        }

        width += token.token.word.chars().count() + 1; // word + space
        prev_table_row = current_row;
    }

    // Trailing | for table rows
    if prev_table_row.is_some() {
        width += 1;
    }

    width
}

/// Get block prefix for visual indication
fn block_prefix(block: &BlockContext) -> &'static str {
    match block {
        // Only show list marker for nested lists (depth > 1)
        BlockContext::ListItem(depth) if *depth > 1 => "- ",
        BlockContext::ListItem(_) => "",
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
    styling_enabled: bool,
    gutter_area: Option<Rect>,
) {
    if area.height == 0 {
        return;
    }

    let num_lines = area.height as usize;
    // Include current line (it will show words before current_pos)
    let end_line = (current_line_idx + 1).min(lines.len());
    let start_line = end_line.saturating_sub(num_lines);
    let lines_to_show: Vec<_> = lines[start_line..end_line].iter().collect();

    // Compute table column widths for visible lines
    let lines_vec: Vec<_> = lines_to_show.iter().map(|l| (*l).clone()).collect();
    let column_widths = compute_table_column_widths(&lines_vec);

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
            area.x,
            y,
            area.width,
            distance_from_bottom,
            current_pos,
            ContextType::Before,
            styling_enabled,
            gutter_area,
            &column_widths,
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
    styling_enabled: bool,
    gutter_area: Option<Rect>,
) {
    if area.height == 0 || current_line_idx >= lines.len() {
        return;
    }

    let num_lines = area.height as usize;
    let end_line = (current_line_idx + num_lines).min(lines.len());
    let lines_to_show = &lines[current_line_idx..end_line];

    // Compute table column widths for visible lines
    let column_widths = compute_table_column_widths(lines_to_show);

    // Render lines, with fading (farther = dimmer)
    for (i, line) in lines_to_show.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        render_line(
            frame,
            line,
            area.x,
            y,
            area.width,
            i,
            current_pos,
            ContextType::After,
            styling_enabled,
            gutter_area,
            &column_widths,
        );
    }
}

/// Mode for rendering words - either show text or blank spaces
#[derive(Clone, Copy, PartialEq)]
enum WordMode {
    Visible,
    Blank,
}

/// Render a single line at the given position
/// Words are shown or blanked based on their position relative to current_pos
#[allow(clippy::too_many_arguments)]
fn render_line(
    frame: &mut Frame,
    line: &DocLine,
    x: u16,
    y: u16,
    width: u16,
    distance: usize,
    current_pos: usize,
    context_type: ContextType,
    styling_enabled: bool,
    gutter_area: Option<Rect>,
    column_widths: &std::collections::HashMap<usize, usize>,
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

    // Render gutter hint if enabled
    if let Some(gutter) = gutter_area {
        let hint = first_token.token.block.hint_chars();
        if !hint.is_empty() {
            let gutter_style = Style::default().fg(GUIDE_COLOR);
            let hint_text = format!("{:>width$}", hint, width = GUTTER_WIDTH as usize);
            let hint_para = Paragraph::new(Line::from(Span::styled(hint_text, gutter_style)));
            let hint_area = Rect {
                x: gutter.x,
                y,
                width: GUTTER_WIDTH,
                height: 1,
            };
            frame.render_widget(hint_para, hint_area);
        }
    }

    // Calculate padding - only center headings, left-align others
    let content_width = calculate_line_width(line);
    let should_center = matches!(&first_token.token.block, BlockContext::Heading(_));
    let padding_size = calculate_padding(content_width, width as usize, should_center);
    let padding = " ".repeat(padding_size);
    let mut spans = vec![Span::raw(padding), Span::styled(prefix, style)];

    // Add words - visible or blank depending on position
    let mut prev_table_row: Option<usize> = None;
    let mut current_col: Option<usize> = None;
    let mut cell_content_width = 0usize;

    for (j, (global_idx, token)) in line.tokens.iter().enumerate() {
        let current_row = table_row(&token.token.block);
        let is_new_cell = current_row.is_some() && token.token.timing_hint.is_cell_start;

        // When starting a new cell, add padding for the previous cell
        if is_new_cell {
            if let Some(col) = current_col {
                let target_width = column_widths.get(&col).copied().unwrap_or(0);
                if cell_content_width < target_width {
                    let pad = " ".repeat(target_width - cell_content_width);
                    spans.push(Span::styled(pad, style));
                }
            }
            cell_content_width = 0;
            current_col = token.token.timing_hint.table_column;
        }

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

        let word_len = token.token.word.chars().count();
        let word_text = format!("{} ", token.token.word);
        let display_text = match mode {
            WordMode::Visible => word_text,
            WordMode::Blank => " ".repeat(word_len + 1),
        };

        // Track cell content width
        if current_col.is_some() {
            cell_content_width += word_len + 1; // word + space
        }

        let mut word_style = style; // Base gray style
        if styling_enabled {
            if matches!(&token.token.style, TokenStyle::Bold | TokenStyle::BoldItalic) {
                word_style = word_style.add_modifier(Modifier::BOLD);
            }
            if matches!(&token.token.style, TokenStyle::Italic | TokenStyle::BoldItalic) {
                word_style = word_style.add_modifier(Modifier::ITALIC);
            }
        }

        spans.push(Span::styled(display_text, word_style));
        prev_table_row = current_row;
    }

    // Add padding for the last cell
    if let Some(col) = current_col {
        let target_width = column_widths.get(&col).copied().unwrap_or(0);
        if cell_content_width < target_width {
            let pad = " ".repeat(target_width - cell_content_width);
            spans.push(Span::styled(pad, style));
        }
    }

    // Add trailing | for table rows
    if prev_table_row.is_some() {
        spans.push(Span::styled("|", style));
    }

    let line_area = Rect {
        x,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_short_line_centered() {
        // Headings should be centered (center=true)
        let padding = calculate_padding(20, 80, true);
        // Should center: (80 - 20) / 2 = 30
        assert_eq!(padding, 30);
    }

    #[test]
    fn test_heading_long_line_left_aligned() {
        // Long heading still gets left-aligned (above threshold)
        let padding = calculate_padding(60, 80, true);
        assert_eq!(padding, MIN_PADDING);
    }

    #[test]
    fn test_paragraph_always_left_aligned() {
        // Paragraphs should always be left-aligned (center=false)
        let padding = calculate_padding(20, 80, false);
        assert_eq!(padding, MIN_PADDING);
    }

    #[test]
    fn test_list_item_left_aligned() {
        // List items should be left-aligned
        let padding = calculate_padding(20, 80, false);
        assert_eq!(padding, MIN_PADDING);
    }

    #[test]
    fn test_table_cell_left_aligned() {
        // Table cells should be left-aligned
        let padding = calculate_padding(20, 80, false);
        assert_eq!(padding, MIN_PADDING);
    }

    #[test]
    fn test_heading_empty_content() {
        // Edge case: empty heading
        let padding = calculate_padding(0, 80, true);
        // Should center at 40
        assert_eq!(padding, 40);
    }
}
