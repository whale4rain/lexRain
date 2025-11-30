use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: Option<i64>,
    pub spelling: String,
    pub phonetic: Option<String>,
    pub definition: String, // JSON string or simple text
    pub chinese_definition: Option<String>, // Chinese translation
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLog {
    pub word_id: i64,
    pub repetition: i32,     // n
    pub interval: i32,       // I (days)
    pub e_factor: f64,       // EF
    pub next_review: DateTime<Utc>,
    pub status: LearningStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningStatus {
    New = 0,
    Learning = 1,
    Mastered = 2,
}

impl From<i32> for LearningStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => LearningStatus::Learning,
            2 => LearningStatus::Mastered,
            _ => LearningStatus::New,
        }
    }
}

impl From<LearningStatus> for i32 {
    fn from(status: LearningStatus) -> Self {
        status as i32
    }
}
