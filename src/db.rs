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

        Ok(Self { conn })
    }

    pub fn add_word(&self, word: &Word) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO words (spelling, phonetic, definition, tags) VALUES (?1, ?2, ?3, ?4)",
            params![word.spelling, word.phonetic, word.definition, word.tags],
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
            "SELECT w.id, w.spelling, w.phonetic, w.definition, w.tags,
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
                tags: row.get(4)?,
            };
            
            let next_review_str: String = row.get(8)?;
            let next_review = DateTime::parse_from_rfc3339(&next_review_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now());

            let log = LearningLog {
                word_id: word.id.unwrap(),
                repetition: row.get(5)?,
                interval: row.get(6)?,
                e_factor: row.get(7)?,
                next_review,
                status: LearningStatus::from(row.get::<_, i32>(9)?),
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
}
