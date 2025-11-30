use super::{Action, Component, Screen};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub struct DashboardComponent {
    db: Database,
    stats: (i64, i64, i64), // total, mastered, due
    today_completed: i64,
    show_completion_message: bool,
}

impl DashboardComponent {
    pub fn new(db: Database) -> Self {
        let stats = db.get_stats().unwrap_or((0, 0, 0));
        let today_completed = db.get_today_completed_count().unwrap_or(0);

        Self {
            db,
            stats,
            today_completed,
            show_completion_message: false,
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self.db.get_stats().unwrap_or((0, 0, 0));
        self.today_completed = self.db.get_today_completed_count().unwrap_or(0);
    }

    pub fn set_completion_message(&mut self, show: bool) {
        self.show_completion_message = show;
    }
}

impl Component for DashboardComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Char('q') => Ok(Action::Quit),
            KeyCode::Char('r') => Ok(Action::NavigateTo(Screen::Review)),
            KeyCode::Char('n') => {
                // Learn new words - handled by app
                Ok(Action::LearnNew)
            }
            KeyCode::Char('d') => {
                self.show_completion_message = false;
                Ok(Action::NavigateTo(Screen::Dictionary))
            }
            KeyCode::Char('h') => {
                self.show_completion_message = false;
                Ok(Action::NavigateTo(Screen::History))
            }
            KeyCode::Char('s') => {
                self.show_completion_message = false;
                Ok(Action::NavigateTo(Screen::Statistics))
            }
            KeyCode::Esc => {
                self.show_completion_message = false;
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Statistics
                Constraint::Length(3),  // Progress bar
                Constraint::Length(3),  // Today's progress
                Constraint::Min(1),     // Actions/Messages
            ])
            .margin(1)
            .split(area);

        let (total, mastered, due) = self.stats;

        // Statistics
        let stats_text = format!(
            "Total Words: {} | Mastered: {} | Due Today: {}",
            total, mastered, due
        );
        let stats_widget = Paragraph::new(stats_text)
            .block(Block::default().title(" Statistics ").borders(Borders::ALL));
        frame.render_widget(stats_widget, chunks[0]);

        // Progress bar
        let progress = if total > 0 {
            (mastered as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" Mastery Progress ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent(progress as u16);
        frame.render_widget(gauge, chunks[1]);

        // Today's completed reviews
        let today_text = format!("Today's Completed Reviews: {} 🎯", self.today_completed);
        let today_widget = Paragraph::new(today_text)
            .block(
                Block::default()
                    .title(" Today's Progress ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(today_widget, chunks[2]);

        // Show completion message or instructions
        if self.show_completion_message {
            let completion_lines = vec![
                Line::from(Span::styled(
                    "✓ Great job! All due reviews completed!",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "'n'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to learn "),
                    Span::styled("new words", Style::default().fg(Color::Cyan)),
                    Span::raw(" | "),
                    Span::styled(
                        "'d'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" for Dictionary"),
                ]),
                Line::from(vec![
                    Span::styled(
                        "'h'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" for History | "),
                    Span::styled(
                        "'s'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" for Statistics | "),
                    Span::styled(
                        "'q'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to Quit"),
                ]),
            ];
            let completion_msg = Paragraph::new(completion_lines)
                .block(Block::default().title(" Actions ").borders(Borders::ALL))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(completion_msg, chunks[3]);
        } else {
            let instructions = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        "'r'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Review Due Words | "),
                    Span::styled(
                        "'n'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Learn New Words"),
                ]),
                Line::from(vec![
                    Span::styled(
                        "'d'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Dictionary | "),
                    Span::styled(
                        "'h'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" History | "),
                    Span::styled(
                        "'s'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Statistics | "),
                    Span::styled(
                        "'q'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Quit"),
                ]),
            ])
            .block(Block::default().title(" Actions ").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(instructions, chunks[3]);
        }
    }
}
