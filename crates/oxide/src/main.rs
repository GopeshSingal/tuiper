mod app;
mod typing;
mod ui;
mod words;

use app::{App, Screen};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
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

    let mut app = App::new();
    let _ = run_app(&mut terminal, &mut app);

    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
