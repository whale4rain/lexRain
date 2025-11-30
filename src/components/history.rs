use super::{Action, Component, Screen};
use crate::db::Database;
use crate::models::Word;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub struct HistoryComponent {
    history_list: Vec<(Word, String, u8)>, // word, reviewed_at, quality
    selected_index: usize,
}

impl HistoryComponent {
    pub fn new(db: Database) -> Result<Self> {
        let history_list = db.get_recent_reviews(100)?;
        Ok(Self {
            history_list,
            selected_index: 0,
        })
    }
}

impl Component for HistoryComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.history_list.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Ok(Action::None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                Ok(Action::None)
            }
            KeyCode::PageDown => {
                self.selected_index = (self.selected_index + 10).min(self.history_list.len().saturating_sub(1));
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(10);
                Ok(Action::None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected_index = 0;
                Ok(Action::None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.selected_index = self.history_list.len().saturating_sub(1);
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .history_list
            .iter()
            .map(|(word, reviewed_at, quality)| {
                let quality_text = match quality {
                    1 => ("Forgot", Color::Red),
                    2 => ("Hard", Color::Yellow),
                    3 => ("Good", Color::Green),
                    4 => ("Easy", Color::Cyan),
                    _ => ("Unknown", Color::Gray),
                };

                let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reviewed_at) {
                    dt.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    reviewed_at.clone()
                };

                let mut content_spans = vec![
                    Span::styled(
                        format!("{:20}", word.spelling),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{:10}", quality_text.0),
                        Style::default().fg(quality_text.1),
                    ),
                    Span::raw(" | "),
                    Span::styled(time_str, Style::default().fg(Color::DarkGray)),
                ];

                if let Some(translation) = &word.translation {
                    if let Some(first_meaning) = translation.split('\n').next() {
                        content_spans.push(Span::raw("\n  "));
                        content_spans.push(Span::styled(
                            first_meaning,
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }

                ListItem::new(Line::from(content_spans))
            })
            .collect();

        let list_title = format!(
            " Review History ({}/{}) - ↑/↓ or j/k to navigate ",
            self.selected_index + 1,
            self.history_list.len()
        );

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(list_title))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));

        frame.render_stateful_widget(list, area, &mut list_state);

        // Render scrollbar
        if !self.history_list.is_empty() {
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut ScrollbarState::new(self.history_list.len())
                    .position(self.selected_index),
            );
        }
    }
}
