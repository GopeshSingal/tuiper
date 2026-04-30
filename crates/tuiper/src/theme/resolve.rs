//! Truecolor detection and mapping theme RGB to xterm-256 for reduced terminals.

use ratatui::style::Color;

use super::{Theme, ThemeField};

/// Whether stdout supports 24-bit color (`has_16m`). Respects `NO_COLOR`, pipes, etc.
pub fn detect_truecolor() -> bool {
    supports_color::on(supports_color::Stream::Stdout)
        .map(|s| s.has_16m)
        .unwrap_or(false)
}

/// Map sRGB to the nearest xterm 256-color index (16–231 cube, 232–255 grayscale ramp).
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        return if r < 8 {
            16
        } else if r > 248 {
            231
        } else {
            ((r - 8) / 10) + 232
        };
    }
    let r = ((r as u16 * 5) / 255).min(5) as u8;
    let g = ((g as u16 * 5) / 255).min(5) as u8;
    let b = ((b as u16 * 5) / 255).min(5) as u8;
    16 + 36 * r + 6 * g + b
}

/// Pass through RGB when the terminal supports truecolor; otherwise map `Color::Rgb` to indexed.
pub fn resolve_for_terminal(color: Color, truecolor: bool) -> Color {
    if truecolor {
        return color;
    }
    match color {
        Color::Rgb(r, g, b) => Color::Indexed(rgb_to_ansi256(r, g, b)),
        other => other,
    }
}

/// Resolved theme colors for the current terminal capability.
#[derive(Debug, Clone, Copy)]
pub struct ThemePaint<'a> {
    pub theme: &'a Theme,
    pub display_truecolor: bool,
}

impl<'a> ThemePaint<'a> {
    pub fn new(theme: &'a Theme, display_truecolor: bool) -> Self {
        Self {
            theme,
            display_truecolor,
        }
    }

    pub fn get(&self, field: ThemeField) -> Color {
        resolve_for_terminal(self.theme.get(field), self.display_truecolor)
    }

    pub fn resolve(&self, color: Color) -> Color {
        resolve_for_terminal(color, self.display_truecolor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gray_ramp_mid() {
        assert_eq!(rgb_to_ansi256(128, 128, 128), 244);
    }

    #[test]
    fn pure_red_cube() {
        let i = rgb_to_ansi256(255, 0, 0);
        assert!((16..=231).contains(&i));
    }

    #[test]
    fn pure_green_cube() {
        let i = rgb_to_ansi256(0, 255, 0);
        assert!((16..=231).contains(&i));
    }

    #[test]
    fn resolve_passes_rgb_when_truecolor() {
        let c = Color::Rgb(10, 20, 30);
        assert_eq!(resolve_for_terminal(c, true), c);
    }

    #[test]
    fn resolve_maps_rgb_when_not_truecolor() {
        let c = Color::Rgb(255, 0, 0);
        match resolve_for_terminal(c, false) {
            Color::Indexed(i) => assert_ne!(i, 0),
            _ => panic!("expected Indexed"),
        }
    }

    #[test]
    fn resolve_leaves_named() {
        let c = Color::Cyan;
        assert_eq!(resolve_for_terminal(c, false), c);
    }
}
