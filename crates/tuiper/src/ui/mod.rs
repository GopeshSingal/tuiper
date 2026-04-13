mod common;
mod config;
mod lobby;
mod queue;
mod race;
mod results;

use crate::app::{App, Screen};

use common::base_style;

use ratatui::widgets::Block;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.theme;
    frame.render_widget(Block::default().style(base_style(theme)), area);

    match app.screen {
        Screen::Lobby => lobby::draw_lobby(frame, theme, app),
        Screen::Queue => queue::draw_queue(frame, theme),
        Screen::Race => race::draw_race(frame, theme, app),
        Screen::Results => results::draw_results(frame, theme, app),
        Screen::Config => config::draw_config(frame, theme, app),
    }
}
