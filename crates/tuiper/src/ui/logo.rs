use crate::theme::{Theme, ThemeField};

use super::common::base_style;

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

const LOGO_LINES: &[&str] = &[
    "  _______  _    _  _____  _____   ______  _____  ",
    " |__   __|| |  | ||_   _||  __ \\ |  ____||  __ \\ ",
    "    | |   | |  | |  | |  | |__) || |__   | |__) |",
    "    | |   | |  | |  | |  |  ___/ |  __|  |  _  / ",
    "    | |   | |__| | _| |_ | |     | |____ | | \\ \\ ",
    "    |_|    \\____/ |_____||_|     |______||_|  \\_\\",
];

pub(super) const LOGO_WIDTH: u16 = 49;

fn logo_style(theme: &Theme) -> ratatui::style::Style {
    base_style(theme)
        .fg(theme.get(ThemeField::TypedCorrect))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn logo_lines(theme: &Theme) -> Vec<Line<'static>> {
    let style = logo_style(theme);
    LOGO_LINES
        .iter()
        .map(|line| Line::from(Span::styled((*line).to_string(), style)))
        .collect()
}

pub(super) fn compact_logo_line(theme: &Theme) -> Line<'static> {
    Line::from(Span::styled("Tuiper", logo_style(theme)))
}
