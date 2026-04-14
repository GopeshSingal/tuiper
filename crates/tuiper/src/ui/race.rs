use crate::app::{App, RaceMode};
use crate::theme::{Theme, ThemeField};
use crate::typing::{CharState, TypingState};

use super::common::{base_style, default_block, default_paragraph, line_from_typing};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

struct RaceLayout {
    header: Rect,
    middle: Rect,
    body: Rect,
    footer: Rect,
}

fn race_layout(area: Rect, app: &App, is_typing: bool) -> RaceLayout {
    let middle_h = if is_typing && (app.is_waiting_for_multiplayer_start() || app.is_multi()) {
        1
    } else {
        0
    };
    let header_h = if is_typing { 4 } else { 2 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Length(middle_h),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    RaceLayout {
        header: chunks[0],
        middle: chunks[1],
        body: chunks[2],
        footer: chunks[3],
    }
}

fn race_progress_line(theme: &Theme, width: u16, ratio: f64) -> Line<'static> {
    const DOT_TRACK: &str = "·";
    const DOT_FILL: &str = "•";
    const DOT_FILL_CURRENT: &str = "●";

    let w = width as usize;
    if w < 2 {
        return Line::from("");
    }
    let inner = w - 2;
    let ratio = ratio.clamp(0.0, 1.0);
    let filled_end = ratio * inner as f64;
    let filled_count = (filled_end.floor() as usize).min(inner);

    let bracket_style = base_style(theme).fg(theme.get(ThemeField::Untyped));
    let track_style = base_style(theme).fg(theme.get(ThemeField::Untyped));
    let fill_style = base_style(theme).fg(theme.get(ThemeField::TypedCorrect));
    let fill_current_style = base_style(theme)
        .fg(theme.get(ThemeField::TypedCorrect))
        .add_modifier(Modifier::BOLD);

    let mut spans = Vec::with_capacity(inner + 2);
    spans.push(Span::styled("[", bracket_style));

    for i in 0..inner {
        if i >= filled_count {
            spans.push(Span::styled(DOT_TRACK, track_style));
        } else if i == filled_count - 1 {
            spans.push(Span::styled(DOT_FILL_CURRENT, fill_current_style));
        } else {
            spans.push(Span::styled(DOT_FILL, fill_style));
        }
    }

    spans.push(Span::styled("]", bracket_style));
    Line::from(spans)
}

fn render_race_header(frame: &mut Frame, theme: &Theme, app: &App, t: &TypingState, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let finish_mode = if app.is_multi() {
        RaceMode::Time
    } else {
        app.mode
    };

    let ratio = t.progress_ratio(finish_mode);

    let stats = format!(
        "WPM: {:.0} Raw: {:.0} Acc: {:.1}% Consistency: {:.0}%",
        t.wpm(),
        t.raw_wpm(),
        t.accuracy(),
        t.consistency(),
    );

    let stats_block = default_block("Stats", theme);
    let stats_inner = stats_block.inner(rows[0]);
    frame.render_widget(stats_block, rows[0]);
    frame.render_widget(
        Paragraph::new(stats).style(base_style(theme).fg(Color::Cyan)),
        stats_inner,
    );

    let bar = race_progress_line(theme, rows[1].width, ratio);
    frame.render_widget(default_paragraph(bar, theme), rows[1]);
}

fn render_race_middle(frame: &mut Frame, theme: &Theme, app: &App, area: Rect) {
    if app.is_waiting_for_multiplayer_start() {
        let countdown = app.multiplayer_countdown_secs().unwrap_or(0);
        let waiting_str = format!("Starting in {}s...", countdown);
        frame.render_widget(
            Paragraph::new(waiting_str).style(base_style(theme).fg(Color::Yellow)),
            area,
        );
    } else if app.is_multi() {
        let opponent_stats = format!("Opponent WPM: {:.0}", app.opponent_wpm);
        frame.render_widget(
            Paragraph::new(opponent_stats).style(base_style(theme).fg(Color::Yellow)),
            area,
        );
    } else {
        frame.render_widget(default_paragraph("", theme), area);
    }
}

fn render_race_text(frame: &mut Frame, theme: &Theme, app: &App, t: &TypingState, area: Rect) {
    let states = t.char_states();
    let pending_error = t.has_unfixed_error();
    let text_chars: Vec<char> = t.text().chars().collect();

    let opponent_cursor_idx = if app.is_multi() && !app.is_waiting_for_multiplayer_start() {
        let oc = app.opponent_chars as usize;
        (oc < text_chars.len()).then_some(oc)
    } else {
        None
    };

    let line = line_from_typing(
        theme,
        text_chars.iter().cloned().enumerate(),
        |i| {
            (
                states.get(i).copied().unwrap_or(CharState::Untyped),
                pending_error,
            )
        },
        opponent_cursor_idx,
    );

    let block = default_block("Type here", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(default_paragraph(line, theme), inner);
}

fn render_race_loading(frame: &mut Frame, theme: &Theme, area: Rect) {
    let msg = Paragraph::new("Loading...").block(default_block("", theme));
    frame.render_widget(msg, area);
}

fn render_race_footer(frame: &mut Frame, theme: &Theme, app: &App, area: Rect) {
    let hint = if app.is_waiting_for_multiplayer_start() {
        "Waiting for race to start"
    } else if app.is_multi() {
        "Race is in progress"
    } else {
        "Tab / Enter: restart"
    };

    frame.render_widget(
        Paragraph::new(hint).style(base_style(theme).fg(Color::DarkGray)),
        area,
    );
}

pub(super) fn draw_race(frame: &mut Frame, theme: &Theme, app: &App) {
    let typing = app.typing();
    let layout = race_layout(frame.area(), app, typing.is_some());

    if let Some(t) = typing {
        render_race_header(frame, theme, app, t, layout.header);
        render_race_middle(frame, theme, app, layout.middle);
        render_race_text(frame, theme, app, t, layout.body);
    } else {
        render_race_loading(frame, theme, layout.body);
    }

    render_race_footer(frame, theme, app, layout.footer);
}
