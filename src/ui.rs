use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Tabs, Wrap, Chart, Dataset, Axis, GraphType},
    symbols,
    Frame,
};
use crate::app::{App, CurrentScreen, ReviewState};
use crate::models::LearningStatus;
use crate::theme::Theme;

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Footer/Help
        ])
        .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let titles = vec!["Dashboard", "Review", "Dictionary", "History", "Statistics", "Quit"];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Theme::text_normal())
                .title(" LexRain ")
        )
        .select(match app.current_screen {
            CurrentScreen::Dashboard => 0,
            CurrentScreen::Review => 1,
            CurrentScreen::Dictionary => 2,
            CurrentScreen::History => 3,
            CurrentScreen::Statistics => 4,
            CurrentScreen::Exiting => 5,
        })
        .highlight_style(
            Style::default()
                .fg(Theme::PRIMARY)
                .bg(Theme::FOREGROUND)
                .add_modifier(Modifier::BOLD)
        );
    frame.render_widget(tabs, area);
}

fn render_content(app: &mut App, frame: &mut Frame, area: Rect) {
    match app.current_screen {
        CurrentScreen::Dashboard => render_dashboard(app, frame, area),
        CurrentScreen::Review => render_review(app, frame, area),
        CurrentScreen::Dictionary => render_dictionary(app, frame, area),
        CurrentScreen::History => render_history(app, frame, area),
        CurrentScreen::Statistics => render_statistics(app, frame, area),
        _ => {}
    }
}

