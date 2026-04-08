mod app;
mod network;
mod theme;
mod typing;
mod ui;
mod words;

use app::{App, Screen};
use theme::{ThemeEditColumn, ThemeField};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

use protocols::{ClientMessage, ServerMessage};
use strum::IntoEnumIterator;

const DEFAULT_WS_URL: &str = "ws://127.0.0.1:8080/ws";

fn ws_url() -> String {
    std::env::var("WS_URL").unwrap_or_else(|_| DEFAULT_WS_URL.to_string())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    main_tx: &mpsc::Sender<ServerMessage>,
    main_rx: &mpsc::Receiver<ServerMessage>,
) -> io::Result<()> {
    loop {
        while let Ok(msg) = main_rx.try_recv() {
            app.handle_server_message(msg);
        }

        terminal.draw(|f| ui::draw(f, app))?;
        app.tick();

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.screen {
                    Screen::Lobby => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.quit = true;
                            break;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            app.start_race(30);
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            let value = 30;
                            if let Some(ref tx) = app.ws_tx {
                                let _ = tx.send(ClientMessage::JoinQueue { value });
                            } else {
                                let (app_tx, app_rx) = mpsc::channel();
                                app.ws_tx = Some(app_tx.clone());
                                network::run_ws_thread(ws_url(), main_tx.clone(), app_rx);
                                let _ = app_tx.send(ClientMessage::JoinQueue { value });
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            app.theme_edit_row = 0;
                            app.theme_edit_col = ThemeEditColumn::default();
                            app.screen = Screen::Config;
                        }
                        _ => {}
                    },
                    Screen::Queue => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if let Some(ref tx) = app.ws_tx {
                                let _ = tx.send(ClientMessage::LeaveQueue);
                            }
                            if key.code == KeyCode::Esc {
                                app.quit = true;
                                break;
                            }
                            app.disconnect_websocket();
                            app.screen = Screen::Lobby;
                        }
                        _ => {}
                    },
                    Screen::Race => {
                        let waiting_for_start = app.is_waiting_for_multiplayer_start();
                        let is_multi = app.is_multi();
                        if let Some(ref mut t) = app.typing {
                            match key.code {
                                KeyCode::Esc => {
                                    app.quit = true;
                                    break;
                                }
                                KeyCode::Backspace => {
                                    if !waiting_for_start {
                                        t.backspace();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if !waiting_for_start {
                                        t.type_char(c);
                                    }
                                }
                                KeyCode::Tab | KeyCode::Enter => {
                                    if !is_multi {
                                        let value = t.value();
                                        app.start_race(value);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Screen::Results => match key.code {
                        KeyCode::Char('q') => {
                            app.result = None;
                            app.race_results = None;
                            app.disconnect_websocket();
                            app.screen = Screen::Lobby;
                        }
                        KeyCode::Esc => {
                            app.quit = true;
                            break;
                        }
                        KeyCode::Tab | KeyCode::Enter => {
                            let value = 30;
                            if app.race_results.is_some() {
                                if let Some(ref tx) = app.ws_tx {
                                    let _ = tx.send(ClientMessage::JoinQueue { value });
                                } else {
                                    let (app_tx, app_rx) = mpsc::channel();
                                    app.ws_tx = Some(app_tx.clone());
                                    network::run_ws_thread(ws_url(), main_tx.clone(), app_rx);
                                    let _ = app_tx.send(ClientMessage::JoinQueue { value });
                                }
                            } else {
                                app.start_race(value);
                            }
                        }
                        _ => {}
                    },
                    Screen::Config => {
                        let fields: Vec<ThemeField> = ThemeField::iter().collect();
                        let n = fields.len();
                        match key.code {
                            KeyCode::Char('q') => {
                                let _ = theme::save(&app.theme);
                                app.screen = Screen::Lobby;
                            }
                            KeyCode::Up => {
                                if app.theme_edit_row > 0 {
                                    app.theme_edit_row -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.theme_edit_row + 1 < n {
                                    app.theme_edit_row += 1;
                                }
                            }
                            KeyCode::Left => {
                                app.theme_edit_col = app.theme_edit_col.next_left();
                            }
                            KeyCode::Right => {
                                app.theme_edit_col = app.theme_edit_col.next_right();
                            }
                            KeyCode::Tab => {
                                let field = fields[app.theme_edit_row];
                                match app.theme_edit_col {
                                    ThemeEditColumn::Palette => {
                                        app.theme.cycle_palette(field, 1);
                                    }
                                    ThemeEditColumn::Shade => {
                                        app.theme.cycle_shade(field, 1);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.quit = true;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (main_tx, main_rx) = mpsc::channel();

    let mut app = App::new();

    let _ = run_app(&mut terminal, &mut app, &main_tx, &main_rx);

    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
