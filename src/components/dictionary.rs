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
use std::collections::HashMap;

const LIST_LIMIT: usize = 100;

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

pub struct DictionaryComponent {
    db: Database,
    search_input: SearchInput,
    word_list: Vec<(Word, Option<LearningLog>)>,
    selected_index: usize,
    table_state: TableState,
    detail_scroll: u16, // Scroll position for detail view
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
            detail_scroll: 0,
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
            self.detail_scroll = 0;
        }
    }

    fn select_previous(&mut self) {
        if !self.word_list.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.table_state.select(Some(self.selected_index % LIST_LIMIT));
            self.detail_scroll = 0;
        }
    }

    fn select_first(&mut self) {
        if !self.word_list.is_empty() {
            self.selected_index = 0;
            self.table_state.select(Some(0));
            self.detail_scroll = 0;
        }
    }

    fn select_last(&mut self) {
        if !self.word_list.is_empty() {
            self.selected_index = self.word_list.len() - 1;
            self.table_state.select(Some(self.selected_index % LIST_LIMIT));
            self.detail_scroll = 0;
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
            KeyCode::Left | KeyCode::Char('h') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
                Ok(Action::None)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
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
                Constraint::Length(20), // Selected word detail (increased from 8 to 20)
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
            let mut detail_lines = vec![];
            
            // Word + Phonetic
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
            detail_lines.push(Line::from(word_line_spans));
            detail_lines.push(Line::from(""));
            
            // POS + Collins + Oxford
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
                detail_lines.push(Line::from(meta_spans));
                detail_lines.push(Line::from(""));
            }
            
            // Tags (考试标签)
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
                    detail_lines.push(Line::from(vec![
                        Span::styled(
                            "考试: ",
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            tag_display.join(" · "),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));
                    detail_lines.push(Line::from(""));
                }
            }
            
            // Chinese Translation
            if let Some(translation) = &word.translation {
                detail_lines.push(Line::from(Span::styled(
                    "━━━ 中文释义 ━━━",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                for line in translation.lines() {
                    if !line.trim().is_empty() {
                        detail_lines.push(Line::from(format!("  {}", line)));
                    }
                }
                detail_lines.push(Line::from(""));
            }
            
            // English Definition
            detail_lines.push(Line::from(Span::styled(
                "━━━ English Definition ━━━",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for line in word.definition.lines() {
                if !line.trim().is_empty() {
                    detail_lines.push(Line::from(format!("  {}", line)));
                }
            }
            detail_lines.push(Line::from(""));
            
            // Exchange (词形变化)
            if let Some(exchange) = &word.exchange {
                if !exchange.is_empty() {
                    detail_lines.push(Line::from(Span::styled(
                        "━━━ 词形变化 ━━━",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    )));
                    
                    let exchange_map = parse_exchange(exchange);
                    let order = ["0", "p", "d", "i", "3", "s", "r", "t", "1"];
                    
                    for key in &order {
                        if let Some(value) = exchange_map.get(*key) {
                            detail_lines.push(Line::from(vec![
                                Span::styled(
                                    format!("  {} ", exchange_type_name(key)),
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::styled(
                                    value.clone(),
                                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                                ),
                            ]));
                        }
                    }
                    detail_lines.push(Line::from(""));
                }
            }
            
            // Frequency (词频)
            let mut freq_info = vec![];
            if let Some(bnc) = word.bnc {
                freq_info.push(format!("BNC: {}", bnc));
            }
            if let Some(frq) = word.frq {
                freq_info.push(format!("当代: {}", frq));
            }
            if !freq_info.is_empty() {
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        "词频: ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        freq_info.join(" | "),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ]));
                detail_lines.push(Line::from(""));
            }

            // Learning status
            if let Some(log) = log {
                detail_lines.push(Line::from(Span::styled(
                    "━━━ 学习状态 ━━━",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        "状态: ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:?}", log.status),
                        Style::default().fg(match log.status {
                            LearningStatus::New => Color::Gray,
                            LearningStatus::Learning => Color::Yellow,
                            LearningStatus::Mastered => Color::Green,
                        }),
                    ),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        format!("复习次数: {} | 间隔: {} 天 | 记忆因子: {:.2}", 
                            log.repetition, log.interval, log.e_factor),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            let detail_content_height = detail_lines.len() as u16;
            let detail = Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Detail (h/l: scroll) ")
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.detail_scroll, 0));
            frame.render_widget(detail, layout[2]);
            
            // Detail scrollbar
            if detail_content_height > layout[2].height {
                frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓")),
                    layout[2].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut ScrollbarState::new(detail_content_height as usize)
                        .position(self.detail_scroll as usize),
                );
            }
        }
    }
}
