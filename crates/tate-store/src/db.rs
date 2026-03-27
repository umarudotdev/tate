use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use tate_core::card::CardRow;

use crate::error::StorageError;

const SCHEMA_VERSION: &str = "3";

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cards (
    entry      TEXT PRIMARY KEY,
    ease       REAL NOT NULL DEFAULT 2.5,
    interval   INTEGER NOT NULL DEFAULT 0,
    due        TEXT NOT NULL,
    reps       INTEGER NOT NULL DEFAULT 0,
    lapses     INTEGER NOT NULL DEFAULT 0,
    added      TEXT NOT NULL,
    retired    INTEGER NOT NULL DEFAULT 0,
    body_hash  TEXT
);

CREATE TABLE IF NOT EXISTS reviews (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry       TEXT NOT NULL,
    reviewed_at TEXT NOT NULL,
    grade       INTEGER NOT NULL,
    FOREIGN KEY (entry) REFERENCES cards(entry)
);

CREATE TABLE IF NOT EXISTS questions (
    entry      TEXT PRIMARY KEY,
    body_hash  TEXT NOT NULL,
    question   TEXT NOT NULL,
    answer     TEXT,
    source_text TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (entry) REFERENCES cards(entry)
);

CREATE INDEX IF NOT EXISTS idx_reviews_entry ON reviews(entry);
CREATE INDEX IF NOT EXISTS idx_cards_due ON cards(due) WHERE retired = 0;
";

#[derive(Debug)]
pub struct QuestionRow {
    pub body_hash: String,
    pub question: String,
    pub answer: Option<String>,
    pub source_text: Option<String>,
}

#[derive(Debug)]
pub struct ReviewRow {
    pub entry: String,
    pub reviewed_at: String,
    pub grade: u8,
}

