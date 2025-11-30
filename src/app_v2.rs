use crate::components::*;
use crate::components::{
    dashboard::DashboardComponent, dictionary::DictionaryComponent, history::HistoryComponent,
    review::ReviewComponent, statistics::StatisticsComponent, wordbook::WordbookComponent,
};
use crate::db::Database;
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub struct AppV2 {
    current_screen: Screen,
    dashboard: DashboardComponent,
    review: Option<ReviewComponent>,
    dictionary: Option<DictionaryComponent>,
    history: Option<HistoryComponent>,
    statistics: Option<StatisticsComponent>,
    wordbook: Option<WordbookComponent>,
}

impl AppV2 {
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self {
            current_screen: Screen::Dashboard,
            dashboard: DashboardComponent::new(db),
            review: None,
            dictionary: None,
            history: None,
            statistics: None,
            wordbook: None,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        let action = match self.current_screen {
            Screen::Dashboard => self.dashboard.handle_key(key)?,
            Screen::Review => {
                if let Some(review) = &mut self.review {
                    let action = review.handle_key(key)?;
                    // Check if review is complete after handling key
                    if review.is_complete() && matches!(action, Action::None) {
                        self.dashboard.set_completion_message(true);
                        self.navigate_to(Screen::Dashboard)?;
                        return Ok(false);
                    }
                    action
                } else {
                    Action::NavigateTo(Screen::Dashboard)
                }
            }
            Screen::Dictionary => {
                if let Some(dict) = &mut self.dictionary {
                    dict.handle_key(key)?
                } else {
                    Action::NavigateTo(Screen::Dashboard)
                }
            }
            Screen::History => {
                if let Some(hist) = &mut self.history {
                    hist.handle_key(key)?
                } else {
                    Action::NavigateTo(Screen::Dashboard)
                }
            }
            Screen::Statistics => {
                if let Some(stats) = &mut self.statistics {
                    stats.handle_key(key)?
                } else {
                    Action::NavigateTo(Screen::Dashboard)
                }
            }
            Screen::Wordbook => {
                if let Some(wb) = &mut self.wordbook {
                    wb.handle_key(key)?
                } else {
                    Action::NavigateTo(Screen::Dashboard)
                }
            }
        };

        self.handle_action(action)
    }

    fn handle_action(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => Ok(true),
            Action::NavigateTo(screen) => {
                self.navigate_to(screen)?;
                Ok(false)
            }
            Action::StartWordbookReview(tag, shuffle) => {
                self.start_wordbook_review(&tag, shuffle)?;
                Ok(false)
            }
            Action::None => Ok(false),
        }
    }

    fn navigate_to(&mut self, screen: Screen) -> Result<()> {
        match screen {
            Screen::Dashboard => {
                self.dashboard.refresh_stats();
                self.current_screen = Screen::Dashboard;
            }
            Screen::Review => {
                // Check if we should start review or learn new
                let db = Database::initialize()?;
                let mut review = ReviewComponent::new(db);

                // Try to start due reviews first
                if !review.start_review(review::ReviewMode::Due)? {
                    // No due reviews, show completion message
                    self.dashboard.set_completion_message(true);
                    self.current_screen = Screen::Dashboard;
                    return Ok(());
                }

                self.review = Some(review);
                self.current_screen = Screen::Review;
            }
            Screen::Dictionary => {
                let db = Database::initialize()?;
                self.dictionary = Some(DictionaryComponent::new(db)?);
                self.current_screen = Screen::Dictionary;
            }
            Screen::History => {
                let db = Database::initialize()?;
                self.history = Some(HistoryComponent::new(db)?);
                self.current_screen = Screen::History;
            }
            Screen::Statistics => {
                let db = Database::initialize()?;
                self.statistics = Some(StatisticsComponent::new(db)?);
                self.current_screen = Screen::Statistics;
            }
            Screen::Wordbook => {
                let db = Database::initialize()?;
                self.wordbook = Some(WordbookComponent::new(db)?);
                self.current_screen = Screen::Wordbook;
            }
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Render header
        self.render_header(frame, area);

        // Calculate content area (excluding header and footer)
        let content_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height.saturating_sub(6),
        };

        // Render current screen
        match self.current_screen {
            Screen::Dashboard => self.dashboard.view(frame, content_area),
            Screen::Review => {
                if let Some(review) = &mut self.review {
                    review.view(frame, content_area);
                }
            }
            Screen::Dictionary => {
                if let Some(dict) = &mut self.dictionary {
                    dict.view(frame, content_area);
                }
            }
            Screen::History => {
                if let Some(hist) = &mut self.history {
                    hist.view(frame, content_area);
                }
            }
            Screen::Statistics => {
                if let Some(stats) = &mut self.statistics {
                    stats.view(frame, content_area);
                }
            }
            Screen::Wordbook => {
                if let Some(wb) = &mut self.wordbook {
                    wb.view(frame, content_area);
                }
            }
        }

        // Render footer
        self.render_footer(frame, area);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            style::{Color, Modifier, Style},
            widgets::{Block, Borders, Tabs},
        };

        let titles = vec![
            "Dashboard",
            "Review",
            "Dictionary",
            "History",
            "Statistics",
            "Wordbook",
            "Quit",
        ];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(" LexRain "))
            .select(match self.current_screen {
                Screen::Dashboard => 0,
                Screen::Review => 1,
                Screen::Dictionary => 2,
                Screen::History => 3,
                Screen::Statistics => 4,
                Screen::Wordbook => 5,
            })
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            );

        let header_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        };

        frame.render_widget(tabs, header_area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        use crate::components::common::StatusBar;

        let footer_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(3),
            width: area.width,
            height: 3,
        };

        let status_bar = match self.current_screen {
            Screen::Dashboard => StatusBar::new()
                .add_item("r", "Review")
                .add_item("w", "Wordbook")
                .add_item("d", "Dictionary")
                .add_item("h", "History")
                .add_item("s", "Statistics")
                .add_item("q", "Quit"),
            Screen::Review => StatusBar::new()
                .add_item("Space", "Show Answer")
                .add_item("1-4", "Rate Quality")
                .add_item("q/Esc", "Back"),
            Screen::Dictionary => StatusBar::new()
                .add_item("Type", "Search")
                .add_item("↑/↓/j/k", "Navigate")
                .add_item("g/G", "First/Last")
                .add_item("PgUp/PgDn", "Page")
                .add_item("q/Esc", "Back"),
            Screen::History => StatusBar::new().add_item("q/Esc", "Back"),
            Screen::Statistics => StatusBar::new().add_item("q/Esc", "Back"),
            Screen::Wordbook => StatusBar::new()
                .add_item("Enter", "Start Review")
                .add_item("s", "Toggle Shuffle")
                .add_item("↑/↓", "Select")
                .add_item("q", "Back"),
        };

        status_bar.render(frame, footer_area);
    }

    pub fn start_wordbook_review(&mut self, tag: &str, shuffle: bool) -> Result<()> {
        let db = Database::initialize()?;
        let mut review = ReviewComponent::new(db);

        if !review.start_review(review::ReviewMode::Wordbook(tag.to_string(), shuffle))? {
            // No words available in this wordbook
            return Ok(());
        }

        self.review = Some(review);
        self.current_screen = Screen::Review;
        Ok(())
    }
}
