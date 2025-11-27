use chrono::{Duration, Utc};
use crate::models::{LearningLog, LearningStatus};

/// SuperMemo-2 Algorithm Implementation
///
/// # Parameters
/// - quality: 0-5 rating
///   - 0-2: Forgot
///   - 3-5: Remembered
///
/// # Returns
/// Updated (repetition, interval, e_factor)
pub fn update_memory_state(
    current_repetition: i32,
    current_ef: f64,
    quality: u8,
) -> (i32, i32, f64) {
    let mut next_repetition = current_repetition;
    let next_interval;

    if quality >= 3 {
        if current_repetition == 0 {
            next_interval = 1;
        } else if current_repetition == 1 {
            next_interval = 6;
        } else {
            next_interval = (current_repetition as f64 * current_ef).round() as i32;
        }
        next_repetition += 1;
    } else {
        next_repetition = 0;
        next_interval = 1;
    }

    // EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
    // EF cannot go below 1.3
    let mut next_ef = current_ef + (0.1 - (5.0 - quality as f64) * (0.08 + (5.0 - quality as f64) * 0.02));
    if next_ef < 1.3 {
        next_ef = 1.3;
    }

    (next_repetition, next_interval, next_ef)
}

pub fn process_review(log: &mut LearningLog, quality: u8) {
    let (n, i, ef) = update_memory_state(log.repetition, log.e_factor, quality);
    
    log.repetition = n;
    log.interval = i;
    log.e_factor = ef;
    log.next_review = Utc::now() + Duration::days(i as i64);
    
    if quality >= 3 {
        // Simple logic: if interval > 21 days, consider mastered for now, or just keep as Learning
        if i > 21 {
            log.status = LearningStatus::Mastered;
        } else {
            log.status = LearningStatus::Learning;
        }
    } else {
        log.status = LearningStatus::Learning; // Reset to learning if forgot
    }
}
