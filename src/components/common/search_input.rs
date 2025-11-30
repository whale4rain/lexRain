use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SearchInput {
    pub value: String,
    placeholder: String,
    focused: bool,
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: "Type to search...".to_string(),
            focused: true,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.value.push(c);
                true
            }
            KeyCode::Backspace => {
                self.value.pop();
                true
            }
            _ => false,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let style = if self.value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let input = Paragraph::new(display_text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search ")
                    .border_style(if self.focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .style(style);

        frame.render_widget(input, area);
    }
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new()
    }
}
