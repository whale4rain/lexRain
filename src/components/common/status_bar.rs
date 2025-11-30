use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub struct StatusBar {
    items: Vec<(String, String)>, // (key, description)
}

impl StatusBar {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.items.push((key.into(), description.into()));
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        for (i, (key, desc)) in self.items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }
            spans.push(Span::styled(
                key,
                Theme::text_warning(),
            ));
            spans.push(Span::raw(": "));
            spans.push(Span::raw(desc));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line)
            .style(Theme::text_secondary())
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
