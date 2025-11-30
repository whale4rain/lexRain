use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use chrono::{DateTime, Utc};
use crate::models::{Word, LearningLog, LearningStatus};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn initialize() -> Result<Self> {
        let conn = Connection::open("lexrain.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS words (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                spelling TEXT NOT NULL UNIQUE,
                phonetic TEXT,
                definition TEXT,
                chinese_definition TEXT,
                tags TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS learning_log (
                word_id INTEGER PRIMARY KEY,
                repetition INTEGER NOT NULL,
                interval INTEGER NOT NULL,
                e_factor REAL NOT NULL,
                next_review TEXT NOT NULL,
                status INTEGER NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS review_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word_id INTEGER NOT NULL,
                reviewed_at TEXT NOT NULL,
                quality INTEGER NOT NULL,
                repetition INTEGER NOT NULL,
                interval INTEGER NOT NULL,
                e_factor REAL NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id)
            )",
            [],
        )?;

        // Migration: Add chinese_definition column if it doesn't exist
        let column_exists: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('words') WHERE name='chinese_definition'",
            [],
            |row| row.get(0),
        );

        if let Ok(0) = column_exists {
            conn.execute(
                "ALTER TABLE words ADD COLUMN chinese_definition TEXT",
                [],
            )?;
        }

        Ok(Self { conn })
    }

    pub fn add_word(&self, word: &Word) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO words (spelling, phonetic, definition, chinese_definition, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![word.spelling, word.phonetic, word.definition, word.chinese_definition, word.tags],
        )?;
        Ok(())
    }

    pub fn get_word_id(&self, spelling: &str) -> Result<Option<i64>> {
        let id = self.conn.query_row(
            "SELECT id FROM words WHERE spelling = ?1",
            params![spelling],
            |row| row.get(0),
        ).optional()?;
        Ok(id)
    }

    pub fn get_due_reviews(&self) -> Result<Vec<(Word, LearningLog)>> {
        let now = Utc::now();
        let mut stmt = self.conn.prepare(
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.chinese_definition, w.tags,
                    l.repetition, l.interval, l.e_factor, l.next_review, l.status
             FROM words w
             JOIN learning_log l ON w.id = l.word_id
             WHERE l.next_review <= ?1
             ORDER BY l.next_review ASC"
        )?;

        let rows = stmt.query_map(params![now.to_rfc3339()], |row| {
            let word = Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get(3)?,
                chinese_definition: row.get(4)?,
                tags: row.get(5)?,
            };

            let next_review_str: String = row.get(9)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id: word.id.unwrap(),
                repetition: row.get(6)?,
                interval: row.get(7)?,
                e_factor: row.get(8)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(10)?),
            };
            Ok((word, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Initialize a learning log for a new word if it doesn't exist
    pub fn init_learning_log(&self, word_id: i64) -> Result<()> {
        let exists: Option<i64> = self.conn.query_row(
            "SELECT 1 FROM learning_log WHERE word_id = ?1",
            params![word_id],
            |row| row.get(0),
        ).optional()?;

        if exists.is_none() {
            self.conn.execute(
                "INSERT INTO learning_log (word_id, repetition, interval, e_factor, next_review, status)
                 VALUES (?1, 0, 0, 2.5, ?2, 0)",
                params![word_id, Utc::now().to_rfc3339()],
            )?;
        }
        Ok(())
    }

    pub fn update_log(&self, log: &LearningLog) -> Result<()> {
        self.conn.execute(
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
        // Total words
        let total: i64 = self.conn.query_row("SELECT COUNT(*) FROM words", [], |r| r.get(0))?;
        // Mastered
        let mastered: i64 = self.conn.query_row("SELECT COUNT(*) FROM learning_log WHERE status = 2", [], |r| r.get(0))?;
        // Due today (approximate)
        let now = Utc::now().to_rfc3339();
        let due: i64 = self.conn.query_row("SELECT COUNT(*) FROM learning_log WHERE next_review <= ?1", params![now], |r| r.get(0))?;

        Ok((total, mastered, due))
    }

    // Get all words with their learning status
    pub fn get_all_words(&self) -> Result<Vec<(Word, Option<LearningLog>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.chinese_definition, w.tags,
                    l.repetition, l.interval, l.e_factor, l.next_review, l.status
             FROM words w
             LEFT JOIN learning_log l ON w.id = l.word_id
             ORDER BY w.spelling ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            let word = Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get(3)?,
                chinese_definition: row.get(4)?,
                tags: row.get(5)?,
            };

            let log = if let Ok(rep) = row.get::<_, i32>(6) {
                let next_review_str: String = row.get(9)?;
                let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                Some(LearningLog {
                    word_id: word.id.unwrap(),
                    repetition: rep,
                    interval: row.get(7)?,
                    e_factor: row.get(8)?,
                    next_review,
                    status: LearningStatus::from(row.get::<_, i32>(10)?),
                })
            } else {
                None
            };

            Ok((word, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Search words by spelling or definition
    pub fn search_words(&self, query: &str) -> Result<Vec<(Word, Option<LearningLog>)>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.chinese_definition, w.tags,
                    l.repetition, l.interval, l.e_factor, l.next_review, l.status
             FROM words w
             LEFT JOIN learning_log l ON w.id = l.word_id
             WHERE w.spelling LIKE ?1 OR w.definition LIKE ?1 OR w.chinese_definition LIKE ?1
             ORDER BY w.spelling ASC"
        )?;

        let rows = stmt.query_map(params![search_pattern], |row| {
            let word = Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get(3)?,
                chinese_definition: row.get(4)?,
                tags: row.get(5)?,
            };

            let log = if let Ok(rep) = row.get::<_, i32>(6) {
                let next_review_str: String = row.get(9)?;
                let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                Some(LearningLog {
                    word_id: word.id.unwrap(),
                    repetition: rep,
                    interval: row.get(7)?,
                    e_factor: row.get(8)?,
                    next_review,
                    status: LearningStatus::from(row.get::<_, i32>(10)?),
                })
            } else {
                None
            };

            Ok((word, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Add review to history
    pub fn add_review_history(&self, word_id: i64, quality: u8, log: &LearningLog) -> Result<()> {
        self.conn.execute(
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
        let mut stmt = self.conn.prepare(
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.chinese_definition, w.tags,
                    h.reviewed_at, h.quality
             FROM review_history h
             JOIN words w ON h.word_id = w.id
             ORDER BY h.reviewed_at DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let word = Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get(3)?,
                chinese_definition: row.get(4)?,
                tags: row.get(5)?,
            };
            let reviewed_at: String = row.get(6)?;
            let quality: u8 = row.get(7)?;
            Ok((word, reviewed_at, quality))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // Get review statistics for forgetting curve
    pub fn get_review_stats_by_interval(&self) -> Result<Vec<(i32, f64, i64)>> {
        // Returns (interval_days, avg_quality, count)
        let mut stmt = self.conn.prepare(
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
        let mut stmt = self.conn.prepare(
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
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM review_history
             WHERE DATE(reviewed_at) = DATE('now')",
            [],
            |r| r.get(0)
        )?;
        Ok(count)
    }

    // Get new words to learn (words with status = New, limit by count)
    pub fn get_new_words_to_learn(&self, limit: i64) -> Result<Vec<(Word, LearningLog)>> {
        let mut stmt = self.conn.prepare(
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.chinese_definition, w.tags,
                    l.repetition, l.interval, l.e_factor, l.next_review, l.status
             FROM words w
             JOIN learning_log l ON w.id = l.word_id
             WHERE l.status = 0
             ORDER BY w.id ASC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let word = Word {
                id: Some(row.get(0)?),
                spelling: row.get(1)?,
                phonetic: row.get(2)?,
                definition: row.get(3)?,
                chinese_definition: row.get(4)?,
                tags: row.get(5)?,
            };

            let next_review_str: String = row.get(9)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id: word.id.unwrap(),
                repetition: row.get(6)?,
                interval: row.get(7)?,
                e_factor: row.get(8)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(10)?),
            };
            Ok((word, log))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
