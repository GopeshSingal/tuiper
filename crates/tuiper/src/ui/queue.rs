use crate::theme::ThemePaint;

use super::common::{default_block, default_paragraph};

use ratatui::text::Line;
use ratatui::Frame;

pub(super) fn draw_queue(frame: &mut Frame, paint: &ThemePaint<'_>) {
    let area = frame.area();
    let block = default_block("Finding opponent...", paint);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("Q: leave queue"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(default_paragraph(text, paint), inner);
}
