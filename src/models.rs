use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: Option<i64>,
    pub spelling: String,
    pub phonetic: Option<String>,
    pub definition: String,
    pub translation: Option<String>, // Chinese translation from ECDICT
    pub pos: Option<String>,         // Part of speech
    pub collins: i32,                // Collins star rating (0-5)
    pub oxford: bool,                // Oxford 3000 core word
    pub tag: Option<String>,         // Tags: zk/gk/cet4/cet6/ky/toefl/ielts/gre
    pub bnc: Option<i32>,           // BNC corpus frequency rank
    pub frq: Option<i32>,           // Contemporary corpus frequency rank
    pub exchange: Option<String>,    // Word forms (tenses, plural, etc.)
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
