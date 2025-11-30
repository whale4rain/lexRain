use super::{Action, Component, Screen};
use crate::components::common::SearchInput;
use crate::db::Database;
use crate::models::{LearningLog, LearningStatus, Word};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Wrap,
    },
    Frame,
};

const LIST_LIMIT: usize = 100;

pub struct DictionaryComponent {
    db: Database,
    search_input: SearchInput,
    word_list: Vec<(Word, Option<LearningLog>)>,
    selected_index: usize,
    table_state: TableState,
}

impl DictionaryComponent {
    pub fn new(db: Database) -> Result<Self> {
        let word_list = db.get_all_words()?;
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Ok(Self {
            db,
            search_input: SearchInput::new().with_placeholder("Type to search...".to_string()),
            word_list,
            selected_index: 0,
            table_state,
        })
    }

    fn update_search(&mut self) -> Result<()> {
        if self.search_input.value.is_empty() {
            self.word_list = self.db.get_all_words()?;
        } else {
            self.word_list = self.db.search_words(&self.search_input.value)?;
        }
        self.selected_index = 0;
        Ok(())
    }

    fn select_next(&mut self) {
        if !self.word_list.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.word_list.len() - 1);
            self.table_state.select(Some(self.selected_index % LIST_LIMIT));
        }
    }

    fn select_previous(&mut self) {
        if !self.word_list.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.table_state.select(Some(self.selected_index % LIST_LIMIT));
        }
    }

    fn select_first(&mut self) {
        if !self.word_list.is_empty() {
            self.selected_index = 0;
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.word_list.is_empty() {
            self.selected_index = self.word_list.len() - 1;
            self.table_state.select(Some(self.selected_index % LIST_LIMIT));
        }
    }
}

impl Component for DictionaryComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                Ok(Action::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(Action::None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
                Ok(Action::None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.select_previous();
                }
                Ok(Action::None)
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.select_next();
                }
                Ok(Action::None)
            }
            KeyCode::Char(_c) => {
                self.search_input.handle_key(key);
                self.update_search()?;
                Ok(Action::None)
            }
            KeyCode::Backspace => {
                self.search_input.handle_key(key);
                self.update_search()?;
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Block::default().borders(Borders::ALL), area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search input
                Constraint::Min(10),    // Word table
                Constraint::Length(8),  // Selected word detail
            ])
            .margin(1)
            .split(area);

        // Search input
        self.search_input.render(frame, layout[0]);

        // Word table with scrollbar
        let page = self.selected_index / LIST_LIMIT;
        let items_len = self.word_list.len();

        let rows: Vec<Row> = self
            .word_list
            .iter()
            .skip(page * LIST_LIMIT)
            .take(LIST_LIMIT)
            .map(|(word, log)| {
                let status_symbol = if let Some(log) = log {
                    match log.status {
                        LearningStatus::New => "◯",
                        LearningStatus::Learning => "◐",
                        LearningStatus::Mastered => "●",
                    }
                } else {
                    "◯"
                };

                let status_color = if let Some(log) = log {
                    match log.status {
                        LearningStatus::New => Color::Gray,
                        LearningStatus::Learning => Color::Yellow,
                        LearningStatus::Mastered => Color::Green,
                    }
                } else {
                    Color::Gray
                };

                let phonetic = word
                    .phonetic
                    .as_ref()
                    .map(|p| format!("[{}]", p))
                    .unwrap_or_default();

                let interval = if let Some(log) = log {
                    format!("{} days", log.interval)
                } else {
                    "-".to_string()
                };

                Row::new(vec![
                    Cell::from(Span::styled(status_symbol, Style::default().fg(status_color))),
                    Cell::from(Span::styled(&word.spelling, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                    Cell::from(Span::styled(phonetic, Style::default().fg(Color::DarkGray))),
                    Cell::from(interval),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),  // Status
                Constraint::Length(20), // Word
                Constraint::Length(20), // Phonetic
                Constraint::Min(10),    // Interval
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(Span::styled("", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Word", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Phonetic", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Interval", Style::default().add_modifier(Modifier::BOLD))),
            ])
            .style(Style::default().fg(Color::Yellow))
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Dictionary ({} words) ", items_len))
                .title_bottom(
                    if items_len > 0 {
                        Line::from(vec![
                            Span::raw("|"),
                            Span::styled(
                                format!(" {}/{} ", self.selected_index + 1, items_len),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("|"),
                        ])
                        .right_aligned()
                    } else {
                        Line::default()
                    }
                ),
        )
        .row_highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(table, layout[1], &mut self.table_state);

        // Scrollbar
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            layout[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut ScrollbarState::new(items_len).position(self.selected_index),
        );

        // Selected word detail
        if let Some((word, log)) = self.word_list.get(self.selected_index) {
            let mut detail_lines = vec![
                Line::from(vec![
                    Span::styled("Word: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&word.spelling, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Definition: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&word.definition),
                ]),
            ];

            if let Some(translation) = &word.translation {
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(vec![
                    Span::styled("中文: ", Style::default().fg(Color::Yellow)),
                    Span::raw(translation),
                ]));
            }

            // Show ECDICT metadata
            let mut meta_parts = Vec::new();
            if word.collins > 0 {
                meta_parts.push(format!("Collins {}", "★".repeat(word.collins as usize)));
            }
            if word.oxford {
                meta_parts.push("Oxford 3000".to_string());
            }
            if let Some(tag) = &word.tag {
                if !tag.is_empty() {
                    meta_parts.push(tag.replace(" ", ", ").to_uppercase());
                }
            }
            if !meta_parts.is_empty() {
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(Color::Magenta)),
                    Span::styled(meta_parts.join(" | "), Style::default().fg(Color::Cyan)),
                ]));
            }

            if let Some(log) = log {
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(vec![
                    Span::styled("Learning: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:?}", log.status),
                        Style::default().fg(match log.status {
                            LearningStatus::New => Color::Gray,
                            LearningStatus::Learning => Color::Yellow,
                            LearningStatus::Mastered => Color::Green,
                        }),
                    ),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("Rep: {} | Interval: {} days | EF: {:.2}", log.repetition, log.interval, log.e_factor),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            let detail = Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Detail ")
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(detail, layout[2]);
        }
    }
}
