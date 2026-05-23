use crate::app::Screen;
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

const SIDEBAR_WIDTH: u16 = 24;

struct NavItem {
    label: &'static str,
    key_hint: Option<&'static str>,
    screen: Screen,
}

pub const SHELL_SCREENS: [Screen; 4] = [
    Screen::Lobby,
    Screen::Leaderboard,
    Screen::Statistics,
    Screen::Config,
];

const NAV_ITEMS: [NavItem; 4] = [
    NavItem {
        label: "Lobby",
        key_hint: Some("Q"),
        screen: Screen::Lobby,
    },
    NavItem {
        label: "Leaderboard",
        key_hint: Some("L"),
        screen: Screen::Leaderboard,
    },
    NavItem {
        label: "Statistics",
        key_hint: Some("T"),
        screen: Screen::Statistics,
    },
    NavItem {
        label: "Customize",
        key_hint: Some("C"),
        screen: Screen::Config,
    },
];

pub fn adjacent_shell_screen(current: Screen, delta: i32) -> Screen {
    let idx = SHELL_SCREENS
        .iter()
        .position(|&s| s == current)
        .unwrap_or(0);
    let next = (idx as i32 + delta).rem_euclid(SHELL_SCREENS.len() as i32) as usize;
    SHELL_SCREENS[next]
}

pub fn split_shell(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn draw_sidebar(frame: &mut Frame, area: Rect, theme: &Theme, screen: Screen) {
    let block = default_block("Tuiper", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint_style = base_style(theme).fg(Color::DarkGray);
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("Shift + ↑/↓ to navigate", hint_style)),
        Line::from(""),
    ];
    for item in NAV_ITEMS {
        let active = screen == item.screen;
        let prefix = if active { "> " } else { "  " };
        let label_style = if active {
            base_style(theme)
                .fg(theme.get(ThemeField::TypedCorrect))
                .add_modifier(Modifier::BOLD)
        } else {
            base_style(theme).fg(Color::DarkGray)
        };
        let mut spans = vec![Span::styled(format!("{prefix}{}", item.label), label_style)];
        if let Some(key) = item.key_hint {
            spans.push(Span::styled(
                format!(" ({key})"),
                base_style(theme).fg(Color::DarkGray),
            ));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        inner,
    );
}
