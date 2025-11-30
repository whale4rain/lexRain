use super::{Action, Component, Screen};
use crate::components::common::ProgressBar;
use crate::db::Database;
use crate::models::{LearningLog, Word};
use crate::sm2;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

/// Parse exchange field into a readable format
fn parse_exchange(exchange: &str) -> HashMap<&str, String> {
    let mut result = HashMap::new();
    for part in exchange.split('/') {
        if let Some((key, value)) = part.split_once(':') {
            result.insert(key, value.to_string());
        }
    }
    result
}

/// Get exchange type description
fn exchange_type_name(key: &str) -> &str {
    match key {
        "p" => "过去式",
        "d" => "过去分词",
        "i" => "现在分词",
        "3" => "第三人称单数",
        "r" => "比较级",
        "t" => "最高级",
        "s" => "复数",
        "0" => "原型",
        "1" => "原型变换",
        _ => key,
    }
}

/// Parse pos field: "v:100/n:50" -> "动词/名词"
fn parse_pos(pos: &str) -> String {
    let parts: Vec<&str> = pos.split('/').collect();
    let mut result = Vec::new();
    
    for part in parts {
        if let Some((pos_code, _weight)) = part.split_once(':') {
            let pos_name = match pos_code {
                "n" => "n. 名词",
                "v" => "v. 动词",
                "adj" | "a" | "j" => "adj. 形容词",
                "adv" | "ad" | "r" => "adv. 副词",
                "prep" => "prep. 介词",
                "conj" | "c" => "conj. 连词",
                "pron" => "pron. 代词",
                "int" | "i" => "interj. 感叹词",
                "art" => "art. 冠词",
                "num" => "num. 数词",
                "aux" => "aux. 助动词",
                _ => continue,
            };
            result.push(pos_name);
        }
    }
    
    if result.is_empty() {
        String::new()
    } else {
        result.join(" / ")
    }
}
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
                    Constraint::Length(5),      // Word + Phonetic + Metadata
                    Constraint::Min(10),        // Definition (scrollable)
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

            // Word Header (Word + Phonetic + Metadata in one compact area)
            let mut header_lines = vec![];
            
            // Line 1: Word + Phonetic
            let mut word_line_spans = vec![
                Span::styled(
                    &word.spelling,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
            ];
            if let Some(phonetic) = &word.phonetic {
                word_line_spans.push(Span::raw("  "));
                word_line_spans.push(Span::styled(
                    format!("[ {} ]", phonetic),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            header_lines.push(Line::from(word_line_spans));
            
            // Line 2: POS + Collins + Oxford
            let mut meta_spans = vec![];
            if let Some(pos) = &word.pos {
                if !pos.is_empty() {
                    let pos_display = parse_pos(pos);
                    if !pos_display.is_empty() {
                        meta_spans.push(Span::styled(
                            pos_display,
                            Style::default().fg(Color::Yellow),
                        ));
                    }
                }
            }
            if word.collins > 0 {
                if !meta_spans.is_empty() {
                    meta_spans.push(Span::raw("  |  "));
                }
                meta_spans.push(Span::styled(
                    format!("柯林斯 {}", "★".repeat(word.collins as usize)),
                    Style::default().fg(Color::Magenta),
                ));
            }
            if word.oxford {
                if !meta_spans.is_empty() {
                    meta_spans.push(Span::raw("  |  "));
                }
                meta_spans.push(Span::styled(
                    "牛津3000",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
            }
            if !meta_spans.is_empty() {
                header_lines.push(Line::from(meta_spans));
            }
            
            // Line 3: Tags
            if let Some(tag) = &word.tag {
                if !tag.is_empty() {
                    let tags: Vec<&str> = tag.split_whitespace().collect();
                    let tag_display: Vec<String> = tags.iter().map(|t| {
                        match *t {
                            "zk" => "中考",
                            "gk" => "高考",
                            "cet4" => "CET-4",
                            "cet6" => "CET-6",
                            "ky" => "考研",
                            "toefl" => "TOEFL",
                            "ielts" => "IELTS",
                            "gre" => "GRE",
                            _ => t,
                        }.to_string()
                    }).collect();
                    header_lines.push(Line::from(vec![
                        Span::styled(
                            "考试: ",
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            tag_display.join(" · "),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));
                }
            }
            
            let header = Paragraph::new(header_lines)
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            frame.render_widget(header, layout[1]);

            // Definition
            match self.state {
                ReviewState::Question => {
                    let hint = Paragraph::new("Press <Space> to show definition")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(Style::default().fg(Color::Gray));
                    frame.render_widget(hint, layout[2]);
                }
                ReviewState::Answer => {
                    // Split definition area into two columns: left for definitions, right for exchange
                    let def_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(70),  // Left: Definitions
                            Constraint::Percentage(30),  // Right: Exchange
                        ])
                        .split(layout[2]);
                    
                    // Left column: Chinese + English definitions
                    let mut left_lines = vec![];
                    
                    // Chinese Translation (top)
                    if let Some(translation) = &word.translation {
                        left_lines.push(Line::from(Span::styled(
                            "━━━ 中文释义 ━━━",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )));
                        
                        for line in translation.lines() {
                            if !line.trim().is_empty() {
                                left_lines.push(Line::from(format!("  {}", line)));
                            }
                        }
                        left_lines.push(Line::from(""));
                    }
                    
                    // English Definition (bottom)
                    left_lines.push(Line::from(Span::styled(
                        "━━━ English Definition ━━━",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )));
                    
                    for line in word.definition.lines() {
                        if !line.trim().is_empty() {
                            left_lines.push(Line::from(format!("  {}", line)));
                        }
                    }
                    
                    // Frequency info at bottom
                    let mut freq_info = vec![];
                    if let Some(bnc) = word.bnc {
                        freq_info.push(format!("BNC: {}", bnc));
                    }
                    if let Some(frq) = word.frq {
                        freq_info.push(format!("当代: {}", frq));
                    }
                    if !freq_info.is_empty() {
                        left_lines.push(Line::from(""));
                        left_lines.push(Line::from(vec![
                            Span::styled(
                                "词频: ",
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                freq_info.join(" | "),
                                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }

                    let left_content_height = left_lines.len() as u16;
                    let left_text = Paragraph::new(left_lines)
                        .wrap(Wrap { trim: true })
                        .alignment(ratatui::layout::Alignment::Left)
                        .scroll((self.scroll, 0))
                        .block(Block::default().borders(Borders::ALL).title(" 释义 (j/k: scroll) "));
                    frame.render_widget(left_text, def_layout[0]);

                    // Left scrollbar
                    if left_content_height > def_layout[0].height {
                        frame.render_stateful_widget(
                            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                                .begin_symbol(Some("↑"))
                                .end_symbol(Some("↓")),
                            def_layout[0].inner(Margin {
                                vertical: 1,
                                horizontal: 0,
                            }),
                            &mut ScrollbarState::new(left_content_height as usize)
                                .position(self.scroll as usize),
                        );
                    }
                    
                    // Right column: Exchange (词形变化)
                    let mut right_lines = vec![];
                    
                    if let Some(exchange) = &word.exchange {
                        if !exchange.is_empty() {
                            right_lines.push(Line::from(Span::styled(
                                "词形变化",
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            )));
                            right_lines.push(Line::from(""));
                            
                            let exchange_map = parse_exchange(exchange);
                            let order = ["0", "p", "d", "i", "3", "s", "r", "t", "1"];
                            
                            for key in &order {
                                if let Some(value) = exchange_map.get(*key) {
                                    right_lines.push(Line::from(Span::styled(
                                        exchange_type_name(key),
                                        Style::default().fg(Color::DarkGray),
                                    )));
                                    right_lines.push(Line::from(Span::styled(
                                        format!("  {}", value),
                                        Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                                    )));
                                    right_lines.push(Line::from(""));
                                }
                            }
                        } else {
                            right_lines.push(Line::from(Span::styled(
                                "无词形变化",
                                Style::default().fg(Color::DarkGray),
                            )));
                        }
                    } else {
                        right_lines.push(Line::from(Span::styled(
                            "无词形变化",
                            Style::default().fg(Color::DarkGray),
                        )));
                    }

                    let right_text = Paragraph::new(right_lines)
                        .wrap(Wrap { trim: true })
                        .alignment(ratatui::layout::Alignment::Left)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(right_text, def_layout[1]);
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
