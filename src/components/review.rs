use super::{Action, Component, Screen};
use crate::components::common::ProgressBar;
use crate::db::Database;
use crate::models::{LearningLog, Word};
use crate::sm2;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewState {
    Question,
    Answer,
}

pub struct ReviewComponent {
    db: Database,
    review_queue: Vec<(Word, LearningLog)>,
    current_item: Option<(Word, LearningLog)>,
    pub state: ReviewState,
    total_count: usize,
    completed_count: usize,
    scroll: u16, // Scroll position for definition text
}

impl ReviewComponent {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            review_queue: Vec::new(),
            current_item: None,
            state: ReviewState::Question,
            total_count: 0,
            completed_count: 0,
            scroll: 0,
        }
    }

    pub fn start_review(&mut self, mode: ReviewMode) -> Result<bool> {
        self.review_queue = match mode {
            ReviewMode::Due => self.db.get_due_reviews()?,
            ReviewMode::LearnNew => self.db.get_new_words_to_learn(20)?,
        };

        self.total_count = self.review_queue.len();
        self.completed_count = 0;

        if self.review_queue.is_empty() {
            return Ok(false);
        }

        self.next_card();
        Ok(true)
    }

    fn next_card(&mut self) {
        self.current_item = self.review_queue.pop();
        self.state = ReviewState::Question;
        self.scroll = 0; // Reset scroll for new card
    }

    fn show_answer(&mut self) {
        self.state = ReviewState::Answer;
        self.scroll = 0; // Reset scroll when showing answer
    }

    fn submit_review(&mut self, quality: u8) -> Result<()> {
        if let Some((word, mut log)) = self.current_item.take() {
            let word_id = word.id.unwrap();
            sm2::process_review(&mut log, quality);
            self.db.update_log(&log)?;
            self.db.add_review_history(word_id, quality, &log)?;

            self.completed_count += 1;
            self.next_card();
        }
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.current_item.is_none()
    }
}

pub enum ReviewMode {
    Due,
    LearnNew,
}

impl Component for ReviewComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match self.state {
            ReviewState::Question => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
                KeyCode::Char(' ') | KeyCode::Enter => {
                    self.show_answer();
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            },
            ReviewState::Answer => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll = self.scroll.saturating_add(1);
                    Ok(Action::None)
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll = self.scroll.saturating_sub(1);
                    Ok(Action::None)
                }
                KeyCode::Char('1') => {
                    self.submit_review(1)?;
                    if self.is_complete() {
                        Ok(Action::NavigateTo(Screen::Dashboard))
                    } else {
                        Ok(Action::None)
                    }
                }
                KeyCode::Char('2') => {
                    self.submit_review(2)?;
                    if self.is_complete() {
                        Ok(Action::NavigateTo(Screen::Dashboard))
                    } else {
                        Ok(Action::None)
                    }
                }
                KeyCode::Char('3') => {
                    self.submit_review(3)?;
                    if self.is_complete() {
                        Ok(Action::NavigateTo(Screen::Dashboard))
                    } else {
                        Ok(Action::None)
                    }
                }
                KeyCode::Char('4') => {
                    self.submit_review(4)?;
                    if self.is_complete() {
                        Ok(Action::NavigateTo(Screen::Dashboard))
                    } else {
                        Ok(Action::None)
                    }
                }
                _ => Ok(Action::None),
            },
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if let Some((word, _)) = &self.current_item {
            let block = Block::default().borders(Borders::ALL).title(" Review ");
            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),      // Progress bar
                    Constraint::Percentage(35), // Word
                    Constraint::Percentage(15), // Phonetic
                    Constraint::Percentage(50), // Definition
                ])
                .split(inner_area);

            // Progress bar
            let progress_bar = ProgressBar::new(self.completed_count, self.total_count)
                .with_label(format!(
                    "Progress: {}/{} (Remaining: {})",
                    self.completed_count,
                    self.total_count,
                    self.total_count - self.completed_count
                ))
                .with_color(Color::Cyan);
            progress_bar.render(frame, layout[0]);

            // Word
            let word_text = Paragraph::new(Span::styled(
                &word.spelling,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
            frame.render_widget(word_text, layout[1]);

            // Phonetic
            if let Some(phonetic) = &word.phonetic {
                let phonetic_text = Paragraph::new(format!("[ {} ]", phonetic))
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(phonetic_text, layout[2]);
            }

            // Definition
            match self.state {
                ReviewState::Question => {
                    let hint = Paragraph::new("Press <Space> to show definition")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(Style::default().fg(Color::Gray));
                    frame.render_widget(hint, layout[3]);
                }
                ReviewState::Answer => {
                    let mut def_lines = vec![
                        Line::from(Span::styled(
                            "English:",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(word.definition.as_str()),
                    ];

                    if let Some(translation) = &word.translation {
                        def_lines.push(Line::from(""));
                        def_lines.push(Line::from(Span::styled(
                            "中文:",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )));
                        def_lines.push(Line::from(translation.as_str()));
                    }

                    // Calculate content height for scrollbar
                    let content_height = def_lines.len() as u16;

                    let def_text = Paragraph::new(def_lines)
                        .wrap(Wrap { trim: true })
                        .alignment(ratatui::layout::Alignment::Left)
                        .scroll((self.scroll, 0))
                        .block(Block::default().borders(Borders::TOP).title(" Definition (↑/↓ or j/k to scroll) "));
                    frame.render_widget(def_text, layout[3]);

                    // Render scrollbar if content is longer than visible area
                    if content_height > layout[3].height {
                        frame.render_stateful_widget(
                            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                                .begin_symbol(Some("↑"))
                                .end_symbol(Some("↓")),
                            layout[3].inner(Margin {
                                vertical: 1,
                                horizontal: 0,
                            }),
                            &mut ScrollbarState::new(content_height as usize)
                                .position(self.scroll as usize),
                        );
                    }
                }
            }
        } else {
            let msg = Paragraph::new("No words to review!")
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(msg, area);
        }
    }
}
