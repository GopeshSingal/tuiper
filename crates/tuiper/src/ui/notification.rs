use crate::app::Notification;
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

const BANNER_HEIGHT: u16 = 3;

pub(super) fn draw_notification(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    notification: &Notification,
) {
    let height = BANNER_HEIGHT.min(area.height);
    if height == 0 {
        return;
    }

    let banner_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(height),
        width: area.width,
        height,
    };

    let block = default_block("Error", theme);
    let inner = block.inner(banner_area);
    frame.render_widget(block, banner_area);

    let text = vec![Line::from(Span::styled(
        notification.message.clone(),
        base_style(theme).fg(theme.get(ThemeField::TypedIncorrect)),
    ))];
    frame.render_widget(default_paragraph(text, theme), inner);
}
