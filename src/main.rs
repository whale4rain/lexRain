mod app;
mod db;
mod event;
mod models;
mod sm2;
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
    /// Import words from a JSON file
    #[arg(short, long)]
    import: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let db = Database::initialize()?;

    // Handle import if requested
    if let Some(path) = args.import {
        println!("Importing from {}...", path);
        let content = std::fs::read_to_string(&path)?;
        let words: Vec<models::Word> = serde_json::from_str(&content)?;

        let mut imported_count = 0;
        for word in words {
            db.add_word(&word)?;
            // Get the word_id after insertion
            if let Some(word_id) = db.get_word_id(&word.spelling)? {
                db.init_learning_log(word_id)?;
                imported_count += 1;
            }
        }

        println!("Successfully imported {} words!", imported_count);
        return Ok(());
    }

    // Initialize TUI
    let mut terminal = tui::init()?;
    let mut app = App::new(db);
    let event_handler = event::EventHandler::new(Duration::from_millis(250));

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
                                KeyCode::Char('d') => app.current_screen = CurrentScreen::Dictionary,
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
                                KeyCode::Esc => app.current_screen = CurrentScreen::Dashboard,
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

