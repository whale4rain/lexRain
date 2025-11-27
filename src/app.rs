use anyhow::Result;
use crate::db::Database;
use crate::models::{Word, LearningLog};
use crate::sm2;

pub enum CurrentScreen {
    Dashboard,
    Review,
    Dictionary,
    Exiting,
}

pub enum ReviewState {
    Question,
    Answer,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub db: Database,
    pub stats: (i64, i64, i64), // total, mastered, due
    
    // Review Mode State
    pub review_queue: Vec<(Word, LearningLog)>,
    pub current_review_item: Option<(Word, LearningLog)>,
    pub review_state: ReviewState,
}

impl App {
    pub fn new(db: Database) -> Self {
        let stats = db.get_stats().unwrap_or((0, 0, 0));
        Self {
            current_screen: CurrentScreen::Dashboard,
            db,
            stats,
            review_queue: Vec::new(),
            current_review_item: None,
            review_state: ReviewState::Question,
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self.db.get_stats().unwrap_or((0, 0, 0));
    }

    pub fn start_review(&mut self) -> Result<()> {
        self.review_queue = self.db.get_due_reviews()?;

        if self.review_queue.is_empty() {
            // No words to review, stay on dashboard
            return Ok(());
        }

        self.next_review_card();
        self.current_screen = CurrentScreen::Review;
        Ok(())
    }

    pub fn next_review_card(&mut self) {
        self.current_review_item = self.review_queue.pop();
        self.review_state = ReviewState::Question;

        // If no more cards, return to dashboard
        if self.current_review_item.is_none() {
            self.refresh_stats();
            self.current_screen = CurrentScreen::Dashboard;
        }
    }

    pub fn show_answer(&mut self) {
        self.review_state = ReviewState::Answer;
    }

    pub fn submit_review(&mut self, quality: u8) -> Result<()> {
        if let Some((_, mut log)) = self.current_review_item.take() {
            sm2::process_review(&mut log, quality);
            self.db.update_log(&log)?;

            // Refresh statistics
            self.refresh_stats();

            // If quality is low, maybe requeue it for same session?
            // For now, we just save and move to next.
            self.next_review_card();
        }
        Ok(())
    }
}