#[derive(Debug, Default)]
pub struct CardCounts {
    pub new: u32,
    pub learning: u32,
    pub mature: u32,
    pub retired: u32,
}

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path).map_err(StorageError::Write)?;
        let store = SqliteStore { conn };
        store.ensure_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory().map_err(StorageError::Write)?;
        let store = SqliteStore { conn };
        store.ensure_schema()?;
        Ok(store)
    }

    fn ensure_schema(&self) -> Result<(), StorageError> {
        let has_meta: bool = self
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='meta'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(StorageError::Read)?
            > 0;

        if has_meta {
            let version: Option<String> = self
                .conn
                .query_row(
                    "SELECT value FROM meta WHERE key = 'schema_version'",
                    [],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StorageError::Read)?;

            if version.as_deref() == Some(SCHEMA_VERSION) {
                return Ok(());
            }

            tracing::warn!(
                expected = SCHEMA_VERSION,
                found = ?version,
                "schema version mismatch, recreating database"
            );
            self.drop_all()?;
        }

        self.create_schema()
    }

    fn drop_all(&self) -> Result<(), StorageError> {
        self.conn
            .execute_batch(
                "DROP TABLE IF EXISTS reviews;
                 DROP TABLE IF EXISTS questions;
                 DROP TABLE IF EXISTS cards;
                 DROP TABLE IF EXISTS meta;
                 DROP INDEX IF EXISTS idx_reviews_entry;
                 DROP INDEX IF EXISTS idx_cards_due;",
            )
            .map_err(StorageError::Write)
    }

    fn create_schema(&self) -> Result<(), StorageError> {
        self.conn
            .execute_batch(SCHEMA_SQL)
            .map_err(StorageError::Write)?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }
    pub fn save_card(&self, row: &CardRow) -> Result<(), StorageError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO cards (entry, ease, interval, due, reps, lapses, added, retired, body_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    row.entry,
                    row.ease,
                    row.interval,
                    row.due,
                    row.reps,
                    row.lapses,
                    row.added,
                    row.retired as i32,
                    row.body_hash,
                ],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }

    pub fn get_card(&self, entry: &str) -> Result<Option<CardRow>, StorageError> {
        self.conn
            .query_row(
                "SELECT entry, ease, interval, due, reps, lapses, added, retired, body_hash
                 FROM cards WHERE entry = ?1",
                params![entry],
                row_to_card_row,
            )
            .optional()
            .map_err(StorageError::Read)
    }

    pub fn get_all_cards(&self) -> Result<Vec<CardRow>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT entry, ease, interval, due, reps, lapses, added, retired, body_hash FROM cards",
            )
            .map_err(StorageError::Read)?;

        let rows = stmt
            .query_map([], row_to_card_row)
            .map_err(StorageError::Read)?;

        let mut cards = Vec::new();
        for r in rows {
            cards.push(r.map_err(StorageError::Read)?);
        }
        Ok(cards)
    }

    pub fn due_cards(&self, today: &str) -> Result<Vec<CardRow>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT entry, ease, interval, due, reps, lapses, added, retired, body_hash
                 FROM cards WHERE due <= ?1 AND retired = 0 ORDER BY due ASC",
            )
            .map_err(StorageError::Read)?;

        let rows = stmt
            .query_map(params![today], row_to_card_row)
            .map_err(StorageError::Read)?;

        let mut cards = Vec::new();
        for r in rows {
            cards.push(r.map_err(StorageError::Read)?);
        }
        Ok(cards)
    }

    pub fn retire_card(&self, entry: &str) -> Result<(), StorageError> {
        self.conn
            .execute(
                "UPDATE cards SET retired = 1 WHERE entry = ?1",
                params![entry],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }

    pub fn update_body_hash(&self, entry: &str, hash: Option<&str>) -> Result<(), StorageError> {
        self.conn
            .execute(
                "UPDATE cards SET body_hash = ?1 WHERE entry = ?2",
                params![hash, entry],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }
    pub fn save_question(
        &self,
        entry: &str,
        body_hash: &str,
        question: &str,
        answer: Option<&str>,
        source_text: Option<&str>,
    ) -> Result<(), StorageError> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO questions (entry, body_hash, question, answer, source_text, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![entry, body_hash, question, answer, source_text, now],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }

    pub fn get_question(&self, entry: &str) -> Result<Option<QuestionRow>, StorageError> {
        self.conn
            .query_row(
                "SELECT body_hash, question, answer, source_text FROM questions WHERE entry = ?1",
                params![entry],
                |row| {
                    Ok(QuestionRow {
                        body_hash: row.get(0)?,
                        question: row.get(1)?,
                        answer: row.get(2)?,
                        source_text: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(StorageError::Read)
    }
    pub fn save_review(&self, entry: &str, grade: u8) -> Result<(), StorageError> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO reviews (entry, reviewed_at, grade) VALUES (?1, ?2, ?3)",
                params![entry, now, grade],
            )
            .map_err(StorageError::Write)?;
        Ok(())
    }

    pub fn streak(&self) -> Result<u32, StorageError> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT date(reviewed_at) as d FROM reviews ORDER BY d DESC")
            .map_err(StorageError::Read)?;

        let dates: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(StorageError::Read)?
            .filter_map(|r| r.ok())
            .collect();

        if dates.is_empty() {
            return Ok(0);
        }

        let mut streak = 0u32;
        let mut expected = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());

        for date_str in &dates {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if date == expected {
                    streak += 1;
                    expected -= chrono::TimeDelta::days(1);
                } else if date == expected + chrono::TimeDelta::days(1) {
                    if streak == 0 {
                        expected = date;
                        streak = 1;
                        expected -= chrono::TimeDelta::days(1);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        Ok(streak)
    }

    pub fn card_counts(&self) -> Result<CardCounts, StorageError> {
        let mut counts = CardCounts::default();

        let mut stmt = self
            .conn
            .prepare("SELECT reps, interval, retired FROM cards")
            .map_err(StorageError::Read)?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, bool>(2)?,
                ))
            })
            .map_err(StorageError::Read)?;

        for r in rows {
            let (reps, interval, retired) = r.map_err(StorageError::Read)?;
            if retired {
                counts.retired += 1;
            } else if reps == 0 {
                counts.new += 1;
            } else if interval >= 21 {
                counts.mature += 1;
            } else {
                counts.learning += 1;
            }
        }

        Ok(counts)
    }

    pub fn execute_in_transaction<F>(&mut self, f: F) -> Result<(), StorageError>
    where
        F: FnOnce(&Connection) -> Result<(), StorageError>,
    {
        let tx = self.conn.transaction().map_err(StorageError::Write)?;
        f(&tx)?;
        tx.commit().map_err(StorageError::Write)
    }
}

