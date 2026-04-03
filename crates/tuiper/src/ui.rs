use crate::app::App;
use crate::app::Screen;
use crate::typing::CharState;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const APP_BG: Color = Color::Rgb(16, 16, 16);

fn base_style() -> Style {
    Style::default().bg(APP_BG)
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(base_style()), area);

    match app.screen {
        Screen::Lobby => draw_lobby(frame),
        Screen::Queue => draw_queue(frame),
        Screen::Race => draw_race(frame, app),
        Screen::Results => draw_results(frame, app),
    }
}

fn draw_lobby(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tuiper")
        .style(base_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("S: start a race"),
        Line::from("F: find an opponent"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .style(base_style()),
        inner,
    );
}

fn draw_queue(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Finding opponent")
        .style(base_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("Q: leave queue"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .style(base_style()),
        inner,
    );
}

fn draw_race(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let typing = app.typing();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    if let Some(t) = typing {
        let header_str = format!("Time: {}s | {:.0}s elapsed", t.value(), t.elapsed_secs());
        let stats = format!(
            "WPM: {:.0} Raw: {:.0} Acc: {:.1}% Consistency: {:.0}% | {}",
            t.wpm(),
            t.raw_wpm(),
            t.accuracy(),
            t.consistency(),
            header_str
        );
        frame.render_widget(
            Paragraph::new(stats).style(base_style().fg(Color::Cyan)),
            chunks[0],
        );

        if app.is_waiting_for_multiplayer_start() {
            let countdown = app.multiplayer_countdown_secs().unwrap_or(0);
            let waiting = format!("Starting in {}s...", countdown);
            frame.render_widget(
                Paragraph::new(waiting).style(base_style().fg(Color::Yellow)),
                chunks[1],
            );
        } else if app.is_multi() {
            let opponent_stats = format!("Opponent WPM: {:.0}, Opponent Chars: {}", app.opponent_wpm, app.opponent_chars);
            frame.render_widget(
                Paragraph::new(opponent_stats).style(base_style().fg(Color::Yellow)),
                chunks[1],
            );
        } else {
            frame.render_widget(Paragraph::new("").style(base_style()), chunks[1]);
        }

        let states = t.char_states();
        let pending_error = t.has_unfixed_error();
        let text_chars: Vec<char> = t.text().chars().collect();
        let opponent_cursor_idx =
            if app.is_multi() && !app.is_waiting_for_multiplayer_start() {
                let oc = app.opponent_chars as usize;
                if oc < text_chars.len() {
                    Some(oc)
                } else {
                    None
                }
            } else {
                None
            };
        let mut spans: Vec<Span> = Vec::new();
        for (i, &c) in text_chars.iter().enumerate() {
            let state = states.get(i).copied().unwrap_or(CharState::Untyped);
            let mut style = match state {
                CharState::Correct => base_style().fg(Color::Green),
                CharState::Incorrect => base_style()
                    .fg(Color::Red)
                    .add_modifier(Modifier::UNDERLINED),
                CharState::Current if pending_error => base_style().bg(Color::DarkGray).fg(Color::Red),
                CharState::Current => base_style().bg(Color::DarkGray).fg(Color::White),
                CharState::Untyped => base_style().fg(Color::DarkGray),
            };
            if opponent_cursor_idx == Some(i) {
                if matches!(state, CharState::Current) {
                    style = if pending_error {
                        base_style().bg(Color::LightMagenta).fg(Color::Red)
                    } else {
                        base_style().bg(Color::LightMagenta).fg(Color::White)
                    };
                } else {
                    style = style.bg(Color::Magenta);
                }
            }
            spans.push(Span::styled(c.to_string(), style));
        }
        let line = Line::from(spans);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Type the given text!")
            .style(base_style());
        let inner = block.inner(chunks[2]);
        frame.render_widget(block, chunks[2]);
        frame.render_widget(
            Paragraph::new(line)
                .wrap(Wrap { trim: false })
                .style(base_style()),
            inner,
        );
    } else {
        let msg = Paragraph::new("Loading...").block(
            Block::default()
                .borders(Borders::ALL)
                .style(base_style()),
        );
        frame.render_widget(msg, chunks[2]);
    }

    let hint = if app.is_waiting_for_multiplayer_start() {
        "Waiting for race to start"
    } else if app.is_multi() {
        "Race is in progress"
    } else {
        "Tab: restart"
    };
    frame.render_widget(
        Paragraph::new(hint).style(base_style().fg(Color::DarkGray)),
        chunks[3],
    );
}

fn draw_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Results ")
        .style(base_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(ref res) = app.race_results {
        let winner_str = match &res.winner {
            Some(protocols::Winner::Me) => "You won :)",
            Some(protocols::Winner::Opponent) => "You lost :(",
            None => "",
        };
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(winner_str, base_style().fg(Color::Green))),
            Line::from(""),
            Line::from(vec![
                Span::styled("You: ", base_style().fg(Color::Cyan)),
                Span::styled(
                    format!("{:.0} WPM  {:.1}% acc", res.me.wpm, res.me.accuracy),
                    base_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Opponent: ", base_style().fg(Color::Cyan)),
                Span::styled(
                    format!("{:.0} WPM  {:.1}% acc", res.opponent.wpm, res.opponent.accuracy),
                    base_style(),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Tab or Enter: race again  Q: lobby",
                base_style(),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(base_style()),
            inner,
        );
    } else if let Some(ref r) = app.result() {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("WPM: ", base_style().fg(Color::Cyan)),
                Span::styled(format!("{:.0}", r.wpm), base_style()),
            ]),
            Line::from(vec![
                Span::styled("Raw WPM: ", base_style().fg(Color::Cyan)),
                Span::styled(format!("{:.0}", r.raw_wpm), base_style()),
            ]),
            Line::from(vec![
                Span::styled("Accuracy: ", base_style().fg(Color::Cyan)),
                Span::styled(format!("{:.1}%", r.accuracy), base_style()),
            ]),
            Line::from(vec![
                Span::styled("Consistency: ", base_style().fg(Color::Cyan)),
                Span::styled(format!("{:.0}%", r.consistency), base_style()),
            ]),
            Line::from(vec![
                Span::styled("Time: ", base_style().fg(Color::Cyan)),
                Span::styled(format!("{:.1}", r.duration_secs), base_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Tab or Enter: try again    Q: lobby",
                base_style(),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(base_style()),
            inner,
        );
    } else {
        frame.render_widget(
            Paragraph::new("No results to display!").style(base_style()),
            inner,
        );
    }
}
