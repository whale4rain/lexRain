use super::{Action, Component, Screen};
use crate::db::Database;
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    symbols,
    text::Span,
    widgets::{Axis, Bar, BarChart, BarGroup, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

pub struct StatisticsComponent {
    interval_data: Vec<(i32, f64, i64)>, // interval, avg_quality, count
    daily_data: Vec<(String, i64)>,      // date, count
}

impl StatisticsComponent {
    pub fn new(db: Database) -> Result<Self> {
        let interval_data = db.get_review_stats_by_interval()?;
        let daily_data = db.get_daily_review_counts(7)?; // 改为7天

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
                .style(Theme::text_title())
                .data(&data);

            let x_labels = vec![
                Span::raw("0"),
                Span::raw(format!("{}", (x_max / 2.0) as i32)),
                Span::raw(format!("{}", x_max as i32)),
            ];

            let chart = Chart::new(vec![dataset])
                .block(
                    Theme::block_default()
                        .title(" Forgetting Curve (Quality vs Interval Days) "),
                )
                .x_axis(
                    Axis::default()
                        .title("Interval (days)")
                        .style(Theme::text_normal())
                        .bounds([0.0, x_max])
                        .labels(x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Quality")
                        .style(Theme::text_normal())
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
                Theme::block_default()
                    .title(" Forgetting Curve "),
            );
            frame.render_widget(msg, layout[0]);
        }

        // Daily Review Count Bar Chart (Last 7 Days)
        if !self.daily_data.is_empty() {
            // Prepare bar chart data
            let bars: Vec<Bar> = self
                .daily_data
                .iter()
                .map(|(date, count)| {
                    // Extract day (MM-DD format)
                    let label = if date.len() >= 10 {
                        &date[5..10] // Extract MM-DD
                    } else {
                        date.as_str()
                    };
                    Bar::default()
                        .value(*count as u64)
                        .label(label.into())
                        .style(Theme::text_success())
                        .value_style(Theme::text_accent().add_modifier(ratatui::style::Modifier::BOLD))
                })
                .collect();

            let bar_chart = BarChart::default()
                .block(
                    Theme::block_success_with_title(" 📊 Daily Review Activity (Last 7 Days) ")
                )
                .bar_width(9)
                .bar_gap(2)
                .bar_style(Theme::text_success())
                .value_style(Theme::text_normal())
                .data(BarGroup::default().bars(&bars));

            frame.render_widget(bar_chart, layout[1]);
        } else {
            let msg = Paragraph::new(
                "No daily review data available yet.\nComplete some reviews to see your activity!",
            )
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Theme::block_success_with_title(" 📊 Daily Review Activity ")
            );
            frame.render_widget(msg, layout[1]);
        }
    }
}
