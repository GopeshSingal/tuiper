mod app;
mod network;
mod typing;
mod ui;
mod words;

use app::{App, Screen};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

use protocols::{ClientMessage, ServerMessage};

const DEFAULT_WS_URL: &str = "ws://127.0.0.1:8080/ws";

fn ws_url() -> String {
    std::env::var("WS_URL").unwrap_or_else(|_| DEFAULT_WS_URL.to_string())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    main_tx_opt: &mut Option<mpsc::Sender<ServerMessage>>,
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
                        KeyCode::Char('s') => {
                            app.start_race(30);
                        }
                        KeyCode::Char('f') => {
                            let value = 30;
                            if let Some(ref tx) = app.ws_tx {
                                let _ = tx.send(ClientMessage::JoinQueue { value });
                            } else if let Some(main_tx) = main_tx_opt.take() {
                                let (app_tx, app_rx) = mpsc::channel();
                                app.ws_tx = Some(app_tx.clone());
                                network::run_ws_thread(ws_url(), main_tx, app_rx);
                                let _ = app_tx.send(ClientMessage::JoinQueue { value });
                            }
                        }
                        _ => {}
                    },
                    Screen::Queue => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if let Some(ref tx) = app.ws_tx {
                                let _ = tx.send(ClientMessage::LeaveQueue);
                            }
                            app.screen = Screen::Lobby;
                        }
                        _ => {}
                    },
                    Screen::Race => {
                        if let Some(ref mut t) = app.typing {
                            match key.code {
                                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.quit = true;
                                    break;
                                }
                                KeyCode::Backspace => {
                                    t.backspace();
                                }
                                KeyCode::Char(c) => {
                                    t.type_char(c);
                                }
                                KeyCode::Tab => {
                                    let value = t.value();
                                    app.start_race(value);
                                }
                                _ => {}
                            }
                        }
                    }
                    Screen::Results => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.quit = true;
                            break;
                        }
                        KeyCode::Tab | KeyCode::Enter => {
                            app.result = None;
                            app.race_results = None;
                            app.screen = Screen::Lobby;
                        }
                        _ => {}
                    },
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
    let mut main_tx_opt = Some(main_tx);

    let mut app = App::new();

    let _ = run_app(&mut terminal, &mut app, &mut main_tx_opt, &main_rx);

    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
