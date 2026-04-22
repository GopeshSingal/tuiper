use crate::theme::{Theme, ThemeField};
use crate::typing::CharState;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn base_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.get(ThemeField::WindowBg))
        .fg(theme.get(ThemeField::BaseText))
}

fn typing_char_style(
    theme: &Theme,
    state: CharState,
    pending_error: bool,
    opponent_cursor_idx: Option<usize>,
    i: usize,
) -> Style {
    let opp = opponent_cursor_idx == Some(i);
    if opp && matches!(state, CharState::Current) {
        return if pending_error {
            base_style(theme)
                .bg(theme.get(ThemeField::OppCursorBg))
                .fg(theme.get(ThemeField::TypedIncorrect))
        } else {
            base_style(theme)
                .bg(theme.get(ThemeField::OppCursorBg))
                .fg(theme.get(ThemeField::OppCursorFg))
        };
    }

    let mut style = match state {
        CharState::Correct => base_style(theme).fg(theme.get(ThemeField::TypedCorrect)),
        CharState::Incorrect => base_style(theme)
            .fg(theme.get(ThemeField::TypedIncorrect))
            .add_modifier(Modifier::UNDERLINED),
        CharState::Current if pending_error => base_style(theme)
            .bg(theme.get(ThemeField::CursorBg))
            .fg(theme.get(ThemeField::TypedIncorrect)),
        CharState::Current => base_style(theme)
            .bg(theme.get(ThemeField::CursorBg))
            .fg(theme.get(ThemeField::CursorFg)),
        CharState::Untyped => base_style(theme).fg(theme.get(ThemeField::Untyped)),
    };

    if opp {
        style = style
            .bg(theme.get(ThemeField::OppCursorBg))
            .fg(theme.get(ThemeField::OppCursorFg));
    }
    style
}

pub(super) fn default_block<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(base_style(theme))
}

pub(super) fn default_paragraph<'a, T>(text: T, theme: &Theme) -> Paragraph<'a>
where
    T: Into<ratatui::text::Text<'a>>,
{
    Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .style(base_style(theme))
}

pub(super) fn line_from_typing(
    theme: &Theme,
    indexed_chars: impl Iterator<Item = (usize, char)>,
    at: impl Fn(usize) -> (CharState, bool),
    opponent_cursor_idx: Option<usize>,
    current_word_range: Option<std::ops::Range<usize>>,
) -> Line<'_> {
    Line::from(
        indexed_chars
            .map(|(i, c)| {
                let (state, pe) = at(i);
                let mut style = typing_char_style(theme, state, pe, opponent_cursor_idx, i);
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
