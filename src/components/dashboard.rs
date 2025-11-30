use super::{Action, Component, Screen};
use crate::db::Database;
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
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
            KeyCode::Char('c') => {
                self.show_completion_message = false;
                Ok(Action::NavigateTo(Screen::Settings))
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
        
        // Stats card - clearer labels
        let stats_lines = vec![
            Line::from(vec![
                Span::styled("📚 ", Theme::text_title()),
                Span::styled("Learning: ", Theme::text_normal()),
                Span::styled(
                    format!("{} words", total),
                    Theme::text_title(),
                ),
                Span::styled(" (total in your vocabulary)", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("✓ ", Theme::text_success()),
                Span::styled("Mastered: ", Theme::text_normal()),
                Span::styled(
                    format!("{} words", mastered),
                    Theme::text_success(),
                ),
                Span::styled(" (fully learned)", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⏰ ", Theme::text_warning()),
                Span::styled("Due Now: ", Theme::text_normal()),
                Span::styled(
                    format!("{} words", due),
                    Theme::text_warning(),
                ),
                Span::styled(" (need review)", Theme::text_secondary()),
            ]),
        ];
        let stats_widget = Paragraph::new(stats_lines)
            .block(Theme::block_with_title(" 📊 Learning Stats "))
            .style(Theme::text_normal());
        frame.render_widget(stats_widget, left_chunks[0]);

        // Today's progress card
        let daily_goal = self.db.get_daily_goal().unwrap_or(20);
        let progress_text = vec![
            Line::from(vec![
                Span::styled("🎯 ", Theme::text_accent()),
                Span::styled("Today: ", Theme::text_normal()),
                Span::styled(
                    format!("{}", self.today_completed),
                    Theme::text_accent(),
                ),
                Span::raw(" / "),
                Span::styled(
                    format!("{} words reviewed", daily_goal),
                    Theme::text_title(),
                ),
            ]),
        ];
        let progress_widget = Paragraph::new(progress_text)
            .block(Theme::block_with_title(" 📅 Today's Progress "))
            .style(Theme::text_normal());
        frame.render_widget(progress_widget, left_chunks[1]);

        // Show completion message or instructions
        if self.show_completion_message {
            let completion_lines = vec![
                Line::from(Span::styled(
                    "✓ Great job! All due reviews completed!",
                    Theme::text_success(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("'w'", Theme::text_warning()),
                    Span::raw(" for "),
                    Span::styled("Wordbook Review", Theme::text_title()),
                    Span::raw(" | "),
                    Span::styled("'d'", Theme::text_warning()),
                    Span::raw(" for Dictionary"),
                ]),
                Line::from(vec![
                    Span::styled("'h'", Theme::text_warning()),
                    Span::raw(" for History | "),
                    Span::styled("'s'", Theme::text_warning()),
                    Span::raw(" for Statistics | "),
                    Span::styled("'q'", Theme::text_warning()),
                    Span::raw(" to Quit"),
                ]),
            ];
            let completion_msg = Paragraph::new(completion_lines)
                .block(Theme::block_success_with_title(" 🎉 Actions "))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(completion_msg, left_chunks[2]);
        } else {
            let instructions = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("'r'", Theme::text_warning()),
                    Span::raw(" Review Due Words | "),
                    Span::styled("'w'", Theme::text_warning()),
                    Span::raw(" Wordbook Review"),
                ]),
                Line::from(vec![
                    Span::styled("'d'", Theme::text_warning()),
                    Span::raw(" Dictionary | "),
                    Span::styled("'h'", Theme::text_warning()),
                    Span::raw(" History | "),
                    Span::styled("'s'", Theme::text_warning()),
                    Span::raw(" Statistics | "),
                    Span::styled("'c'", Theme::text_warning()),
                    Span::raw(" Settings | "),
                    Span::styled("'q'", Theme::text_warning()),
                    Span::raw(" Quit"),
                ]),
            ])
            .block(Theme::block_with_title(" ⌨️  Quick Actions "))
            .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(instructions, left_chunks[2]);
        }

        // === RIGHT COLUMN ===

        // Calendar with checkin marks
        let today = OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc())
            .date();
        
        // Create event store with today highlighted
        let mut event_store = CalendarEventStore::today(
            Theme::text_normal()
                .add_modifier(Modifier::BOLD)
                .bg(Theme::PRIMARY)
        );

        // Add checkin marks for completed days
        if let Ok(checkin_dates) = self.db.get_checkin_dates(today.year(), today.month() as u32) {
            let checkin_style = Theme::text_success()
                .bg(Color::Rgb(0, 50, 0));
            
            for date_str in checkin_dates {
                // Parse YYYY-MM-DD format
                let parts: Vec<&str> = date_str.split('-').collect();
                if parts.len() == 3 {
                    if let (Ok(y), Ok(m), Ok(d)) = (
                        parts[0].parse::<i32>(),
                        parts[1].parse::<u8>(),
                        parts[2].parse::<u8>()
                    ) {
                        if let Ok(month) = time::Month::try_from(m) {
                            if let Ok(parsed_date) = time::Date::from_calendar_date(y, month, d) {
                                event_store.add(parsed_date, checkin_style);
                            }
                        }
                    }
                }
            }
        }

        let calendar = Monthly::new(today, event_store)
            .show_month_header(Theme::text_title())
            .show_weekdays_header(Theme::text_warning())
            .default_style(Theme::text_normal());

        let calendar_block = Theme::block_with_title(" 📅 Calendar ");
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
            .block(Theme::block_accent_with_title(" Today "))
            .style(Theme::text_title());
        frame.render_widget(today_widget, right_chunks[1]);

        // Overall learning progress bar
        let progress = if total > 0 {
            (mastered as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let progress_title = format!(" ✓ Mastered: {} / {} ({:.1}%) ", mastered, total, progress);
        let gauge = Gauge::default()
            .block(
                Theme::block_success().title(progress_title),
            )
            .gauge_style(Theme::text_success())
            .percent(progress as u16)
            .label(format!("{} mastered", mastered));
        frame.render_widget(gauge, right_chunks[2]);
    }
}
