use crate::types::{TimedToken, Section};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Reading,
    Outline,
}

pub struct App {
    tokens: Vec<TimedToken>,
    sections: Vec<Section>,
    position: usize,
    wpm: u16,
    paused: bool,
    view_mode: ViewMode,
    outline_selection: usize,
    show_help: bool,
}

impl App {
    pub fn new(tokens: Vec<TimedToken>, sections: Vec<Section>) -> Self {
        Self {
            tokens,
            sections,
            position: 0,
            wpm: 300,
            paused: false,
            view_mode: ViewMode::Reading,
            outline_selection: 0,
            show_help: false,
        }
    }

    // Getters
    #[must_use]
    pub const fn position(&self) -> usize { self.position }
    #[must_use]
    pub const fn wpm(&self) -> u16 { self.wpm }
    #[must_use]
    pub const fn is_paused(&self) -> bool { self.paused }
    #[must_use]
    pub const fn view_mode(&self) -> ViewMode { self.view_mode }
    #[must_use]
    pub fn current_token(&self) -> Option<&TimedToken> { self.tokens.get(self.position) }
    #[must_use]
    pub fn tokens(&self) -> &[TimedToken] { &self.tokens }
    #[must_use]
    pub fn sections(&self) -> &[Section] { &self.sections }
    #[must_use]
    pub const fn outline_selection(&self) -> usize { self.outline_selection }

    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.tokens.is_empty() { 0.0 }
        else { self.position as f64 / self.tokens.len() as f64 }
    }

    // Mutations
    pub fn toggle_pause(&mut self) { self.paused = !self.paused; }

    pub fn increase_wpm(&mut self) { self.wpm = (self.wpm + 25).min(800); }

    pub fn decrease_wpm(&mut self) { self.wpm = self.wpm.saturating_sub(25).max(100); }

    pub fn advance(&mut self) {
        if self.position < self.tokens.len().saturating_sub(1) {
            self.position += 1;
        }
    }

    pub fn rewind_sentence(&mut self) {
        self.position = self.position.saturating_sub(10);
    }

    pub fn skip_sentence(&mut self) {
        self.position = (self.position + 10).min(self.tokens.len().saturating_sub(1));
    }

    pub fn toggle_outline(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Reading => ViewMode::Outline,
            ViewMode::Outline => ViewMode::Reading,
        };
    }

    pub fn outline_up(&mut self) {
        self.outline_selection = self.outline_selection.saturating_sub(1);
    }

    pub fn outline_down(&mut self) {
        if !self.sections.is_empty() {
            self.outline_selection = (self.outline_selection + 1).min(self.sections.len() - 1);
        }
    }

    pub fn jump_to_section(&mut self) {
        if let Some(section) = self.sections.get(self.outline_selection) {
            self.position = section.token_start;
            self.view_mode = ViewMode::Reading;
        }
    }

    #[must_use]
    pub const fn show_help(&self) -> bool { self.show_help }

    pub fn toggle_help(&mut self) { self.show_help = !self.show_help; }

    #[must_use]
    pub fn current_section_title(&self) -> Option<&str> {
        for section in self.sections.iter().rev() {
            if self.position >= section.token_start {
                return Some(&section.title);
            }
        }
        None
    }

    /// Get tokens around current position for context display
    #[must_use]
    pub fn context_tokens(&self, before: usize, after: usize) -> (&[TimedToken], &[TimedToken]) {
        let start = self.position.saturating_sub(before);
        let end = (self.position + after + 1).min(self.tokens.len());

        let before_slice = &self.tokens[start..self.position];
        let after_slice = if self.position + 1 < end {
            &self.tokens[self.position + 1..end]
        } else {
            &[]
        };

        (before_slice, after_slice)
    }
}