fn render_dashboard(app: &App, frame: &mut Frame, area: Rect) {
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

    let (total, mastered, due) = app.stats;

    let stats_text = format!("Total Words: {} | Mastered: {} | Due Today: {}", total, mastered, due);
    let p = Paragraph::new(stats_text)
        .block(Block::default().title(" Statistics ").borders(Borders::ALL));
    frame.render_widget(p, chunks[0]);

    let progress = if total > 0 { (mastered as f64 / total as f64) * 100.0 } else { 0.0 };
    let gauge = Gauge::default()
        .block(Block::default().title(" Mastery Progress ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(progress as u16);
    frame.render_widget(gauge, chunks[1]);

    // Today's completed reviews
    let today_text = format!("Today's Completed Reviews: {} 🎯", app.today_completed);
    let today_widget = Paragraph::new(today_text)
        .block(Block::default().title(" Today's Progress ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(today_widget, chunks[2]);

    // Show completion message or instructions
    if app.show_completion_message {
        let completion_lines = vec![
            Line::from(Span::styled(
                "✓ Great job! All due reviews completed!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("'n'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" to learn "),
                Span::styled("new words", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled("'d'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" for Dictionary"),
            ]),
            Line::from(vec![
                Span::styled("'h'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" for History | "),
                Span::styled("'s'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" for Statistics | "),
                Span::styled("'q'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
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
                Span::styled("'r'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Review Due Words | "),
                Span::styled("'n'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Learn New Words"),
            ]),
            Line::from(vec![
                Span::styled("'d'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Dictionary | "),
                Span::styled("'h'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" History | "),
                Span::styled("'s'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Statistics | "),
                Span::styled("'q'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Quit"),
            ]),
        ])
        .block(Block::default().title(" Actions ").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(instructions, chunks[3]);
    }
}

fn render_review(app: &App, frame: &mut Frame, area: Rect) {
    if let Some((word, _)) = &app.current_review_item {
        let block = Block::default().borders(Borders::ALL).title(" Review ");
        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Progress bar
                Constraint::Percentage(35), // Word
                Constraint::Percentage(15), // Phonetic
                Constraint::Percentage(50), // Definition (Hidden/Shown)
            ])
            .split(inner_area);

        // Progress bar
        let remaining = app.total_review_count - app.completed_review_count;
        let progress_text = format!("Progress: {}/{} (Remaining: {})",
            app.completed_review_count, app.total_review_count, remaining);
        let progress_percent = if app.total_review_count > 0 {
            ((app.completed_review_count as f64 / app.total_review_count as f64) * 100.0) as u16
        } else {
            0
        };
        let progress_bar = Gauge::default()
            .block(Block::default().borders(Borders::BOTTOM))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(progress_percent)
            .label(progress_text);
        frame.render_widget(progress_bar, layout[0]);

        // Word
        let word_text = Paragraph::new(Span::styled(
            &word.spelling,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(word_text, layout[1]);

        // Phonetic
        if let Some(phonetic) = &word.phonetic {
            let phonetic_text = Paragraph::new(format!("[ {} ]", phonetic))
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(phonetic_text, layout[2]);
        }

        // Definition
        match app.review_state {
            ReviewState::Question => {
                let hint = Paragraph::new("Press <Space> to show definition")
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::Gray));
                frame.render_widget(hint, layout[3]);
            }
            ReviewState::Answer => {
                let mut def_lines = vec![
                    Line::from(Span::styled("English:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                    Line::from(word.definition.as_str()),
                ];

                if let Some(translation) = &word.translation {
                    def_lines.push(Line::from(""));
                    def_lines.push(Line::from(Span::styled("中文:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                    def_lines.push(Line::from(translation.as_str()));
                }

                let def_text = Paragraph::new(def_lines)
                    .wrap(Wrap { trim: true })
                    .alignment(ratatui::layout::Alignment::Center)
                    .block(Block::default().borders(Borders::TOP).title(" Definition "));
                frame.render_widget(def_text, layout[3]);
            }
        }
    } else {
        let msg = Paragraph::new("No words to review today! Great job!")
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(msg, area);
    }
}

fn render_dictionary(app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search input
            Constraint::Min(5),     // Word list
            Constraint::Length(6),  // Selected word detail
        ])
        .margin(1)
        .split(area);

    // Search input box
    let search_input = Paragraph::new(app.dict_search_input.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Search (type to filter) "))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(search_input, layout[0]);

    // Word list
    let items: Vec<ListItem> = app.dict_word_list
        .iter()
        .enumerate()
        .map(|(idx, (word, log))| {
            let status_symbol = if let Some(log) = log {
                match log.status {
                    LearningStatus::New => "◯",
                    LearningStatus::Learning => "◐",
                    LearningStatus::Mastered => "●",
                }
            } else {
                "◯"
            };

            let status_color = if let Some(log) = log {
                match log.status {
                    LearningStatus::New => Color::Gray,
                    LearningStatus::Learning => Color::Yellow,
                    LearningStatus::Mastered => Color::Green,
                }
            } else {
                Color::Gray
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{} ", status_symbol),
                    Style::default().fg(status_color)
                ),
                Span::styled(
                    format!("{:20}", word.spelling),
                    Style::default().fg(Color::Cyan)
                ),
                Span::styled(
                    word.phonetic.as_ref().map(|p| format!(" [{}]", p)).unwrap_or_default(),
                    Style::default().fg(Color::DarkGray)
                ),
            ]);

            if idx == app.dict_selected_index {
                ListItem::new(content).style(Style::default().bg(Color::DarkGray))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let list_title = format!(" Word List ({} words) ", app.dict_word_list.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title));
    frame.render_widget(list, layout[1]);

    // Selected word detail
    if let Some((word, log)) = app.dict_word_list.get(app.dict_selected_index) {
        let status_text = if let Some(log) = log {
            format!(
                "Status: {:?} | Repetition: {} | Interval: {} days | EF: {:.2}",
                log.status, log.repetition, log.interval, log.e_factor
            )
        } else {
            "Status: Not learned yet".to_string()
        };

        let detail_lines = vec![
            Line::from(vec![
                Span::styled("Definition: ", Style::default().fg(Color::Yellow)),
                Span::raw(&word.definition),
            ]),
            Line::from(""),
            Line::from(Span::styled(status_text, Style::default().fg(Color::DarkGray))),
        ];

        let detail = Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title(" Detail "))
            .wrap(Wrap { trim: true });
        frame.render_widget(detail, layout[2]);
    }
}

fn render_history(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app.history_list
        .iter()
        .map(|(word, reviewed_at, quality)| {
            let quality_text = match quality {
                1 => ("Forgot", Color::Red),
                2 => ("Hard", Color::Yellow),
                3 => ("Good", Color::Green),
                4 => ("Easy", Color::Cyan),
                _ => ("Unknown", Color::Gray),
            };

            let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reviewed_at) {
                dt.format("%Y-%m-%d %H:%M").to_string()
            } else {
                reviewed_at.clone()
            };

            let mut content_spans = vec![
                Span::styled(
                    format!("{:20}", word.spelling),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("{:10}", quality_text.0),
                    Style::default().fg(quality_text.1)
                ),
                Span::raw(" | "),
                Span::styled(
                    time_str,
                    Style::default().fg(Color::DarkGray)
                ),
            ];

            if let Some(translation) = &word.translation {
                content_spans.push(Span::raw("\n  "));
                content_spans.push(Span::styled(
                    translation.as_str(),
                    Style::default().fg(Color::Gray),
                ));
            }

            ListItem::new(Line::from(content_spans))
        })
        .collect();

    let list_title = format!(" Review History (Last {} reviews) ", app.history_list.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title));

    frame.render_widget(list, area);
}

fn render_statistics(app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Forgetting curve chart
            Constraint::Percentage(50),  // Daily review chart
        ])
        .margin(1)
        .split(area);

    // Forgetting Curve Chart (Retention by Interval)
    if !app.stats_interval_data.is_empty() {
        let data: Vec<(f64, f64)> = app.stats_interval_data
            .iter()
            .map(|(interval, avg_quality, _)| (*interval as f64, *avg_quality))
            .collect();

        let max_interval = app.stats_interval_data
            .iter()
            .map(|(interval, _, _)| *interval)
            .max()
            .unwrap_or(30) as f64;

        // Add padding to max_interval for better visualization
        let x_max = (max_interval * 1.1).max(10.0);

        let dataset = Dataset::default()
            .name("Avg Quality")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(Color::Cyan))
            .data(&data);

        let x_labels = vec![
            Span::raw("0"),
            Span::raw(format!("{}", (x_max / 2.0) as i32)),
            Span::raw(format!("{}", x_max as i32)),
        ];

        let chart = Chart::new(vec![dataset])
            .block(Block::default().borders(Borders::ALL).title(" Forgetting Curve (Quality vs Interval Days) "))
            .x_axis(
                Axis::default()
                    .title("Interval (days)")
                    .style(Style::new().fg(Color::White))
                    .bounds([0.0, x_max])
                    .labels(x_labels)
            )
            .y_axis(
                Axis::default()
                    .title("Quality")
                    .style(Style::new().fg(Color::White))
                    .bounds([1.0, 4.0])
                    .labels(vec![
                        Span::raw("1.0"),
                        Span::raw("2.5"),
                        Span::raw("4.0"),
                    ])
            );

        frame.render_widget(chart, layout[0]);
    } else {
        let msg = Paragraph::new("No review data available yet.\nComplete some reviews to see the forgetting curve!")
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Forgetting Curve "));
        frame.render_widget(msg, layout[0]);
    }

    // Daily Review Count Chart
    if !app.stats_daily_data.is_empty() {
        let data: Vec<(f64, f64)> = app.stats_daily_data
            .iter()
            .enumerate()
            .map(|(idx, (_, count))| (idx as f64, *count as f64))
            .collect();

        let max_count = app.stats_daily_data
            .iter()
            .map(|(_, count)| *count)
            .max()
            .unwrap_or(10) as f64;

        // Add padding to y-axis for better visualization
        let y_max = (max_count * 1.2).max(5.0);

        let dataset = Dataset::default()
            .name("Reviews")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(Color::Green))
            .data(&data);

        let days_count = app.stats_daily_data.len() as f64;

        // Format date labels more clearly
        let x_labels = if days_count > 0.0 {
            let first_date = app.stats_daily_data.first().map(|(d, _)| {
                // Extract month-day from date string (YYYY-MM-DD)
                if d.len() >= 10 {
                    &d[5..10]  // MM-DD
                } else {
                    d.as_str()
                }
            }).unwrap_or("Start");

            let last_date = app.stats_daily_data.last().map(|(d, _)| {
                if d.len() >= 10 {
                    &d[5..10]
                } else {
                    d.as_str()
                }
            }).unwrap_or("End");

            vec![
                Span::raw(first_date),
                Span::raw("..."),
                Span::raw(last_date),
            ]
        } else {
            vec![Span::raw("0"), Span::raw("15"), Span::raw("30")]
        };

        let chart = Chart::new(vec![dataset])
            .block(Block::default().borders(Borders::ALL).title(" Daily Review Activity (Last 30 Days) "))
            .x_axis(
                Axis::default()
                    .title("Date")
                    .style(Style::new().fg(Color::White))
                    .bounds([0.0, days_count.max(1.0)])
                    .labels(x_labels)
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
                    ])
            );

        frame.render_widget(chart, layout[1]);
    } else {
        let msg = Paragraph::new("No daily review data available yet.\nComplete some reviews to see your activity!")
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Daily Review Activity "));
        frame.render_widget(msg, layout[1]);
    }
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let help_text = match app.current_screen {
        CurrentScreen::Review => match app.review_state {
            ReviewState::Question => "Space: Show Answer | q: Quit",
            ReviewState::Answer => "1: Forgot | 2: Hard | 3: Good | 4: Easy",
        },
        CurrentScreen::Dictionary => "Type: Search | ↑/↓: Navigate | Esc: Back | q: Quit",
        CurrentScreen::History => "Esc: Back | q: Quit",
        CurrentScreen::Statistics => "Esc: Back | q: Quit",
        _ => "Tab: Switch Mode | q: Quit",
    };

    let p = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(p, area);
}
