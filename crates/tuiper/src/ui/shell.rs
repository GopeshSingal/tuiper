use crate::app::Screen;
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const SIDEBAR_WIDTH: u16 = 24;
const SIDEBAR_NAV_HINT: &str = "Shift + ↑/↓ or click to navigate";

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

fn sidebar_inner(area: Rect) -> Rect {
    Block::default()
        .borders(Borders::ALL)
        .title("Tuiper")
        .inner(area)
}

fn sidebar_nav_header_rows(content_width: u16) -> u16 {
    let width = content_width.max(1) as usize;
    let hint_lines = SIDEBAR_NAV_HINT.chars().count().div_ceil(width) as u16;
    hint_lines + 1
}

pub fn screen_at_sidebar_click(area: Rect, column: u16, row: u16) -> Option<Screen> {
    let (sidebar, _) = split_shell(area);
    if column < sidebar.x || column >= sidebar.x + sidebar.width {
        return None;
    }
    let inner = sidebar_inner(sidebar);
    if row < inner.y || row >= inner.y + inner.height {
        return None;
    }
    let rel_row = row - inner.y;
    let header_rows = sidebar_nav_header_rows(inner.width);
    if rel_row < header_rows {
        return None;
    }
    let idx = (rel_row - header_rows) as usize;
    NAV_ITEMS.get(idx).map(|item| item.screen)
}

pub fn draw_sidebar(frame: &mut Frame, area: Rect, theme: &Theme, screen: Screen) {
    let block = default_block("Tuiper", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint_style = base_style(theme).fg(Color::DarkGray);
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(SIDEBAR_NAV_HINT, hint_style)),
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
