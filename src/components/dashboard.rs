use super::{Action, Component, Screen};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    widgets::calendar::{CalendarEventStore, Monthly},
    Frame,
};
use time::OffsetDateTime;

pub struct DashboardComponent {
    db: Database,
    stats: (i64, i64, i64), // total, mastered, due
    today_completed: i64,
    wordbook_count: usize,
    show_completion_message: bool,
}

impl DashboardComponent {
    pub fn new(db: Database) -> Self {
        let stats = db.get_stats().unwrap_or((0, 0, 0));
        let today_completed = db.get_today_completed_count().unwrap_or(0);
        let wordbook_count = db.get_wordbooks().unwrap_or_default().len();

        Self {
            db,
            stats,
            today_completed,
            wordbook_count,
            show_completion_message: false,
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self.db.get_stats().unwrap_or((0, 0, 0));
        self.today_completed = self.db.get_today_completed_count().unwrap_or(0);
        self.wordbook_count = self.db.get_wordbooks().unwrap_or_default().len();
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
            KeyCode::Char('w') => {
                self.show_completion_message = false;
                Ok(Action::NavigateTo(Screen::Wordbook))
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
        // Main layout: 2 columns
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Left column (stats + wordbooks + actions)
                Constraint::Percentage(40),  // Right column (calendar + progress)
            ])
            .split(area);

        // Left column layout
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),   // Stats card (2 rows)
                Constraint::Length(5),   // Wordbooks card
                Constraint::Min(8),      // Actions/Messages
            ])
            .margin(1)
            .split(main_layout[0]);

        // Right column layout
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),  // Calendar
                Constraint::Length(3),   // Today's progress
                Constraint::Min(3),      // Progress bar
            ])
            .margin(1)
            .split(main_layout[1]);

        let (total, mastered, due) = self.stats;

        // === LEFT COLUMN ===
        
        // Stats card with 2x3 grid
        let stats_lines = vec![
            Line::from(vec![
                Span::styled("📊 ", Style::default().fg(Color::Cyan)),
                Span::styled("Total Words: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", total),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::styled("Mastered: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", mastered),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⏰ ", Style::default().fg(Color::Yellow)),
                Span::styled("Due Today: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", due),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("🎯 ", Style::default().fg(Color::Magenta)),
                Span::styled("Completed: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", self.today_completed),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📖 ", Style::default().fg(Color::Blue)),
                Span::styled("Wordbooks: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", self.wordbook_count),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let stats_widget = Paragraph::new(stats_lines)
            .block(Block::default().title(" 📈 Statistics ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(stats_widget, left_chunks[0]);

        // Wordbooks card with icon
        let wordbook_text = vec![
            Line::from(vec![
                Span::styled("📚 ", Style::default().fg(Color::Magenta)),
                Span::raw("Press "),
                Span::styled(
                    "'w'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to explore "),
                Span::styled(
                    format!("{} wordbooks", self.wordbook_count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("   (CET-4/6, TOEFL, IELTS, GRE, etc.)"),
            ]),
        ];
        let wordbook_widget = Paragraph::new(wordbook_text)
            .block(Block::default().title(" 📖 Wordbooks ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(wordbook_widget, left_chunks[1]);

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
                        "'w'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" for "),
                    Span::styled("Wordbook Review", Style::default().fg(Color::Cyan)),
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
                .block(Block::default().title(" 🎉 Actions ").borders(Borders::ALL))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(completion_msg, left_chunks[2]);
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
                        "'w'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Wordbook Review"),
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
            .block(Block::default().title(" ⌨️  Quick Actions ").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(instructions, left_chunks[2]);
        }

        // === RIGHT COLUMN ===

        // Calendar
        let today = OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc())
            .date();
        
        let event_store = CalendarEventStore::today(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue)
                .fg(Color::White),
        );

        let calendar = Monthly::new(today, event_store)
            .show_month_header(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .show_weekdays_header(Style::default().fg(Color::Yellow))
            .default_style(Style::default().fg(Color::White));

        let calendar_block = Block::default()
            .title(" 📅 Calendar ")
            .borders(Borders::ALL);
        let calendar_inner = calendar_block.inner(right_chunks[0]);
        frame.render_widget(calendar_block, right_chunks[0]);
        frame.render_widget(calendar, calendar_inner);

        // Today's date display
        let today_text = format!(
            "📆 {} {}",
            today.format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap_or_else(|_| "Unknown".to_string()),
            today.format(&time::format_description::parse("[weekday]").unwrap())
                .unwrap_or_else(|_| "".to_string())
        );
        let today_widget = Paragraph::new(today_text)
            .block(Block::default().title(" Today ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(today_widget, right_chunks[1]);

        // Progress bar
        let progress = if total > 0 {
            (mastered as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" 📊 Mastery Progress ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent(progress as u16)
            .label(format!("{:.1}%", progress));
        frame.render_widget(gauge, right_chunks[2]);
    }
}
