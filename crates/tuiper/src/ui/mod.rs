mod common;
mod config;
mod leaderboard;
mod lobby;
mod login;
mod logo;
mod notification;
mod queue;
mod race;
mod results;
mod shell;
mod statistics;

pub use shell::{adjacent_shell_screen, screen_at_sidebar_click, split_shell};

use crate::app::{App, Screen};

use common::base_style;

use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App, ws_url: &str) {
    let area = frame.area();
    frame.render_widget(Block::default().style(base_style(&app.theme)), area);

    if app.screen.uses_shell() {
        let (sidebar, main) = shell::split_shell(area);
        shell::draw_sidebar(frame, sidebar, &app.theme, app.screen);
        draw_main_content(frame, main, app, ws_url);
    } else {
        draw_main_content(frame, area, app, ws_url);
    }

    if let Some(ref notification) = app.notification {
        notification::draw_notification(frame, area, &app.theme, notification);
    }
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &mut App, ws_url: &str) {
    let theme = &app.theme;
    match app.screen {
        Screen::Login => login::draw_login(frame, area, theme, app, ws_url),
        Screen::Lobby => lobby::draw_lobby(frame, area, theme, app),
        Screen::Queue => queue::draw_queue(frame, area, theme),
        Screen::Race => race::draw_race(frame, area, theme, app),
        Screen::Results => results::draw_results(frame, area, theme, app),
        Screen::Config => config::draw_config(frame, area, theme, app),
        Screen::Leaderboard => leaderboard::draw_leaderboard(frame, area, theme, app),
        Screen::Statistics => statistics::draw_statistics(frame, area, app),
    }
}
