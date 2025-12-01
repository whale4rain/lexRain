pub mod dashboard;
pub mod review;
pub mod dictionary;
pub mod history;
pub mod statistics;
pub mod wordbook;
pub mod favorites;
pub mod settings;
pub mod common;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Actions that components can trigger to affect the application state
#[derive(Debug, Clone)]
pub enum Action {
    NavigateTo(Screen),
    StartWordbookReview(String, bool), // (tag, shuffle)
    ToggleFavorite(i64), // word_id
    Quit,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Review,
    Dictionary,
    History,
    Statistics,
    Wordbook,
    Favorites,
    Settings,
}

/// Component trait for all UI components
pub trait Component {
    /// Handle keyboard input and return a message
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action>;

    /// Render the component
    fn view(&mut self, frame: &mut Frame, area: Rect);
}
