use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

pub struct ProgressBar {
    current: usize,
    total: usize,
    label: String,
    color: Color,
}

impl ProgressBar {
    pub fn new(current: usize, total: usize) -> Self {
        Self {
            current,
            total,
            label: format!("{}/{}", current, total),
            color: Theme::PRIMARY,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let percent = if self.total > 0 {
            ((self.current as f64 / self.total as f64) * 100.0) as u16
        } else {
            0
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(self.color))
            .percent(percent)
            .label(self.label.clone());

        frame.render_widget(gauge, area);
    }
}
