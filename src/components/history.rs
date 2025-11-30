use super::{Action, Component, Screen};
use crate::components::common::Popup;
use crate::db::Database;
use crate::models::Word;
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Margin, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::collections::HashMap;

/// Parse exchange field
fn parse_exchange(exchange: &str) -> HashMap<&str, String> {
    let mut result = HashMap::new();
    for part in exchange.split('/') {
        if let Some((key, value)) = part.split_once(':') {
            result.insert(key, value.to_string());
        }
    }
    result
}

/// Get exchange type name
fn exchange_type_name(key: &str) -> &str {
    match key {
        "p" => "过去式", "d" => "过去分词", "i" => "现在分词",
        "3" => "第三人称单数", "s" => "复数",
        "r" => "比较级", "t" => "最高级",
        "0" => "原型", "1" => "原型变换",
        _ => key,
    }
}

/// Parse pos field
fn parse_pos(pos: &str) -> String {
    let parts: Vec<&str> = pos.split('/').collect();
    let mut result = Vec::new();
    for part in parts {
        if let Some((pos_code, _weight)) = part.split_once(':') {
            let pos_name = match pos_code {
                "n" => "n. 名词", "v" => "v. 动词",
                "adj" | "a" | "j" => "adj. 形容词",
                "adv" | "ad" | "r" => "adv. 副词",
                "prep" => "prep. 介词", "conj" | "c" => "conj. 连词",
                "pron" => "pron. 代词", "int" | "i" => "interj. 感叹词",
                "art" => "art. 冠词", "num" => "num. 数词",
                "aux" => "aux. 助动词",
                _ => continue,
            };
            result.push(pos_name);
        }
    }
    if result.is_empty() { String::new() } else { result.join(" / ") }
}

pub struct HistoryComponent {
    history_list: Vec<(Word, String, u8)>, // word, reviewed_at, quality
    selected_index: usize,
    show_popup: bool,
    popup: Popup,
}

impl HistoryComponent {
    pub fn new(db: Database) -> Result<Self> {
        let history_list = db.get_recent_reviews(100)?;
        Ok(Self {
            history_list,
            selected_index: 0,
            show_popup: false,
            popup: Popup::new("历史记录详情".to_string()),
        })
    }

    /// 生成历史记录详情内容
    fn build_history_detail<'a>(&self, word: &'a Word, reviewed_at: &str, quality: u8) -> Vec<Line<'a>> {
        let mut lines = vec![];

        // 复习时间和评分
        let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reviewed_at) {
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            reviewed_at.to_string()
        };

        let quality_style = match quality {
            1 => Theme::text_accent(),
            2 => Theme::text_warning(),
            3 => Theme::text_success(),
            4 => Theme::text_info(),
            _ => Theme::text_secondary(),
        };

        let quality_text = match quality {
            1 => "Forgot (忘记)",
            2 => "Hard (困难)",
            3 => "Good (良好)",
            4 => "Easy (简单)",
            _ => "Unknown",
        };

        lines.push(Line::from(vec![
            Span::styled("复习时间: ", Theme::text_secondary()),
            Span::styled(time_str, Theme::text_title()),
            Span::raw("  |  "),
            Span::styled("评分: ", Theme::text_secondary()),
            Span::styled(quality_text, quality_style),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Theme::text_secondary(),
        )));
        lines.push(Line::from(""));

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
                    meta_spans.push(Span::styled(pos_display, Theme::text_warning()));
                }
            }
        }
        if word.collins > 0 {
            if !meta_spans.is_empty() { meta_spans.push(Span::raw("  |  ")); }
            meta_spans.push(Span::styled(
                format!("柯林斯 {}", "★".repeat(word.collins as usize)),
                Theme::text_info(),
            ));
        }
        if word.oxford {
            if !meta_spans.is_empty() { meta_spans.push(Span::raw("  |  ")); }
            meta_spans.push(Span::styled(
                "牛津3000",
                Theme::text_success(),
            ));
        }
        if !meta_spans.is_empty() {
            lines.push(Line::from(meta_spans));
            lines.push(Line::from(""));
        }

        // Tags
        if let Some(tag) = &word.tag {
            if !tag.is_empty() {
                let tags: Vec<&str> = tag.split_whitespace().collect();
                let tag_display: Vec<String> = tags.iter().map(|t| {
                    match *t {
                        "zk" => "中考", "gk" => "高考", "cet4" => "CET-4", "cet6" => "CET-6",
                        "ky" => "考研", "toefl" => "TOEFL", "ielts" => "IELTS", "gre" => "GRE",
                        _ => t,
                    }.to_string()
                }).collect();
                lines.push(Line::from(vec![
                    Span::styled("考试: ", Theme::text_secondary()),
                    Span::styled(tag_display.join(" · "), Theme::text_info()),
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

        // Exchange
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

        // Frequency
        let mut freq_info = vec![];
        if let Some(bnc) = word.bnc { freq_info.push(format!("BNC: {}", bnc)); }
        if let Some(frq) = word.frq { freq_info.push(format!("当代: {}", frq)); }
        if !freq_info.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("词频: ", Theme::text_secondary()),
                Span::styled(
                    freq_info.join(" | "),
                    Theme::text_secondary().add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        lines
    }
}

impl Component for HistoryComponent {
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
            // 正常模式的键位处理
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
                KeyCode::Enter => {
                    // 打开浮窗显示完整信息
                    self.show_popup = true;
                    self.popup.reset_scroll();
                    Ok(Action::None)
                }
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
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .history_list
            .iter()
            .map(|(word, reviewed_at, quality)| {
                let (quality_text, quality_color) = match quality {
                    1 => ("Forgot", Theme::ACCENT),
                    2 => ("Hard", Theme::WARNING),
                    3 => ("Good", Theme::SUCCESS),
                    4 => ("Easy", Theme::INFO),
                    _ => ("Unknown", Theme::SECONDARY),
                };

                let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reviewed_at) {
                    dt.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    reviewed_at.clone()
                };

                let mut content_spans = vec![
                    Span::styled(
                        format!("{:20}", word.spelling),
                        Theme::text_title(),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{:10}", quality_text),
                        Theme::text_normal().fg(quality_color),
                    ),
                    Span::raw(" | "),
                    Span::styled(time_str, Theme::text_secondary()),
                ];

                if let Some(translation) = &word.translation {
                    if let Some(first_meaning) = translation.split('\n').next() {
                        content_spans.push(Span::raw("\n  "));
                        content_spans.push(Span::styled(
                            first_meaning,
                            Theme::text_secondary(),
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
            .block(Theme::block_default().title(list_title))
            .highlight_style(
                Theme::text_success()
                    .add_modifier(Modifier::BOLD)
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

        // 渲染浮窗（如果打开）
        if self.show_popup {
            if let Some((word, reviewed_at, quality)) = self.history_list.get(self.selected_index) {
                let popup_lines = self.build_history_detail(word, reviewed_at, *quality);
                self.popup.render(frame, area, popup_lines);
            }
        }
    }
}