fn row_to_card_row(row: &rusqlite::Row) -> rusqlite::Result<CardRow> {
    Ok(CardRow {
        entry: row.get(0)?,
        ease: row.get(1)?,
        interval: row.get(2)?,
        due: row.get(3)?,
        reps: row.get(4)?,
        lapses: row.get(5)?,
        added: row.get(6)?,
        retired: row.get::<_, i32>(7)? != 0,
        body_hash: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> SqliteStore {
        SqliteStore::open_in_memory().unwrap()
    }

    fn sample_card(entry: &str) -> CardRow {
        CardRow {
            entry: entry.to_string(),
            ease: 2.5,
            interval: 0,
            due: "2026-03-25".to_string(),
            reps: 0,
            lapses: 0,
            added: "2026-03-20".to_string(),
            retired: false,
            body_hash: None,
        }
    }

    #[test]
    fn save_and_get_card() {
        let s = store();
        let card = sample_card("src/main.rs");
        s.save_card(&card).unwrap();

        let loaded = s.get_card("src/main.rs").unwrap().unwrap();
        assert_eq!(loaded.entry, "src/main.rs");
        assert_eq!(loaded.ease, 2.5);
        assert!(!loaded.retired);
    }

    #[test]
    fn get_missing_card_returns_none() {
        let s = store();
        assert!(s.get_card("nonexistent").unwrap().is_none());
    }

    #[test]
    fn due_cards_query() {
        let s = store();
        s.save_card(&CardRow {
            due: "2026-03-25".to_string(),
            ..sample_card("a.rs")
        })
        .unwrap();
        s.save_card(&CardRow {
            due: "2026-03-30".to_string(),
            ..sample_card("b.rs")
        })
        .unwrap();
        s.save_card(&CardRow {
            retired: true,
            ..sample_card("c.rs")
        })
        .unwrap();

        let due = s.due_cards("2026-03-25").unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].entry, "a.rs");
    }

    #[test]
    fn retire_card() {
        let s = store();
        s.save_card(&sample_card("a.rs")).unwrap();
        s.retire_card("a.rs").unwrap();

        let card = s.get_card("a.rs").unwrap().unwrap();
        assert!(card.retired);
    }

    #[test]
    fn question_crud() {
        let s = store();
        s.save_card(&sample_card("a.rs")).unwrap();
        s.save_question(
            "a.rs",
            "hash123",
            "What does this do?",
            Some("It does things."),
            None,
        )
        .unwrap();

        let q = s.get_question("a.rs").unwrap().unwrap();
        assert_eq!(q.body_hash, "hash123");
        assert_eq!(q.question, "What does this do?");
        assert_eq!(q.answer.as_deref(), Some("It does things."));
    }

    #[test]
    fn question_without_answer() {
        let s = store();
        s.save_card(&sample_card("a.rs")).unwrap();
        s.save_question("a.rs", "hash123", "What does this do?", None, None)
            .unwrap();

        let q = s.get_question("a.rs").unwrap().unwrap();
        assert!(q.answer.is_none());
    }

    #[test]
    fn question_with_source_text() {
        let s = store();
        s.save_card(&sample_card("styles.css:2-5")).unwrap();
        s.save_question(
            "styles.css:2-5",
            "hash456",
            "What styles?",
            None,
            Some("body {\n  color: red;\n}"),
        )
        .unwrap();

        let q = s.get_question("styles.css:2-5").unwrap().unwrap();
        assert_eq!(q.source_text.as_deref(), Some("body {\n  color: red;\n}"));
    }

    #[test]
    fn question_missing_returns_none() {
        let s = store();
        assert!(s.get_question("nonexistent").unwrap().is_none());
    }

    #[test]
    fn review_and_card_counts() {
        let s = store();
        s.save_card(&sample_card("new.rs")).unwrap();
        s.save_card(&CardRow {
            reps: 2,
            interval: 6,
            ..sample_card("learning.rs")
        })
        .unwrap();
        s.save_card(&CardRow {
            reps: 5,
            interval: 30,
            ..sample_card("mature.rs")
        })
        .unwrap();
        s.save_card(&CardRow {
            retired: true,
            ..sample_card("retired.rs")
        })
        .unwrap();

        let counts = s.card_counts().unwrap();
        assert_eq!(counts.new, 1);
        assert_eq!(counts.learning, 1);
        assert_eq!(counts.mature, 1);
        assert_eq!(counts.retired, 1);
    }

    #[test]
    fn due_cards_returns_sorted_by_date() {
        let s = store();
        s.save_card(&CardRow {
            due: "2026-03-24".to_string(),
            ..sample_card("late.rs")
        })
        .unwrap();
        s.save_card(&CardRow {
            due: "2026-03-20".to_string(),
            ..sample_card("early.rs")
        })
        .unwrap();

        let due = s.due_cards("2026-03-25").unwrap();
        assert_eq!(due.len(), 2);
        assert_eq!(
            due[0].entry, "early.rs",
            "earliest due date should come first"
        );
        assert_eq!(due[1].entry, "late.rs");
    }

    #[test]
    fn card_counts_empty_db() {
        let s = store();
        let counts = s.card_counts().unwrap();
        assert_eq!(counts.new, 0);
        assert_eq!(counts.learning, 0);
        assert_eq!(counts.mature, 0);
        assert_eq!(counts.retired, 0);
    }

    #[test]
    fn streak_no_reviews_is_zero() {
        let s = store();
        assert_eq!(s.streak().unwrap(), 0);
    }

    #[test]
    fn body_hash_update() {
        let s = store();
        s.save_card(&sample_card("a.rs")).unwrap();
        s.update_body_hash("a.rs", Some("newhash")).unwrap();

        let card = s.get_card("a.rs").unwrap().unwrap();
        assert_eq!(card.body_hash.as_deref(), Some("newhash"));
    }

    #[test]
    fn schema_version_mismatch_recreates() {
        let s = store();
        s.save_card(&sample_card("a.rs")).unwrap();

        s.conn
            .execute(
                "UPDATE meta SET value = '0' WHERE key = 'schema_version'",
                [],
            )
            .unwrap();

        s.ensure_schema().unwrap();

        assert!(s.get_card("a.rs").unwrap().is_none());
    }
}
