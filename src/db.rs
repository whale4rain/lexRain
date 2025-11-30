use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use chrono::{DateTime, Utc};
use crate::models::{Word, LearningLog, LearningStatus};

pub struct Database {
    dict_conn: Connection,  // Read-only ECDICT database
    learn_conn: Connection, // Learning progress database
}

impl Database {
    pub fn initialize() -> Result<Self> {
        // Open ECDICT dictionary database (read-only)
        let dict_conn = Connection::open("ecdict-sqlite-28/stardict.db")?;
        
        // Open learning progress database
        let learn_conn = Connection::open("lexrain_progress.db")?;

        // Create learning log table (word_id references ECDICT stardict.id)
        learn_conn.execute(
            "CREATE TABLE IF NOT EXISTS learning_log (
                word_id INTEGER PRIMARY KEY,
                repetition INTEGER NOT NULL,
                interval INTEGER NOT NULL,
                e_factor REAL NOT NULL,
                next_review TEXT NOT NULL,
                status INTEGER NOT NULL
            )",
            [],
        )?;

        learn_conn.execute(
            "CREATE TABLE IF NOT EXISTS review_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word_id INTEGER NOT NULL,
                reviewed_at TEXT NOT NULL,
                quality INTEGER NOT NULL,
                repetition INTEGER NOT NULL,
                interval INTEGER NOT NULL,
                e_factor REAL NOT NULL
            )",
            [],
        )?;

        Ok(Self { dict_conn, learn_conn })
    }

    // Get word by ID from ECDICT
    fn get_word_by_id(&self, id: i64) -> Result<Word> {
        Ok(self.dict_conn.query_row(
            "SELECT id, word, phonetic, definition, translation, pos, collins, oxford, tag, bnc, frq, exchange
             FROM stardict WHERE id = ?1",
            params![id],
            |row| {
                Ok(Word {
                    id: Some(row.get(0)?),
                    spelling: row.get(1)?,
                    phonetic: row.get(2)?,
                    definition: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    translation: row.get(4)?,
                    pos: row.get(5)?,
                    collins: row.get::<_, Option<i32>>(6)?.unwrap_or(0),
                    oxford: row.get::<_, Option<i32>>(7)?.unwrap_or(0) > 0,
                    tag: row.get(8)?,
                    bnc: row.get(9)?,
                    frq: row.get(10)?,
                    exchange: row.get(11)?,
                })
            },
        )?)
    }

    pub fn get_due_reviews(&self) -> Result<Vec<(Word, LearningLog)>> {
        let now = Utc::now();
        let mut stmt = self.learn_conn.prepare(
            "SELECT word_id, repetition, interval, e_factor, next_review, status
             FROM learning_log
             WHERE next_review <= ?1
             ORDER BY next_review ASC"
        )?;

        let rows = stmt.query_map(params![now.to_rfc3339()], |row| {
            let word_id: i64 = row.get(0)?;
            let next_review_str: String = row.get(4)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id,
                repetition: row.get(1)?,
                interval: row.get(2)?,
                e_factor: row.get(3)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(5)?),
            };
            Ok((word_id, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (word_id, log) = row?;
            if let Ok(word) = self.get_word_by_id(word_id) {
                results.push((word, log));
            }
        }
        Ok(results)
    }

    // Initialize a learning log for a new word if it doesn't exist
    pub fn init_learning_log(&self, word_id: i64) -> Result<()> {
        let exists: Option<i64> = self.learn_conn.query_row(
            "SELECT 1 FROM learning_log WHERE word_id = ?1",
            params![word_id],
            |row| row.get(0),
        ).optional()?;

        if exists.is_none() {
            self.learn_conn.execute(
                "INSERT INTO learning_log (word_id, repetition, interval, e_factor, next_review, status)
                 VALUES (?1, 0, 0, 2.5, ?2, 0)",
                params![word_id, Utc::now().to_rfc3339()],
            )?;
        }
        Ok(())
    }

    pub fn update_log(&self, log: &LearningLog) -> Result<()> {
        self.learn_conn.execute(
            "UPDATE learning_log 
             SET repetition = ?1, interval = ?2, e_factor = ?3, next_review = ?4, status = ?5
             WHERE word_id = ?6",
            params![
                log.repetition,
                log.interval,
                log.e_factor,
                log.next_review.to_rfc3339(),
                i32::from(log.status),
                log.word_id
            ],
        )?;
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        // Total words with learning log
        let total: i64 = self.learn_conn.query_row("SELECT COUNT(*) FROM learning_log", [], |r| r.get(0))?;
        // Mastered
        let mastered: i64 = self.learn_conn.query_row("SELECT COUNT(*) FROM learning_log WHERE status = 2", [], |r| r.get(0))?;
        // Due today
        let now = Utc::now().to_rfc3339();
        let due: i64 = self.learn_conn.query_row("SELECT COUNT(*) FROM learning_log WHERE next_review <= ?1", params![now], |r| r.get(0))?;

        Ok((total, mastered, due))
    }

    // Get all words with their learning status (limit to words we're learning)
    pub fn get_all_words(&self) -> Result<Vec<(Word, Option<LearningLog>)>> {
        let mut stmt = self.learn_conn.prepare(
            "SELECT word_id, repetition, interval, e_factor, next_review, status
             FROM learning_log
             ORDER BY word_id ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            let word_id: i64 = row.get(0)?;
            let next_review_str: String = row.get(4)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id,
                repetition: row.get(1)?,
                interval: row.get(2)?,
                e_factor: row.get(3)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(5)?),
            };
            Ok((word_id, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (word_id, log) = row?;
            if let Ok(word) = self.get_word_by_id(word_id) {
                results.push((word, Some(log)));
            }
        }
        Ok(results)
    }

    // Search words in ECDICT dictionary
    pub fn search_words(&self, query: &str) -> Result<Vec<(Word, Option<LearningLog>)>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.dict_conn.prepare(
            "SELECT id, word, phonetic, definition, translation, pos, collins, oxford, tag, bnc, frq, exchange
             FROM stardict
             WHERE word LIKE ?1 OR translation LIKE ?1
             ORDER BY 
                CASE 
                    WHEN word = ?2 THEN 1
                    WHEN word LIKE ?2 || '%' THEN 2
                    ELSE 3
                END,
                collins DESC, oxford DESC, bnc ASC
             LIMIT 100"
        )?;

        let rows = stmt.query_map(params![search_pattern, query], |row| {
            Ok(Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                translation: row.get(4)?,
                pos: row.get(5)?,
                collins: row.get::<_, Option<i32>>(6)?.unwrap_or(0),
                oxford: row.get::<_, Option<i32>>(7)?.unwrap_or(0) > 0,
                tag: row.get(8)?,
                bnc: row.get(9)?,
                frq: row.get(10)?,
                exchange: row.get(11)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            let word = row?;
            // Check if this word has a learning log
            let log = if let Some(word_id) = word.id {
                self.get_learning_log(word_id)?
            } else {
                None
            };
            results.push((word, log));
        }
        Ok(results)
    }

    // Get learning log for a word
    fn get_learning_log(&self, word_id: i64) -> Result<Option<LearningLog>> {
        let log = self.learn_conn.query_row(
            "SELECT word_id, repetition, interval, e_factor, next_review, status
             FROM learning_log WHERE word_id = ?1",
            params![word_id],
            |row| {
                let next_review_str: String = row.get(4)?;
                let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                Ok(LearningLog {
                    word_id: row.get(0)?,
                    repetition: row.get(1)?,
                    interval: row.get(2)?,
                    e_factor: row.get(3)?,
                    next_review,
                    status: LearningStatus::from(row.get::<_, i32>(5)?),
                })
            },
        ).optional()?;
        Ok(log)
    }

    // Add review to history
    pub fn add_review_history(&self, word_id: i64, quality: u8, log: &LearningLog) -> Result<()> {
        self.learn_conn.execute(
            "INSERT INTO review_history (word_id, reviewed_at, quality, repetition, interval, e_factor)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                word_id,
                Utc::now().to_rfc3339(),
                quality,
                log.repetition,
                log.interval,
                log.e_factor
            ],
        )?;
        Ok(())
    }

    // Get recent review history
    pub fn get_recent_reviews(&self, limit: i64) -> Result<Vec<(Word, String, u8)>> {
        let mut stmt = self.learn_conn.prepare(
            "SELECT word_id, reviewed_at, quality
             FROM review_history
             ORDER BY reviewed_at DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let word_id: i64 = row.get(0)?;
            let reviewed_at: String = row.get(1)?;
            let quality: u8 = row.get(2)?;
            Ok((word_id, reviewed_at, quality))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (word_id, reviewed_at, quality) = row?;
            if let Ok(word) = self.get_word_by_id(word_id) {
                results.push((word, reviewed_at, quality));
            }
        }
        Ok(results)
    }

    // Get review statistics for forgetting curve
    pub fn get_review_stats_by_interval(&self) -> Result<Vec<(i32, f64, i64)>> {
        let mut stmt = self.learn_conn.prepare(
            "SELECT interval, AVG(quality) as avg_quality, COUNT(*) as count
             FROM review_history
             GROUP BY interval
             ORDER BY interval ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Get daily review count for the last N days
    pub fn get_daily_review_counts(&self, days: i64) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.learn_conn.prepare(
            "SELECT DATE(reviewed_at) as review_date, COUNT(*) as count
             FROM review_history
             WHERE reviewed_at >= datetime('now', '-' || ?1 || ' days')
             GROUP BY review_date
             ORDER BY review_date ASC"
        )?;

        let rows = stmt.query_map(params![days], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Get today's completed review count
    pub fn get_today_completed_count(&self) -> Result<i64> {
        let count: i64 = self.learn_conn.query_row(
            "SELECT COUNT(*) FROM review_history
             WHERE DATE(reviewed_at) = DATE('now')",
            [],
            |r| r.get(0)
        )?;
        Ok(count)
    }

    // Get new words to learn from high-quality ECDICT entries
    // Prioritize: oxford 3000, high collins rating, common words by frequency
    pub fn get_new_words_to_learn(&self, limit: i64) -> Result<Vec<(Word, LearningLog)>> {
        // First, check if we have enough words with status = 0
        let new_count: i64 = self.learn_conn.query_row(
            "SELECT COUNT(*) FROM learning_log WHERE status = 0",
            [],
            |r| r.get(0)
        )?;

        if new_count >= limit {
            // Return existing new words
            let mut stmt = self.learn_conn.prepare(
                "SELECT word_id, repetition, interval, e_factor, next_review, status
                 FROM learning_log
                 WHERE status = 0
                 ORDER BY word_id ASC
                 LIMIT ?1"
            )?;

            let rows = stmt.query_map(params![limit], |row| {
                let word_id: i64 = row.get(0)?;
                let next_review_str: String = row.get(4)?;
                let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                let log = LearningLog {
                    word_id,
                    repetition: row.get(1)?,
                    interval: row.get(2)?,
                    e_factor: row.get(3)?,
                    next_review,
                    status: LearningStatus::from(row.get::<_, i32>(5)?),
                };
                Ok((word_id, log))
            })?;

            let mut results = Vec::new();
            for row in rows {
                let (word_id, log) = row?;
                if let Ok(word) = self.get_word_by_id(word_id) {
                    results.push((word, log));
                }
            }
            return Ok(results);
        }

        // Need to add new words from ECDICT
        let needed = limit - new_count;
        
        // Get existing word IDs
        let mut existing_ids = Vec::new();
        let mut stmt = self.learn_conn.prepare("SELECT word_id FROM learning_log")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        for row in rows {
            existing_ids.push(row?);
        }

        // Query ECDICT for high-quality words not yet in learning_log
        let placeholders = if existing_ids.is_empty() {
            String::from("0")
        } else {
            existing_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
        };
        
        let query = format!(
            "SELECT id, word, phonetic, definition, translation, pos, collins, oxford, tag, bnc, frq, exchange
             FROM stardict
             WHERE id NOT IN ({})
             AND translation IS NOT NULL
             AND LENGTH(word) > 1
             AND word NOT LIKE '%-%'
             AND word NOT LIKE '% %'
             ORDER BY 
                oxford DESC,
                collins DESC,
                CASE WHEN bnc IS NOT NULL THEN bnc ELSE 999999 END ASC,
                CASE WHEN frq IS NOT NULL THEN frq ELSE 999999 END ASC
             LIMIT ?1",
            placeholders
        );

        let mut stmt = self.dict_conn.prepare(&query)?;
        let rows = stmt.query_map(params![needed], |row| {
            Ok(Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                translation: row.get(4)?,
                pos: row.get(5)?,
                collins: row.get::<_, Option<i32>>(6)?.unwrap_or(0),
                oxford: row.get::<_, Option<i32>>(7)?.unwrap_or(0) > 0,
                tag: row.get(8)?,
                bnc: row.get(9)?,
                frq: row.get(10)?,
                exchange: row.get(11)?,
            })
        })?;

        // Add these words to learning_log and return them
        let mut results = Vec::new();
        for row in rows {
            let word = row?;
            if let Some(word_id) = word.id {
                self.init_learning_log(word_id)?;
                if let Ok(Some(log)) = self.get_learning_log(word_id) {
                    results.push((word, log));
                }
            }
        }

        // Also get existing new words
        let mut stmt = self.learn_conn.prepare(
            "SELECT word_id, repetition, interval, e_factor, next_review, status
             FROM learning_log
             WHERE status = 0
             ORDER BY word_id ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            let word_id: i64 = row.get(0)?;
            let next_review_str: String = row.get(4)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id,
                repetition: row.get(1)?,
                interval: row.get(2)?,
                e_factor: row.get(3)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(5)?),
            };
            Ok((word_id, log))
        })?;

        for row in rows {
            let (word_id, log) = row?;
            if let Ok(word) = self.get_word_by_id(word_id) {
                results.push((word, log));
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        Ok(results)
    }
}
