use super::{Action, Component, Screen};
use crate::db::Database;
use crate::models::Word;
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct FavoritesComponent {
    db: Database,
    words: Vec<Word>,
    list_state: ListState,
    title: String,
}

impl FavoritesComponent {
    pub fn new(db: Database) -> Result<Self> {
        let words = db.get_favorites()?;
        let mut list_state = ListState::default();
        if !words.is_empty() {
            list_state.select(Some(0));
        }
        let title = format!(" ⭐ 收藏夹 ({} 个单词) ", words.len());

        Ok(Self {
            db,
            words,
            list_state,
            title,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.words = self.db.get_favorites()?;
        self.title = format!(" ⭐ 收藏夹 ({} 个单词) ", self.words.len());
        if self.words.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.words.len() {
                self.list_state.select(Some(self.words.len().saturating_sub(1)));
            }
        }
        Ok(())
    }

    fn next(&mut self) {
        if self.words.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.words.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.words.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.words.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_favorite(&mut self) -> Result<()> {
        if let Some(idx) = self.list_state.selected() {
            if let Some(word) = self.words.get(idx) {
                if let Some(word_id) = word.id {
                    self.db.toggle_favorite(word_id)?;
                    self.refresh()?;
                }
            }
        }
        Ok(())
    }
}

impl Component for FavoritesComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
                Ok(Action::None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
                Ok(Action::None)
            }
            KeyCode::Char('f') | KeyCode::Char('u') => {
                self.toggle_favorite()?;
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),     // List
                Constraint::Length(10), // Detail
            ])
            .margin(1)
            .split(area);

        if self.words.is_empty() {
            let msg = Paragraph::new(vec![
                Line::from(""),
                Line::from("暂无收藏单词"),
                Line::from(""),
                Line::from(vec![
                    Span::raw("在 Review、Dictionary、History 界面按 "),
                    Span::styled("'f'", Theme::text_warning()),
                    Span::raw(" 收藏单词"),
                ]),
            ])
            .alignment(ratatui::layout::Alignment::Center)
            .block(Theme::block_with_title(" ⭐ 收藏夹 "))
            .style(Theme::text_secondary());
            frame.render_widget(msg, area);
            return;
        }

        // Word list
        let items: Vec<ListItem> = self
            .words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let mut spans = vec![
                    Span::styled(
                        format!("{:3}. ", i + 1),
                        Theme::text_secondary(),
                    ),
                    Span::styled(&word.spelling, Theme::text_title()),
                ];

                if let Some(phonetic) = &word.phonetic {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        format!("[{}]", phonetic),
                        Theme::text_secondary(),
                    ));
                }

                if let Some(translation) = &word.translation {
                    let short_trans = translation
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(40)
                        .collect::<String>();
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(short_trans, Theme::text_normal()));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(
                Theme::block_default()
                    .title(self.title.as_str())
                    .title_style(Theme::text_title())
            )
            .highlight_style(
                Theme::text_title()
                    .bg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Word detail
        if let Some(idx) = self.list_state.selected() {
            if let Some(word) = self.words.get(idx) {
                let mut detail_lines = vec![
                    Line::from(vec![
                        Span::styled(&word.spelling, Theme::text_title()),
                    ]),
                ];

                if let Some(phonetic) = &word.phonetic {
                    detail_lines.push(Line::from(vec![
                        Span::styled(format!("[{}]", phonetic), Theme::text_secondary()),
                    ]));
                }

                detail_lines.push(Line::from(""));

                if let Some(translation) = &word.translation {
                    for line in translation.lines().take(5) {
                        detail_lines.push(Line::from(line));
                    }
                }

                let detail = Paragraph::new(detail_lines)
                    .block(Theme::block_accent_with_title(" 详情 "))
                    .style(Theme::text_normal());
                frame.render_widget(detail, chunks[1]);
            }
        }
    }
}
