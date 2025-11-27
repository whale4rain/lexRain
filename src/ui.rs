use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::app::{App, CurrentScreen, ReviewState};
use crate::models::LearningStatus;

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
    let titles = vec!["Dashboard", "Review", "Dictionary", "Quit"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" LexRain "))
        .select(match app.current_screen {
            CurrentScreen::Dashboard => 0,
            CurrentScreen::Review => 1,
            CurrentScreen::Dictionary => 2,
            CurrentScreen::Exiting => 3,
        })
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));
    frame.render_widget(tabs, area);
}

fn render_content(app: &mut App, frame: &mut Frame, area: Rect) {
    match app.current_screen {
        CurrentScreen::Dashboard => render_dashboard(app, frame, area),
        CurrentScreen::Review => render_review(app, frame, area),
        CurrentScreen::Dictionary => render_dictionary(app, frame, area),
        _ => {}
    }
}

fn render_dashboard(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)])
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
    
    let instructions = Paragraph::new("Press 'r' to start Review\nPress 'd' for Dictionary\nPress 'q' to Quit")
        .block(Block::default().title(" Actions ").borders(Borders::ALL));
    frame.render_widget(instructions, chunks[2]);
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
                let def_text = Paragraph::new(word.definition.as_str())
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

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let help_text = match app.current_screen {
        CurrentScreen::Review => match app.review_state {
            ReviewState::Question => "Space: Show Answer | q: Quit",
            ReviewState::Answer => "1: Forgot | 2: Hard | 3: Good | 4: Easy",
        },
        CurrentScreen::Dictionary => "Type: Search | ↑/↓: Navigate | Esc: Back | q: Quit",
        _ => "Tab: Switch Mode | q: Quit",
    };

    let p = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(p, area);
}
