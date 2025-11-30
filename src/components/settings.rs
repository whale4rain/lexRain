use super::{Action, Component, Screen};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SettingsComponent {
    db: Database,
    daily_goal: i64,
    editing: bool,
    input_buffer: String,
    message: Option<String>,
}

impl SettingsComponent {
    pub fn new(db: Database) -> Result<Self> {
        let daily_goal = db.get_daily_goal()?;
        Ok(Self {
            db,
            daily_goal,
            editing: false,
            input_buffer: String::new(),
            message: None,
        })
    }

    fn start_editing(&mut self) {
        self.editing = true;
        self.input_buffer = self.daily_goal.to_string();
        self.message = None;
    }

    fn cancel_editing(&mut self) {
        self.editing = false;
        self.input_buffer.clear();
        self.message = None;
    }

    fn save_setting(&mut self) -> Result<()> {
        if let Ok(goal) = self.input_buffer.parse::<i64>() {
            if goal > 0 && goal <= 1000 {
                self.db.set_daily_goal(goal)?;
                self.daily_goal = goal;
                self.editing = false;
                self.input_buffer.clear();
                self.message = Some("✓ Settings saved successfully!".to_string());
                Ok(())
            } else {
                self.message = Some("Error: Goal must be between 1 and 1000".to_string());
                Ok(())
            }
        } else {
            self.message = Some("Error: Invalid number".to_string());
            Ok(())
        }
    }
}

impl Component for SettingsComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        if self.editing {
            match key.code {
                KeyCode::Esc => {
                    self.cancel_editing();
                    Ok(Action::None)
                }
                KeyCode::Enter => {
                    self.save_setting()?;
                    Ok(Action::None)
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if self.input_buffer.len() < 4 {
                        self.input_buffer.push(c);
                    }
                    Ok(Action::None)
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
                KeyCode::Char('e') | KeyCode::Enter => {
                    self.start_editing();
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            }
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(10), // Daily goal setting
                Constraint::Length(3),  // Message
                Constraint::Min(5),     // Help
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new("⚙️  Settings")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Daily goal setting
        let goal_lines = if self.editing {
            vec![
                Line::from(vec![
                    Span::styled("📊 ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        "Daily Review Goal",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Enter goal (1-1000): "),
                    Span::styled(
                        &self.input_buffer,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    ),
                    Span::styled("_", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::raw(" to save | "),
                    Span::styled("Esc", Style::default().fg(Color::Red)),
                    Span::raw(" to cancel"),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("📊 ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        "Daily Review Goal",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Current goal: "),
                    Span::styled(
                        format!("{} words/day", self.daily_goal),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "'e'",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to edit"),
                ]),
            ]
        };

        let goal_widget = Paragraph::new(goal_lines)
            .block(Block::default().borders(Borders::ALL).title(" Setting "))
            .style(Style::default().fg(Color::White));
        frame.render_widget(goal_widget, chunks[1]);

        // Message
        if let Some(msg) = &self.message {
            let msg_style = if msg.starts_with("✓") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let message_widget = Paragraph::new(msg.as_str())
                .style(msg_style.add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(message_widget, chunks[2]);
        } else {
            let placeholder = Paragraph::new("")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(placeholder, chunks[2]);
        }

        // Help
        let help_lines = vec![
            Line::from(vec![
                Span::styled(
                    "💡 Tips:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("• Complete your daily goal to get a calendar check-in mark"),
            Line::from("• The calendar on dashboard shows your progress history"),
            Line::from("• Adjust the goal based on your learning pace"),
        ];

        let help_widget = Paragraph::new(help_lines)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_widget, chunks[3]);
    }
}
