use anyhow::Result;
use crate::db::Database;
use crate::models::{Word, LearningLog};
use crate::sm2;

pub enum CurrentScreen {
    Dashboard,
    Review,
    Dictionary,
    History,
    Statistics,
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
    pub today_completed: i64, // today's completed reviews

    // Review Mode State
    pub review_queue: Vec<(Word, LearningLog)>,
    pub current_review_item: Option<(Word, LearningLog)>,
    pub review_state: ReviewState,
    pub total_review_count: usize,
    pub completed_review_count: usize,
    pub show_completion_message: bool,

    // Dictionary Mode State
    pub dict_search_input: String,
    pub dict_word_list: Vec<(Word, Option<LearningLog>)>,
    pub dict_selected_index: usize,

    // History Mode State
    pub history_list: Vec<(Word, String, u8)>, // word, reviewed_at, quality

    // Statistics Mode State
    pub stats_interval_data: Vec<(i32, f64, i64)>, // interval, avg_quality, count
    pub stats_daily_data: Vec<(String, i64)>, // date, count
}

impl App {
    pub fn new(db: Database) -> Self {
        let stats = db.get_stats().unwrap_or((0, 0, 0));
        let today_completed = db.get_today_completed_count().unwrap_or(0);
        Self {
            current_screen: CurrentScreen::Dashboard,
            db,
            stats,
            today_completed,
            review_queue: Vec::new(),
            current_review_item: None,
            review_state: ReviewState::Question,
            total_review_count: 0,
            completed_review_count: 0,
            show_completion_message: false,
            dict_search_input: String::new(),
            dict_word_list: Vec::new(),
            dict_selected_index: 0,
            history_list: Vec::new(),
            stats_interval_data: Vec::new(),
            stats_daily_data: Vec::new(),
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self.db.get_stats().unwrap_or((0, 0, 0));
        self.today_completed = self.db.get_today_completed_count().unwrap_or(0);
    }

    pub fn start_review(&mut self) -> Result<()> {
        self.review_queue = self.db.get_due_reviews()?;
        self.total_review_count = self.review_queue.len();
        self.completed_review_count = 0;
        self.show_completion_message = false;

        if self.review_queue.is_empty() {
            // No words to review, show completion message
            self.show_completion_message = true;
            return Ok(());
        }

        self.next_review_card();
        self.current_screen = CurrentScreen::Review;
        Ok(())
    }

    // Start learning new words (review ahead)
    pub fn start_learn_new(&mut self) -> Result<()> {
        self.review_queue = self.db.get_new_words_to_learn(20)?; // Learn up to 20 new words
        self.total_review_count = self.review_queue.len();
        self.completed_review_count = 0;
        self.show_completion_message = false;

        if self.review_queue.is_empty() {
            // No new words available
            return Ok(());
        }

        self.next_review_card();
        self.current_screen = CurrentScreen::Review;
        Ok(())
    }

    pub fn next_review_card(&mut self) {
        self.current_review_item = self.review_queue.pop();
        self.review_state = ReviewState::Question;

        // If no more cards, return to dashboard with completion message
        if self.current_review_item.is_none() {
            self.refresh_stats();
            self.show_completion_message = true;
            self.current_screen = CurrentScreen::Dashboard;
        }
    }

    pub fn dismiss_completion_message(&mut self) {
        self.show_completion_message = false;
    }

    pub fn show_answer(&mut self) {
        self.review_state = ReviewState::Answer;
    }

    pub fn submit_review(&mut self, quality: u8) -> Result<()> {
        if let Some((word, mut log)) = self.current_review_item.take() {
            let word_id = word.id.unwrap();
            sm2::process_review(&mut log, quality);
            self.db.update_log(&log)?;

            // Record review in history
            self.db.add_review_history(word_id, quality, &log)?;

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

    // History methods
    pub fn enter_history(&mut self) -> Result<()> {
        self.history_list = self.db.get_recent_reviews(100)?;
        self.current_screen = CurrentScreen::History;
        Ok(())
    }

    // Statistics methods
    pub fn enter_statistics(&mut self) -> Result<()> {
        self.stats_interval_data = self.db.get_review_stats_by_interval()?;
        self.stats_daily_data = self.db.get_daily_review_counts(30)?;
        self.current_screen = CurrentScreen::Statistics;
        Ok(())
    }
}
