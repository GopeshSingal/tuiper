use crate::app::Screen;
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

const SIDEBAR_WIDTH: u16 = 24;

struct NavItem {
    label: &'static str,
    key_hint: Option<&'static str>,
    screen: Screen,
}

const NAV_ITEMS: [NavItem; 3] = [
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
        label: "Customize",
        key_hint: Some("C"),
        screen: Screen::Config,
    },
];

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

    let mut lines: Vec<Line> = vec![Line::from("")];
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
        Paragraph::new(lines).style(base_style(theme)),
        inner,
    );
}
