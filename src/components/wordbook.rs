use super::{Action, Component, Screen};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub struct WordbookComponent {
    wordbooks: Vec<(String, usize)>, // (tag, count)
    selected_index: usize,
    shuffle_mode: bool,
}

impl WordbookComponent {
    pub fn new(db: Database) -> Result<Self> {
        let wordbooks = db.get_wordbooks()?;
        Ok(Self {
            wordbooks,
            selected_index: 0,
            shuffle_mode: false,
        })
    }

    fn toggle_shuffle(&mut self) {
        self.shuffle_mode = !self.shuffle_mode;
    }

    fn select_wordbook(&self) -> Result<Action> {
        if let Some((tag, _count)) = self.wordbooks.get(self.selected_index) {
            // 返回 Action，携带 tag 和 shuffle 信息
            // 这里需要在 Action 枚举中添加新的变体
            Ok(Action::StartWordbookReview(tag.clone(), self.shuffle_mode))
        } else {
            Ok(Action::None)
        }
    }
}

impl Component for WordbookComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Enter => self.select_wordbook(),
            KeyCode::Char('s') => {
                self.toggle_shuffle();
                Ok(Action::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.wordbooks.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Ok(Action::None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                Ok(Action::None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected_index = 0;
                Ok(Action::None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.selected_index = self.wordbooks.len().saturating_sub(1);
                Ok(Action::None)
            }
            KeyCode::PageDown => {
                self.selected_index = (self.selected_index + 10).min(self.wordbooks.len().saturating_sub(1));
                Ok(Action::None)
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(10);
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),     // Wordbook list
                Constraint::Length(5),   // Help text
            ])
            .split(area);

        // 单词本列表
        let items: Vec<ListItem> = self
            .wordbooks
            .iter()
            .map(|(tag, count)| {
                // 解析 tag 并显示中文名称
                let tag_display = tag.split_whitespace()
                    .map(|t| match t {
                        "zk" => "中考",
                        "gk" => "高考",
                        "cet4" => "CET-4",
                        "cet6" => "CET-6",
                        "ky" => "考研",
                        "toefl" => "TOEFL",
                        "ielts" => "IELTS",
                        "gre" => "GRE",
                        _ => t,
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");

                let content = vec![
                    Span::styled(
                        format!("  {}", tag_display),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("({} 词)", count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ];

                ListItem::new(Line::from(content))
            })
            .collect();

        let list_title = format!(
            " 选择单词本 ({}/{}) - {} ",
            self.selected_index + 1,
            self.wordbooks.len(),
            if self.shuffle_mode { "🔀 乱序" } else { "📚 顺序" }
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

        frame.render_stateful_widget(list, layout[0], &mut list_state);

        // Scrollbar
        if !self.wordbooks.is_empty() {
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                layout[0].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut ScrollbarState::new(self.wordbooks.len())
                    .position(self.selected_index),
            );
        }

        // Help text
        let help_lines = vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" 开始复习  "),
                Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" 切换乱序/顺序  "),
                Span::styled("↑/↓ j/k", Style::default().fg(Color::Cyan)),
                Span::raw(" 选择"),
            ]),
            Line::from(vec![
                Span::styled("g/G", Style::default().fg(Color::Cyan)),
                Span::raw(" 首/尾  "),
                Span::styled("PageUp/Down", Style::default().fg(Color::Cyan)),
                Span::raw(" 翻页  "),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" 返回"),
            ]),
        ];

        let help = Paragraph::new(help_lines)
            .block(Block::default().borders(Borders::ALL).title(" 操作提示 "))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, layout[1]);
    }
}
