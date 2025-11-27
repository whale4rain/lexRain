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
    pub total_review_count: usize,
    pub completed_review_count: usize,

    // Dictionary Mode State
    pub dict_search_input: String,
    pub dict_word_list: Vec<(Word, Option<LearningLog>)>,
    pub dict_selected_index: usize,
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
            total_review_count: 0,
            completed_review_count: 0,
            dict_search_input: String::new(),
            dict_word_list: Vec::new(),
            dict_selected_index: 0,
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self.db.get_stats().unwrap_or((0, 0, 0));
    }

    pub fn start_review(&mut self) -> Result<()> {
        self.review_queue = self.db.get_due_reviews()?;
        self.total_review_count = self.review_queue.len();
        self.completed_review_count = 0;

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
            self.completed_review_count += 1;

            // If quality is low, maybe requeue it for same session?
            // For now, we just save and move to next.
            self.next_review_card();
        }
        Ok(())
    }

    // Dictionary methods
    pub fn enter_dictionary(&mut self) -> Result<()> {
        self.dict_search_input.clear();
        self.dict_word_list = self.db.get_all_words()?;
        self.dict_selected_index = 0;
        self.current_screen = CurrentScreen::Dictionary;
        Ok(())
    }

    pub fn dict_update_search(&mut self) -> Result<()> {
        if self.dict_search_input.is_empty() {
            self.dict_word_list = self.db.get_all_words()?;
        } else {
            self.dict_word_list = self.db.search_words(&self.dict_search_input)?;
        }
        self.dict_selected_index = 0;
        Ok(())
    }

    pub fn dict_select_next(&mut self) {
        if !self.dict_word_list.is_empty() {
            self.dict_selected_index = (self.dict_selected_index + 1) % self.dict_word_list.len();
        }
    }

    pub fn dict_select_previous(&mut self) {
        if !self.dict_word_list.is_empty() {
            if self.dict_selected_index == 0 {
                self.dict_selected_index = self.dict_word_list.len() - 1;
            } else {
                self.dict_selected_index -= 1;
            }
        }
    }
}
