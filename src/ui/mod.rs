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
            rsvp::render(frame, app, chunks[0]);
        }
        ViewMode::Outline => {
            outline::render(frame, app, chunks[0]);
        }
    }

    status::render(frame, app, chunks[1]);
}
