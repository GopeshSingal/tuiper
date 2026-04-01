use crate::app::App;
use crate::app::Screen;
use crate::typing::CharState;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Lobby => draw_lobby(frame),
        Screen::Queue => draw_queue(frame),
        Screen::Race => draw_race(frame, app),
        Screen::Results => draw_results(frame, app),
    }
}

fn draw_lobby(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default().borders(Borders::ALL).title("Tuiper");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("S: start a race"),
        Line::from("F: find an opponent"),
        Line::from("Esc / Q: Quit"),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
}

fn draw_queue(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default().borders(Borders::ALL).title("Finding opponent");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("Esc / Q: Quit"),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
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
        frame.render_widget(Paragraph::new(stats).style(Style::default().fg(Color::Cyan)), chunks[0]);

        if app.is_waiting_for_multiplayer_start() {
            let countdown = app.multiplayer_countdown_secs().unwrap_or(0);
            let waiting = format!("Starting in {}s...", countdown);
            frame.render_widget(Paragraph::new(waiting).style(Style::default().fg(Color::Yellow)), chunks[1]);
        } else if app.is_multi() {
            let opponent_stats = format!("Opponent WPM: {:.0}, Opponent Chars: {}", app.opponent_wpm, app.opponent_chars);
            frame.render_widget(Paragraph::new(opponent_stats).style(Style::default().fg(Color::Yellow)), chunks[1]);
        } else {
            frame.render_widget(Paragraph::new(""), chunks[1]);
        }

        let states = t.char_states();
        let text_chars: Vec<char> = t.text().chars().collect();
        let mut spans: Vec<Span> = Vec::new();
        for (i, &c) in text_chars.iter().enumerate() {
            let s = match states.get(i).copied().unwrap_or(CharState::Untyped) {
                CharState::Correct => Span::styled(c.to_string(), Style::default().fg(Color::Green)),
                CharState::Incorrect => Span::styled(c.to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::UNDERLINED)),
                CharState::Current => Span::styled(c.to_string(), Style::default().bg(Color::DarkGray).fg(Color::White)),
                CharState::Untyped => Span::styled(c.to_string(), Style::default().fg(Color::DarkGray)),
            };
            spans.push(s);
        }
        let line = Line::from(spans);
        let block = Block::default().borders(Borders::ALL).title("Type the given text!");
        let inner = block.inner(chunks[2]);
        frame.render_widget(block, chunks[2]);
        frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: false }), inner);
    } else {
        let msg = Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(msg, chunks[2]);
    }

    let hint = if app.is_waiting_for_multiplayer_start() {
        "Waiting for race to start"
    } else if app.is_multi() {
        "Race is in progress"
    } else {
        "Tab or Esc: restart"
    };
    frame.render_widget(Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)), chunks[3]);
}

fn draw_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let block = Block::default().borders(Borders::ALL).title("Results ");
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
            Line::from(Span::styled(winner_str, Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from(vec![Span::styled("You: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.0} WPM  {:.1}% acc", res.me.wpm, res.me.accuracy))]),
            Line::from(vec![Span::styled("Opponent: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.0} WPM  {:.1}% acc", res.opponent.wpm, res.opponent.accuracy))]),
            Line::from(""),
            Line::from("Tab or Enter: try again   Esc: quit"),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
    } else if let Some(ref r) = app.result() {
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled("WPM: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.0}", r.wpm))]),
            Line::from(vec![Span::styled("Raw WPM: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.0}", r.raw_wpm))]),
            Line::from(vec![Span::styled("Accuracy: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.1}%", r.accuracy))]),
            Line::from(vec![Span::styled("Consistency: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.0}%", r.consistency))]),
            Line::from(vec![Span::styled("Time: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.1}", r.duration_secs))]),
            Line::from(""),
            Line::from("Tab or Enter: try again    Esc: quit"),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
    } else {
        frame.render_widget(Paragraph::new("No results to display!"), inner);
    }
}
