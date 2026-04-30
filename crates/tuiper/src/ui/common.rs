use crate::theme::{ThemeField, ThemePaint};
use crate::typing::CharState;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn base_style(paint: &ThemePaint<'_>) -> Style {
    Style::default()
        .bg(paint.get(ThemeField::WindowBg))
        .fg(paint.get(ThemeField::BaseText))
}

fn typing_char_style(
    paint: &ThemePaint<'_>,
    state: CharState,
    pending_error: bool,
    opponent_cursor_idx: Option<usize>,
    i: usize,
) -> Style {
    let opp = opponent_cursor_idx == Some(i);
    if opp && matches!(state, CharState::Current) {
        return if pending_error {
            base_style(paint)
                .bg(paint.get(ThemeField::OppCursorBg))
                .fg(paint.get(ThemeField::TypedIncorrect))
        } else {
            base_style(paint)
                .bg(paint.get(ThemeField::OppCursorBg))
                .fg(paint.get(ThemeField::OppCursorFg))
        };
    }

    let mut style = match state {
        CharState::Correct => base_style(paint).fg(paint.get(ThemeField::TypedCorrect)),
        CharState::Incorrect => base_style(paint)
            .fg(paint.get(ThemeField::TypedIncorrect))
            .add_modifier(Modifier::UNDERLINED),
        CharState::Current if pending_error => base_style(paint)
            .bg(paint.get(ThemeField::CursorBg))
            .fg(paint.get(ThemeField::TypedIncorrect)),
        CharState::Current => base_style(paint)
            .bg(paint.get(ThemeField::CursorBg))
            .fg(paint.get(ThemeField::CursorFg)),
        CharState::Untyped => base_style(paint).fg(paint.get(ThemeField::Untyped)),
    };

    if opp {
        style = style
            .bg(paint.get(ThemeField::OppCursorBg))
            .fg(paint.get(ThemeField::OppCursorFg));
    }
    style
}

pub(super) fn default_block<'a>(title: &'a str, paint: &ThemePaint<'_>) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(base_style(paint))
}

pub(super) fn default_paragraph<'a, T>(text: T, paint: &ThemePaint<'_>) -> Paragraph<'a>
where
    T: Into<ratatui::text::Text<'a>>,
{
    Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .style(base_style(paint))
}

pub(super) fn line_from_typing<'a>(
    paint: &'a ThemePaint<'a>,
    indexed_chars: impl Iterator<Item = (usize, char)>,
    at: impl Fn(usize) -> (CharState, bool),
    opponent_cursor_idx: Option<usize>,
    current_word_range: Option<std::ops::Range<usize>>,
) -> Line<'a> {
    Line::from(
        indexed_chars
            .map(|(i, c)| {
                let (state, pe) = at(i);
                let mut style = typing_char_style(paint, state, pe, opponent_cursor_idx, i);
                if current_word_range
                    .as_ref()
                    .is_some_and(|range| range.contains(&i))
                {
                    style = style.add_modifier(Modifier::BOLD);
                }
                Span::styled(c.to_string(), style)
            })
            .collect::<Vec<Span>>(),
    )
}
