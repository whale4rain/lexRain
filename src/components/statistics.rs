use super::{Action, Component, Screen};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

pub struct StatisticsComponent {
    interval_data: Vec<(i32, f64, i64)>, // interval, avg_quality, count
    daily_data: Vec<(String, i64)>,      // date, count
}

impl StatisticsComponent {
    pub fn new(db: Database) -> Result<Self> {
        let interval_data = db.get_review_stats_by_interval()?;
        let daily_data = db.get_daily_review_counts(30)?;

        Ok(Self {
            interval_data,
            daily_data,
        })
    }
}

impl Component for StatisticsComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::NavigateTo(Screen::Dashboard)),
            _ => Ok(Action::None),
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Forgetting curve chart
                Constraint::Percentage(50), // Daily review chart
            ])
            .margin(1)
            .split(area);

        // Forgetting Curve Chart
        if !self.interval_data.is_empty() {
            let data: Vec<(f64, f64)> = self
                .interval_data
                .iter()
                .map(|(interval, avg_quality, _)| (*interval as f64, *avg_quality))
                .collect();

            let max_interval = self
                .interval_data
                .iter()
                .map(|(interval, _, _)| *interval)
                .max()
                .unwrap_or(30) as f64;

            let x_max = (max_interval * 1.1).max(10.0);

            let dataset = Dataset::default()
                .name("Avg Quality")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::Cyan))
                .data(&data);

            let x_labels = vec![
                Span::raw("0"),
                Span::raw(format!("{}", (x_max / 2.0) as i32)),
                Span::raw(format!("{}", x_max as i32)),
            ];

            let chart = Chart::new(vec![dataset])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Forgetting Curve (Quality vs Interval Days) "),
                )
                .x_axis(
                    Axis::default()
                        .title("Interval (days)")
                        .style(Style::new().fg(Color::White))
                        .bounds([0.0, x_max])
                        .labels(x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Quality")
                        .style(Style::new().fg(Color::White))
                        .bounds([1.0, 4.0])
                        .labels(vec![Span::raw("1.0"), Span::raw("2.5"), Span::raw("4.0")]),
                );

            frame.render_widget(chart, layout[0]);
        } else {
            let msg = Paragraph::new(
                "No review data available yet.\nComplete some reviews to see the forgetting curve!",
            )
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Forgetting Curve "),
            );
            frame.render_widget(msg, layout[0]);
        }

        // Daily Review Count Chart
        if !self.daily_data.is_empty() {
            let data: Vec<(f64, f64)> = self
                .daily_data
                .iter()
                .enumerate()
                .map(|(idx, (_, count))| (idx as f64, *count as f64))
                .collect();

            let max_count = self
                .daily_data
                .iter()
                .map(|(_, count)| *count)
                .max()
                .unwrap_or(10) as f64;

            let y_max = (max_count * 1.2).max(5.0);

            let dataset = Dataset::default()
                .name("Reviews")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::Green))
                .data(&data);

            let days_count = self.daily_data.len() as f64;

            let x_labels = if days_count > 0.0 {
                let first_date = self.daily_data.first().map(|(d, _)| {
                    if d.len() >= 10 {
                        &d[5..10]
                    } else {
                        d.as_str()
                    }
                }).unwrap_or("Start");

                let last_date = self.daily_data.last().map(|(d, _)| {
                    if d.len() >= 10 {
                        &d[5..10]
                    } else {
                        d.as_str()
                    }
                }).unwrap_or("End");

                vec![Span::raw(first_date), Span::raw("..."), Span::raw(last_date)]
            } else {
                vec![Span::raw("0"), Span::raw("15"), Span::raw("30")]
            };

            let chart = Chart::new(vec![dataset])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Daily Review Activity (Last 30 Days) "),
                )
                .x_axis(
                    Axis::default()
                        .title("Date")
                        .style(Style::new().fg(Color::White))
                        .bounds([0.0, days_count.max(1.0)])
                        .labels(x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Count")
                        .style(Style::new().fg(Color::White))
                        .bounds([0.0, y_max])
                        .labels(vec![
                            Span::raw("0"),
                            Span::raw(format!("{}", (y_max / 2.0) as i64)),
                            Span::raw(format!("{}", y_max as i64)),
                        ]),
                );

            frame.render_widget(chart, layout[1]);
        } else {
            let msg = Paragraph::new(
                "No daily review data available yet.\nComplete some reviews to see your activity!",
            )
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Daily Review Activity "),
            );
            frame.render_widget(msg, layout[1]);
        }
    }
}
