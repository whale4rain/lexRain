use super::{Action, Component, Screen};
use crate::components::common::{SearchInput, Popup};
use crate::db::Database;
use crate::models::{LearningLog, LearningStatus, Word};
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{
        Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Wrap,
    },
    Frame,
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,  // Navigation mode (j/k works)
    Insert,  // Input mode (typing)
}

const LIST_LIMIT: usize = 30;

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
    show_popup: bool,   // Whether to show popup
    popup: Popup,       // Popup component
    mode: Mode,         // Current input mode
    searching: bool,    // Whether currently searching
    loading_frame: usize, // Loading animation frame
}

impl DictionaryComponent {
    pub fn new(db: Database) -> Result<Self> {
        let word_list = db.get_all_words()?;
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Ok(Self {
            db,
            search_input: SearchInput::new().with_placeholder("Press 'i' to search...".to_string()),
            word_list,
            selected_index: 0,
            table_state,
            detail_scroll: 0,
            show_popup: false,
            popup: Popup::new("单词详情".to_string()),
            mode: Mode::Normal,
            searching: false,
            loading_frame: 0,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        // Refresh the word list to update favorited status
        if self.search_input.value.is_empty() {
            self.word_list = self.db.get_all_words()?;
        } else {
            self.word_list = self.db.search_words(&self.search_input.value)?;
        }
        Ok(())
    }

    fn update_search(&mut self) -> Result<()> {
        self.searching = true;
        
        if self.search_input.value.is_empty() {
            self.word_list = self.db.get_all_words()?;
        } else {
            self.word_list = self.db.search_words(&self.search_input.value)?;
        }
        self.selected_index = 0;
        
        self.searching = false;
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

    /// 生成单词详情的内容行（用于浮窗和详情面板）
    fn build_detail_lines<'a>(&self, word: &'a Word, log: &Option<LearningLog>) -> Vec<Line<'a>> {
        let mut lines = vec![];
        
        // Word + Phonetic
        let mut word_line_spans = vec![
            Span::styled(
                &word.spelling,
                Theme::text_title()
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ];
        if let Some(phonetic) = &word.phonetic {
            word_line_spans.push(Span::raw("  "));
            word_line_spans.push(Span::styled(
                format!("[ {} ]", phonetic),
                Theme::text_secondary(),
            ));
        }
        lines.push(Line::from(word_line_spans));
        lines.push(Line::from(""));
        
        // POS + Collins + Oxford
        let mut meta_spans = vec![];
        if let Some(pos) = &word.pos {
            if !pos.is_empty() {
                let pos_display = parse_pos(pos);
                if !pos_display.is_empty() {
                    meta_spans.push(Span::styled(
                        pos_display,
                        Theme::text_warning(),
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
                Theme::text_info(),
            ));
        }
        if word.oxford {
            if !meta_spans.is_empty() {
                meta_spans.push(Span::raw("  |  "));
            }
            meta_spans.push(Span::styled(
                "牛津3000",
                Theme::text_success(),
            ));
        }
        if !meta_spans.is_empty() {
            lines.push(Line::from(meta_spans));
            lines.push(Line::from(""));
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
                lines.push(Line::from(vec![
                    Span::styled(
                        "考试: ",
                        Theme::text_secondary(),
                    ),
                    Span::styled(
                        tag_display.join(" · "),
                        Theme::text_info(),
                    ),
                ]));
                lines.push(Line::from(""));
            }
        }
        
        // Chinese Translation
        if let Some(translation) = &word.translation {
            lines.push(Line::from(Span::styled(
                "━━━ 中文释义 ━━━",
                Theme::text_title(),
            )));
            for line in translation.lines() {
                if !line.trim().is_empty() {
                    lines.push(Line::from(format!("  {}", line)));
                }
            }
            lines.push(Line::from(""));
        }
        
        // English Definition
        lines.push(Line::from(Span::styled(
            "━━━ English Definition ━━━",
            Theme::text_warning(),
        )));
        for line in word.definition.lines() {
            if !line.trim().is_empty() {
                lines.push(Line::from(format!("  {}", line)));
            }
        }
        lines.push(Line::from(""));
        
        // Exchange (词形变化)
        if let Some(exchange) = &word.exchange {
            if !exchange.is_empty() {
                lines.push(Line::from(Span::styled(
                    "━━━ 词形变化 ━━━",
                    Theme::text_accent(),
                )));
                
                let exchange_map = parse_exchange(exchange);
                let order = ["0", "p", "d", "i", "3", "s", "r", "t", "1"];
                
                for key in &order {
                    if let Some(value) = exchange_map.get(*key) {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {} ", exchange_type_name(key)),
                                Theme::text_secondary(),
                            ),
                            Span::styled(
                                value.clone(),
                                Theme::text_title().add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }
                }
                lines.push(Line::from(""));
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
            lines.push(Line::from(vec![
                Span::styled(
                    "词频: ",
                    Theme::text_secondary(),
                ),
                Span::styled(
                    freq_info.join(" | "),
                    Theme::text_secondary().add_modifier(Modifier::ITALIC),
                ),
            ]));
            lines.push(Line::from(""));
        }

        // Learning status
        if let Some(log) = log {
            lines.push(Line::from(Span::styled(
                "━━━ 学习状态 ━━━",
                Theme::text_success(),
            )));
            lines.push(Line::from(vec![
                Span::styled(
                    "状态: ",
                    Theme::text_secondary(),
                ),
                Span::styled(
                    format!("{:?}", log.status),
                    match log.status {
                        LearningStatus::New => Theme::text_secondary(),
                        LearningStatus::Learning => Theme::text_warning(),
                        LearningStatus::Mastered => Theme::text_success(),
                    },
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("复习次数: {} | 间隔: {} 天 | 记忆因子: {:.2}", 
                        log.repetition, log.interval, log.e_factor),
                    Theme::text_secondary(),
                ),
            ]));
        }

        lines
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Tab | KeyCode::Char('i') => {
                // Enter insert mode
                self.mode = Mode::Insert;
                Ok(Action::None)
            }
            KeyCode::Char('f') => {
                // Toggle favorite for selected word
                if let Some((word, _)) = self.word_list.get(self.selected_index) {
                    if let Some(word_id) = word.id {
                        return Ok(Action::ToggleFavorite(word_id));
                    }
                }
                Ok(Action::None)
            }
            KeyCode::Enter => {
                // Open popup for selected word
                if !self.word_list.is_empty() {
                    self.show_popup = true;
                    self.popup.reset_scroll();
                }
                Ok(Action::None)
            }
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
            _ => Ok(Action::None),
        }
    }
    
    fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Tab | KeyCode::Esc => {
                // Exit insert mode and clear search if empty
                self.mode = Mode::Normal;
                if self.search_input.value.is_empty() {
                    self.word_list = self.db.get_all_words()?;
                    self.selected_index = 0;
                }
                Ok(Action::None)
            }
            KeyCode::Enter => {
                // Perform search and exit to normal mode
                if !self.search_input.value.is_empty() {
                    self.update_search()?;
                    self.mode = Mode::Normal;
                }
                Ok(Action::None)
            }
            KeyCode::Char(_c) => {
                // Just update input, don't search immediately
                self.search_input.handle_key(key);
                Ok(Action::None)
            }
            KeyCode::Backspace => {
                // Just update input, don't search immediately
                self.search_input.handle_key(key);
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }
}

impl Component for DictionaryComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        // 如果浮窗打开，处理浮窗的键位
        if self.show_popup {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.show_popup = false;
                    self.popup.reset_scroll();
                    Ok(Action::None)
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.popup.scroll_down();
                    Ok(Action::None)
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.popup.scroll_up();
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            }
        } else {
            // Normal mode vs Insert mode
            match self.mode {
                Mode::Normal => self.handle_normal_mode(key),
                Mode::Insert => self.handle_insert_mode(key),
            }
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Update loading animation frame
        if self.searching {
            self.loading_frame = self.loading_frame.wrapping_add(1);
        }
        
        frame.render_widget(Theme::block_default(), area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search input
                Constraint::Min(10),    // Word table
                Constraint::Length(20), // Selected word detail (increased from 8 to 20)
            ])
            .margin(1)
            .split(area);

        // Search input with mode indicator
        let mode_indicator = match self.mode {
            Mode::Normal => "[Tab to open]",
            Mode::Insert => "[Enter to search]",
        };
        
        let loading_animation = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let search_title = if self.searching {
            let frame = loading_animation[self.loading_frame % loading_animation.len()];
            format!(" Search {} - {} Searching... ", mode_indicator, frame)
        } else {
            format!(" Search {} ", mode_indicator)
        };
        
        let search_block = if self.mode == Mode::Insert {
            Theme::block_warning().title(search_title.clone())
        } else {
            Theme::block_default().title(search_title.clone())
        };
        
        let search_widget = Paragraph::new(if self.search_input.value.is_empty() {
            if self.mode == Mode::Insert {
                "Type and press Enter to search..."
            } else {
                "Press Tab to open search..."
            }
        } else {
            &self.search_input.value
        })
        .block(search_block)
        .style(if self.search_input.value.is_empty() {
            Theme::text_secondary()
        } else {
            Theme::text_warning()
        });
        
        frame.render_widget(search_widget, layout[0]);

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
                        LearningStatus::New => Theme::SECONDARY,
                        LearningStatus::Learning => Theme::WARNING,
                        LearningStatus::Mastered => Theme::SUCCESS,
                    }
                } else {
                    Theme::SECONDARY
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
                    Cell::from(Span::styled(status_symbol, Theme::text_normal().fg(status_color))),
                    Cell::from(Span::styled(&word.spelling, Theme::text_title())),
                    Cell::from(Span::styled(phonetic, Theme::text_secondary())),
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
                Cell::from(Span::styled("", Theme::text_warning())),
                Cell::from(Span::styled("Word", Theme::text_warning())),
                Cell::from(Span::styled("Phonetic", Theme::text_warning())),
                Cell::from(Span::styled("Interval", Theme::text_warning())),
            ])
            .style(Theme::text_warning())
        )
        .block(
            Theme::block_default()
                .title(format!(" Dictionary ({} words) ", items_len))
                .title_bottom(
                    if items_len > 0 {
                        let help = match self.mode {
                            Mode::Normal => "Tab:Search | j/k:↑↓ | Enter:Detail | q:Quit",
                            Mode::Insert => "Tab:Exit | Enter:Search | Type to input",
                        };
                        Line::from(vec![
                            Span::raw("| "),
                            Span::styled(
                                format!("{}/{}", self.selected_index + 1, items_len),
                                Theme::text_title(),
                            ),
                            Span::raw(" | "),
                            Span::styled(help, Theme::text_secondary()),
                            Span::raw(" |"),
                        ])
                        .right_aligned()
                    } else {
                        Line::default()
                    }
                ),
        )
        .row_highlight_style(Theme::text_success());

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
                    Theme::text_title()
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ];
            if let Some(phonetic) = &word.phonetic {
                word_line_spans.push(Span::raw("  "));
                word_line_spans.push(Span::styled(
                    format!("[ {} ]", phonetic),
                    Theme::text_secondary(),
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
                            Theme::text_warning(),
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
                    Theme::text_info(),
                ));
            }
            if word.oxford {
                if !meta_spans.is_empty() {
                    meta_spans.push(Span::raw("  |  "));
                }
                meta_spans.push(Span::styled(
                    "牛津3000",
                    Theme::text_success(),
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
                            Theme::text_secondary(),
                        ),
                        Span::styled(
                            tag_display.join(" · "),
                            Theme::text_info(),
                        ),
                    ]));
                    detail_lines.push(Line::from(""));
                }
            }
            
            // Chinese Translation
            if let Some(translation) = &word.translation {
                detail_lines.push(Line::from(Span::styled(
                    "━━━ 中文释义 ━━━",
                    Theme::text_title(),
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
                Theme::text_warning(),
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
                        Theme::text_accent(),
                    )));
                    
                    let exchange_map = parse_exchange(exchange);
                    let order = ["0", "p", "d", "i", "3", "s", "r", "t", "1"];
                    
                    for key in &order {
                        if let Some(value) = exchange_map.get(*key) {
                            detail_lines.push(Line::from(vec![
                                Span::styled(
                                    format!("  {} ", exchange_type_name(key)),
                                    Theme::text_secondary(),
                                ),
                                Span::styled(
                                    value.clone(),
                                    Theme::text_title().add_modifier(Modifier::ITALIC),
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
                        Theme::text_secondary(),
                    ),
                    Span::styled(
                        freq_info.join(" | "),
                        Theme::text_secondary().add_modifier(Modifier::ITALIC),
                    ),
                ]));
                detail_lines.push(Line::from(""));
            }

            // Learning status
            if let Some(log) = log {
                detail_lines.push(Line::from(Span::styled(
                    "━━━ 学习状态 ━━━",
                    Theme::text_success(),
                )));
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        "状态: ",
                        Theme::text_secondary(),
                    ),
                    Span::styled(
                        format!("{:?}", log.status),
                        match log.status {
                            LearningStatus::New => Theme::text_secondary(),
                            LearningStatus::Learning => Theme::text_warning(),
                            LearningStatus::Mastered => Theme::text_success(),
                        },
                    ),
                ]));
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        format!("复习次数: {} | 间隔: {} 天 | 记忆因子: {:.2}", 
                            log.repetition, log.interval, log.e_factor),
                        Theme::text_secondary(),
                    ),
                ]));
            }

            let detail_content_height = detail_lines.len() as u16;
            let detail = Paragraph::new(detail_lines)
                .block(
                    Theme::block_default()
                        .title(" Detail (h/l: scroll) ")
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

        // 渲染浮窗（如果打开）
        if self.show_popup {
            if let Some((word, log)) = self.word_list.get(self.selected_index) {
                let popup_lines = self.build_detail_lines(word, log);
                self.popup.render(frame, area, popup_lines);
            }
        }
    }
}
