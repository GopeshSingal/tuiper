mod common;
mod config;
mod lobby;
mod queue;
mod race;
mod results;

use crate::app::{App, Screen};

use common::base_style;

use crate::theme::ThemePaint;
use ratatui::widgets::Block;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let paint = ThemePaint::new(&app.theme, app.display_truecolor);
    frame.render_widget(Block::default().style(base_style(&paint)), area);

    match app.screen {
        Screen::Lobby => lobby::draw_lobby(frame, &paint, app),
        Screen::Queue => queue::draw_queue(frame, &paint),
        Screen::Race => race::draw_race(frame, &paint, app),
        Screen::Results => results::draw_results(frame, &paint, app),
        Screen::Config => config::draw_config(frame, &paint, app),
    }
}
