use crate::app::{App, LoginField};
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;

fn field_line(
    theme: &Theme,
    label: &str,
    value: &str,
    focused: bool,
) -> Line<'static> {
    let label_style = base_style(theme).add_modifier(Modifier::BOLD);
    let value_style = if focused {
        base_style(theme).add_modifier(Modifier::REVERSED)
    } else {
        base_style(theme)
    };
    Line::from(vec![
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

pub(super) fn draw_login(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = default_block("Sign in", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(1), Constraint::Length(3)])
        .split(inner);

    let username_value = if app.login_username.is_empty() && app.login_focus == LoginField::Username
    {
        " ".to_string()
    } else {
        app.login_username.clone()
    };

    let password_display = if app.login_password.is_empty()
        && app.login_focus == LoginField::Password
    {
        " ".to_string()
    } else {
        "•".repeat(app.login_password.chars().count())
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "Welcome to Tuiper",
            base_style(theme)
                .fg(theme.get(ThemeField::TypedCorrect))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        field_line(
            theme,
            "Username",
            &username_value,
            app.login_focus == LoginField::Username,
        ),
        field_line(
            theme,
            "Password",
            &password_display,
            app.login_focus == LoginField::Password,
        ),
    ];

    if let Some(err) = &app.login_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            err.clone(),
            base_style(theme).fg(theme.get(ThemeField::TypedIncorrect)),
        )));
    }

    frame.render_widget(default_paragraph(lines, theme), chunks[0]);

    let hints = vec![
        Line::from(Span::styled(
            "Leave username empty and press Enter to play as a guest",
            base_style(theme).fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Tab: switch field    Enter: continue    Esc: quit",
            base_style(theme).fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(default_paragraph(hints, theme), chunks[2]);
}
