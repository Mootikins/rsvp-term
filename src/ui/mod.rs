pub mod common;
pub mod context;
pub mod help;
pub mod outline;
pub mod rsvp;
pub mod status;

use crate::app::App;
use ratatui::Frame;

/// Width of the gutter column for hint chars
pub const GUTTER_WIDTH: u16 = 4;
/// Padding between gutter and content
pub const GUTTER_PADDING: u16 = 2;

pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::ViewMode;
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(2), // Status bar
        ])
        .split(frame.area());

    match app.view_mode() {
        ViewMode::Reading => {
            render_reading_view(frame, app, chunks[0]);
        }
        ViewMode::Outline => {
            outline::render(frame, app, chunks[0]);
        }
    }

    status::render(frame, app, chunks[1]);

    // Render help overlay if active
    if app.show_help() {
        help::render(frame, frame.area());
    }
}

fn render_reading_view(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};

    // If hint_chars enabled, split horizontally first to create gutter area
    let (gutter_area, content_area) = if app.hint_chars_enabled {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(GUTTER_WIDTH + GUTTER_PADDING), // Gutter + padding
                Constraint::Min(0),                                // Content
            ])
            .split(area);
        (Some(horizontal[0]), horizontal[1])
    } else {
        (None, area)
    };

    // Split content into: context above, RSVP line, context below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Context above
            Constraint::Length(3),      // RSVP line (with padding)
            Constraint::Percentage(40), // Context below
        ])
        .split(content_area);

    context::render_before(frame, app, chunks[0], gutter_area);
    rsvp::render(frame, app, chunks[1], gutter_area);
    context::render_after(frame, app, chunks[2], gutter_area);
}
