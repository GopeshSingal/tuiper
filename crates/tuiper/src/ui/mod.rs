mod common;
mod config;
mod leaderboard;
mod lobby;
mod queue;
mod race;
mod results;
mod shell;

use crate::app::{App, Screen};

use common::base_style;

use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.theme;
    frame.render_widget(Block::default().style(base_style(theme)), area);

    if app.screen.uses_shell() {
        let (sidebar, main) = shell::split_shell(area);
        shell::draw_sidebar(frame, sidebar, theme, app.screen);
        draw_main_content(frame, main, app);
    } else {
        draw_main_content(frame, area, app);
    }
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    match app.screen {
        Screen::Lobby => lobby::draw_lobby(frame, area, theme, app),
        Screen::Queue => queue::draw_queue(frame, area, theme),
        Screen::Race => race::draw_race(frame, area, theme, app),
        Screen::Results => results::draw_results(frame, area, theme, app),
        Screen::Config => config::draw_config(frame, area, theme, app),
        Screen::Leaderboard => leaderboard::draw_leaderboard(frame, area, theme, app),
    }
}
