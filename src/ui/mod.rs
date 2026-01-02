pub mod rsvp;
pub mod status;
pub mod outline;
pub mod context;

use ratatui::Frame;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::ViewMode;
    use ratatui::layout::{Layout, Direction, Constraint};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Main content
            Constraint::Length(2),   // Status bar
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
}

fn render_reading_view(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::layout::{Layout, Direction, Constraint};

    // Split into: context above, RSVP line, context below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Context above
            Constraint::Length(3),        // RSVP line (with padding)
            Constraint::Percentage(40),  // Context below
        ])
        .split(area);

    context::render_before(frame, app, chunks[0]);
    rsvp::render(frame, app, chunks[1]);
    context::render_after(frame, app, chunks[2]);
}
