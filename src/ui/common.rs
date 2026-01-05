//! Common UI constants and utilities shared across UI modules

use ratatui::style::Color;

/// Guide line color - slightly lighter than context text
pub const GUIDE_COLOR: Color = Color::Rgb(120, 120, 120);

/// Minimum left padding for content
pub const MIN_PADDING: usize = 2;

/// Threshold for centering: if content uses less than this fraction of width, center it
pub const CENTER_THRESHOLD: f32 = 0.6;

/// Fade zone characters: dotted (2) + dashed (2) + solid fade (2)
pub const FADE_DOTTED: usize = 2;
pub const FADE_DASHED: usize = 2;
pub const FADE_SOLID: usize = 2;
pub const FADE_TOTAL: usize = FADE_DOTTED + FADE_DASHED + FADE_SOLID;

/// Brightness levels for fade gradient
pub const BRIGHTNESS_MIN: u8 = 40;
pub const BRIGHTNESS_DOTTED_END: u8 = 60;
pub const BRIGHTNESS_DASHED_END: u8 = 80;
pub const BRIGHTNESS_SOLID_END: u8 = 120;

/// Calculate brightness for a position in the left fade zone
/// Returns (line_char, brightness) tuple
#[must_use]
pub fn fade_char_left(progress: usize) -> (char, u8) {
    if progress < FADE_DOTTED {
        let b = BRIGHTNESS_MIN + (progress * 20 / FADE_DOTTED.max(1)) as u8;
        ('┄', b)
    } else if progress < FADE_DOTTED + FADE_DASHED {
        let p = progress - FADE_DOTTED;
        let b = BRIGHTNESS_DOTTED_END + (p * 20 / FADE_DASHED.max(1)) as u8;
        ('╌', b)
    } else {
        let p = progress - FADE_DOTTED - FADE_DASHED;
        let b = BRIGHTNESS_DASHED_END + (p * 40 / FADE_SOLID.max(1)) as u8;
        ('─', b)
    }
}

/// Calculate brightness for a position in the right fade zone (mirror of left)
#[must_use]
pub fn fade_char_right(progress: usize) -> (char, u8) {
    // Mirror of left fade
    fade_char_left(progress)
}

/// Calculate left padding for content
/// - If `center` is true and content is short, centers the content
/// - Otherwise left-aligns with minimum padding
#[must_use]
pub fn calculate_padding(content_width: usize, available_width: usize, center: bool) -> usize {
    if available_width == 0 {
        return MIN_PADDING;
    }

    if center {
        let ratio = content_width as f32 / available_width as f32;
        if ratio < CENTER_THRESHOLD {
            return (available_width.saturating_sub(content_width)) / 2;
        }
    }

    MIN_PADDING
}
