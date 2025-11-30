mod app;
mod app_v2;
mod components;
mod db;
mod event;
mod models;
mod sm2;
mod theme;
mod tui;
mod ui;

use anyhow::Result;
use app::{App, CurrentScreen, ReviewState};
use clap::Parser;
use crossterm::event::{KeyCode, KeyEventKind};
use db::Database;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Use the new component-based architecture (default)
    #[arg(long, default_value_t = true)]
    v2: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let db = Database::initialize()?;

    // Initialize TUI
    let mut terminal = tui::init()?;
    let event_handler = event::EventHandler::new(Duration::from_millis(10));

    // Choose architecture version
    if args.v2 {
        return run_v2(terminal, db, event_handler);
    }

    let mut app = App::new(db);

    // Main Loop
    while !matches!(app.current_screen, CurrentScreen::Exiting) {
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        if let Some(event) = event_handler.next()? {
            match event {
                event::AppEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match app.current_screen {
                            CurrentScreen::Dashboard => match key.code {
                                KeyCode::Char('q') => app.current_screen = CurrentScreen::Exiting,
                                KeyCode::Char('r') => app.start_review()?,
                                KeyCode::Char('n') => app.start_learn_new()?,
                                KeyCode::Char('d') => {
                                    app.dismiss_completion_message();
                                    app.enter_dictionary()?;
                                }
                                KeyCode::Char('h') => {
                                    app.dismiss_completion_message();
                                    app.enter_history()?;
                                }
                                KeyCode::Char('s') => {
                                    app.dismiss_completion_message();
                                    app.enter_statistics()?;
                                }
                                KeyCode::Esc => app.dismiss_completion_message(),
                                _ => {}
                            },
                            CurrentScreen::Review => match app.review_state {
                                ReviewState::Question => match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        app.refresh_stats();
                                        app.current_screen = CurrentScreen::Dashboard;
                                    }
                                    KeyCode::Char(' ') | KeyCode::Enter => app.show_answer(),
                                    _ => {}
                                },
                                ReviewState::Answer => match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        app.refresh_stats();
                                        app.current_screen = CurrentScreen::Dashboard;
                                    }
                                    KeyCode::Char('1') => app.submit_review(1)?, // Forgot
                                    KeyCode::Char('2') => app.submit_review(2)?, // Hard
                                    KeyCode::Char('3') => app.submit_review(3)?, // Good
                                    KeyCode::Char('4') => app.submit_review(4)?, // Easy
                                    _ => {}
                                },
                            },
                            CurrentScreen::Dictionary => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.refresh_stats();
                                    app.current_screen = CurrentScreen::Dashboard;
                                }
                                KeyCode::Up | KeyCode::Char('k') => app.dict_select_previous(),
                                KeyCode::Down | KeyCode::Char('j') => app.dict_select_next(),
                                KeyCode::Char(c) => {
                                    app.dict_search_input.push(c);
                                    app.dict_update_search()?;
                                }
                                KeyCode::Backspace => {
                                    app.dict_search_input.pop();
                                    app.dict_update_search()?;
                                }
                                _ => {}
                            },
                            CurrentScreen::History => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.refresh_stats();
                                    app.current_screen = CurrentScreen::Dashboard;
                                }
                                _ => {}
                            },
                            CurrentScreen::Statistics => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.refresh_stats();
                                    app.current_screen = CurrentScreen::Dashboard;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                event::AppEvent::Tick => {
                    // Handle periodic updates if needed
                }
            }
        }
    }

    tui::restore()?;
    Ok(())
}

fn run_v2(
    mut terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    db: Database,
    event_handler: event::EventHandler,
) -> Result<()> {
    let mut app = app_v2::AppV2::new(db)?;

    loop {
        terminal.draw(|frame| app.render(frame))?;

        if let Some(event) = event_handler.next()? {
            match event {
                event::AppEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        if app.handle_key(key)? {
                            break;
                        }
                    }
                }
                event::AppEvent::Tick => {
                    // Handle periodic updates if needed
                }
            }
        }
    }

    tui::restore()?;
    Ok(())
}

