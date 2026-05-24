use crate::theme::{CursorStyle, Theme, ThemeField};
use crate::typing::CharState;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn base_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.get(ThemeField::WindowBg))
        .fg(theme.get(ThemeField::BaseText))
}

fn cursor_text_fg(theme: &Theme, pending_error: bool) -> Color {
    if pending_error {
        theme.get(ThemeField::TypedIncorrect)
    } else {
        theme.get(ThemeField::CursorFg)
    }
}

fn opponent_cursor_text_fg(theme: &Theme, pending_error: bool) -> Color {
    if pending_error {
        theme.get(ThemeField::TypedIncorrect)
    } else {
        theme.get(ThemeField::OppCursorFg)
    }
}

fn cursor_current_style(theme: &Theme, pending_error: bool) -> Style {
    let text_fg = cursor_text_fg(theme, pending_error);
    match theme.cursor_style {
        CursorStyle::Block => base_style(theme)
            .fg(text_fg)
            .bg(theme.get(ThemeField::CursorBg)),
        CursorStyle::Underscore => base_style(theme)
            .fg(text_fg)
            .underline_color(theme.get(ThemeField::CursorBg))
            .add_modifier(Modifier::UNDERLINED),
    }
}

fn opponent_cursor_current_style(theme: &Theme, pending_error: bool) -> Style {
    let text_fg = opponent_cursor_text_fg(theme, pending_error);
    match theme.cursor_style {
        CursorStyle::Block => base_style(theme)
            .fg(text_fg)
            .bg(theme.get(ThemeField::OppCursorBg)),
        CursorStyle::Underscore => base_style(theme)
            .fg(text_fg)
            .underline_color(theme.get(ThemeField::OppCursorBg))
            .add_modifier(Modifier::UNDERLINED),
    }
}

fn underscore_cursor_display(c: char, line_color: Color) -> Option<(String, Style)> {
    if c != ' ' {
        return None;
    }
    Some(("▁".to_string(), Style::default().fg(line_color)))
}

fn char_width(c: char) -> u16 {
    u16::from(!c.is_control())
}

fn typing_line_starts(chars: &[char], max_width: u16) -> Vec<usize> {
    if max_width == 0 {
        return vec![0];
    }
    if chars.is_empty() {
        return vec![0];
    }

    let max = max_width as usize;
    let segment_width = |start: usize, end: usize| -> usize {
        chars[start..end]
            .iter()
            .map(|c| char_width(*c) as usize)
            .sum()
    };

    let mut starts = vec![0];
    let mut line_start = 0;

    while line_start < chars.len() {
        if segment_width(line_start, chars.len()) <= max {
            break;
        }

        let mut end = line_start;
        let mut width = 0usize;
        let mut last_space_break = None;

        while end < chars.len() {
            let cw = char_width(chars[end]) as usize;
            if width + cw > max {
                break;
            }
            if chars[end].is_whitespace() {
                last_space_break = Some(end + 1);
            }
            width += cw;
            end += 1;
        }

        let next_start = if let Some(break_at) = last_space_break {
            break_at
        } else if end > line_start {
            end
        } else {
            line_start + 1
        };

        if next_start <= line_start {
            break;
        }

        starts.push(next_start);
        line_start = next_start;
    }

    starts
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
        return opponent_cursor_current_style(theme, pending_error);
    }

    let mut style = match state {
        CharState::Correct => base_style(theme).fg(theme.get(ThemeField::TypedCorrect)),
        CharState::Incorrect => base_style(theme)
            .fg(theme.get(ThemeField::TypedIncorrect))
            .add_modifier(Modifier::UNDERLINED),
        CharState::Current => cursor_current_style(theme, pending_error),
        CharState::Untyped => base_style(theme).fg(theme.get(ThemeField::Untyped)),
    };

    if opp {
        style = match theme.cursor_style {
            CursorStyle::Block => style
                .bg(theme.get(ThemeField::OppCursorBg))
                .fg(theme.get(ThemeField::OppCursorFg)),
            CursorStyle::Underscore => style
                .fg(theme.get(ThemeField::OppCursorFg))
                .underline_color(theme.get(ThemeField::OppCursorBg))
                .add_modifier(Modifier::UNDERLINED),
        };
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

pub(super) fn typing_paragraph<'a, T>(text: T, theme: &Theme) -> Paragraph<'a>
where
    T: Into<ratatui::text::Text<'a>>,
{
    Paragraph::new(text).style(base_style(theme))
}

pub(super) fn lines_from_typing<'a>(
    theme: &'a Theme,
    width: u16,
    text_chars: &'a [char],
    at: impl Fn(usize) -> (CharState, bool),
    opponent_cursor_idx: Option<usize>,
    current_word_range: Option<std::ops::Range<usize>>,
) -> Vec<Line<'a>> {
    let starts = typing_line_starts(text_chars, width);
    let current_word_range = current_word_range;
    starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = starts.get(i + 1).copied().unwrap_or(text_chars.len());
            line_from_typing(
                theme,
                text_chars[start..end]
                    .iter()
                    .enumerate()
                    .map(|(j, c)| (start + j, *c)),
                |idx| at(idx),
                opponent_cursor_idx,
                current_word_range.clone(),
            )
        })
        .collect()
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
                let opp = opponent_cursor_idx == Some(i);
                let (text, mut style) = if state == CharState::Current {
                    if opp {
                        if theme.cursor_style == CursorStyle::Underscore {
                            if let Some(display) =
                                underscore_cursor_display(c, theme.get(ThemeField::OppCursorBg))
                            {
                                display
                            } else {
                                (c.to_string(), opponent_cursor_current_style(theme, pe))
                            }
                        } else {
                            (c.to_string(), opponent_cursor_current_style(theme, pe))
                        }
                    } else if theme.cursor_style == CursorStyle::Underscore {
                        if let Some(display) =
                            underscore_cursor_display(c, theme.get(ThemeField::CursorBg))
                        {
                            display
                        } else {
                            (c.to_string(), cursor_current_style(theme, pe))
                        }
                    } else {
                        (c.to_string(), cursor_current_style(theme, pe))
                    }
                } else {
                    (
                        c.to_string(),
                        typing_char_style(theme, state, pe, opponent_cursor_idx, i),
                    )
                };
                if current_word_range
                    .as_ref()
                    .is_some_and(|range| range.contains(&i))
                {
                    style = style.add_modifier(Modifier::BOLD);
                }
                Span::styled(text, style)
            })
            .collect::<Vec<Span>>(),
    )
}
