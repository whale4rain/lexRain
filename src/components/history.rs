use super::{Action, Component, Screen};
use crate::db::Database;
use crate::models::Word;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct HistoryComponent {
    history_list: Vec<(Word, String, u8)>, // word, reviewed_at, quality
}

impl HistoryComponent {
    pub fn new(db: Database) -> Result<Self> {
        let history_list = db.get_recent_reviews(100)?;
        Ok(Self { history_list })
    }
}

impl Component for HistoryComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
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

                if let Some(chinese_def) = &word.chinese_definition {
                    content_spans.push(Span::raw("\n  "));
                    content_spans.push(Span::styled(
                        chinese_def.as_str(),
                        Style::default().fg(Color::Gray),
                    ));
                }

                ListItem::new(Line::from(content_spans))
            })
            .collect();

        let list_title = format!(" Review History (Last {} reviews) ", self.history_list.len());
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(list_title));

        frame.render_widget(list, area);
    }
}
