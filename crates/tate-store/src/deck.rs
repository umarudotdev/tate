use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use tate_core::card::CardRow;

use crate::db::SqliteStore;
use crate::error::{DeckFileError, StorageError};

pub struct DeckFile {
    path: PathBuf,
}

impl DeckFile {
    pub fn new(path: PathBuf) -> Self {
        DeckFile { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self) -> Result<Vec<String>, DeckFileError> {
        if !self.path.exists() {
            return Err(DeckFileError::NotFound(self.path.clone()));
        }
        let content = fs::read_to_string(&self.path)?;
        Ok(parse_deck_lines(&content))
    }

    pub fn append(&self, entry: &str) -> Result<(), DeckFileError> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", entry)?;
        Ok(())
    }

    pub fn remove(&self, entry: &str) -> Result<(), DeckFileError> {
        let content = fs::read_to_string(&self.path)?;
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| line.trim() != entry)
            .collect();
        self.atomic_write(&filtered.join("\n"))
    }

    pub fn write_all(&self, entries: &[String]) -> Result<(), DeckFileError> {
        let content = entries.join("\n");
        self.atomic_write(&content)
    }

    fn atomic_write(&self, content: &str) -> Result<(), DeckFileError> {
        let tmp = self.path.with_extension("tmp");
        let mut final_content = content.to_string();
        if !final_content.is_empty() && !final_content.ends_with('\n') {
            final_content.push('\n');
        }
        fs::write(&tmp, &final_content)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

pub fn parse_deck_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

pub fn sync_deck(store: &SqliteStore, entries: &[String], today: &str) -> Result<(), StorageError> {
    let deck_set: HashSet<&str> = entries.iter().map(|s| s.as_str()).collect();

    let existing_cards = store.get_all_cards()?;
    let db_set: HashSet<String> = existing_cards.iter().map(|c| c.entry.clone()).collect();

    for entry in entries {
        if !db_set.contains(entry.as_str()) {
            store.save_card(&CardRow {
                entry: entry.clone(),
                ease: 2.5,
                interval: 0,
                due: today.to_string(),
                reps: 0,
                lapses: 0,
                added: today.to_string(),
                retired: false,
                body_hash: None,
            })?;
        }
    }

    for card in &existing_cards {
        if !deck_set.contains(card.entry.as_str()) && !card.retired {
            store.retire_card(&card.entry)?;
        } else if deck_set.contains(card.entry.as_str()) && card.retired {
            store.save_card(&CardRow {
                entry: card.entry.clone(),
                ease: 2.5,
                interval: 0,
                due: today.to_string(),
                reps: 0,
                lapses: 0,
                added: card.added.clone(),
                retired: false,
                body_hash: None,
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_deck(content: &str) -> (DeckFile, tempfile::TempPath) {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", content).unwrap();
        let path = f.into_temp_path();
        let deck = DeckFile::new(path.to_path_buf());
        (deck, path)
    }

    #[test]
    fn parse_skips_blanks_and_comments() {
        let lines = parse_deck_lines("# comment\nsrc/a.rs\n\n  # another\nsrc/b.rs::foo\n");
        assert_eq!(lines, vec!["src/a.rs", "src/b.rs::foo"]);
    }

    #[test]
    fn read_deck_file() {
        let (deck, _path) = temp_deck("src/a.rs\n# comment\nsrc/b.rs\n");
        let entries = deck.read().unwrap();
        assert_eq!(entries, vec!["src/a.rs", "src/b.rs"]);
    }

    #[test]
    fn read_missing_file_is_error() {
        let deck = DeckFile::new(PathBuf::from("/nonexistent/deck"));
        assert!(deck.read().is_err());
    }

    #[test]
    fn append_entry() {
        let (deck, _path) = temp_deck("src/a.rs\n");
        deck.append("src/b.rs").unwrap();
        let entries = deck.read().unwrap();
        assert_eq!(entries, vec!["src/a.rs", "src/b.rs"]);
    }

    #[test]
    fn remove_entry() {
        let (deck, _path) = temp_deck("src/a.rs\nsrc/b.rs\nsrc/c.rs\n");
        deck.remove("src/b.rs").unwrap();
        let entries = deck.read().unwrap();
        assert_eq!(entries, vec!["src/a.rs", "src/c.rs"]);
    }

    #[test]
    fn sync_adds_new_entries() {
        let store = SqliteStore::open_in_memory().unwrap();
        sync_deck(&store, &["src/a.rs".to_string()], "2026-03-25").unwrap();

        let card = store.get_card("src/a.rs").unwrap().unwrap();
        assert_eq!(card.due, "2026-03-25");
        assert_eq!(card.reps, 0);
    }

    #[test]
    fn sync_retires_missing_entries() {
        let store = SqliteStore::open_in_memory().unwrap();
        store
            .save_card(&CardRow {
                entry: "src/old.rs".to_string(),
                ease: 2.5,
                interval: 0,
                due: "2026-03-25".to_string(),
                reps: 0,
                lapses: 0,
                added: "2026-03-20".to_string(),
                retired: false,
                body_hash: None,
            })
            .unwrap();

        sync_deck(&store, &[], "2026-03-25").unwrap();

        let card = store.get_card("src/old.rs").unwrap().unwrap();
        assert!(card.retired);
    }

    #[test]
    fn sync_resets_readded_entries() {
        let store = SqliteStore::open_in_memory().unwrap();
        store
            .save_card(&CardRow {
                entry: "src/a.rs".to_string(),
                ease: 1.5,
                interval: 30,
                due: "2026-04-25".to_string(),
                reps: 10,
                lapses: 3,
                added: "2026-01-01".to_string(),
                retired: true,
                body_hash: Some("oldhash".to_string()),
            })
            .unwrap();

        sync_deck(&store, &["src/a.rs".to_string()], "2026-03-25").unwrap();

        let card = store.get_card("src/a.rs").unwrap().unwrap();
        assert!(!card.retired);
        assert_eq!(card.ease, 2.5);
        assert_eq!(card.reps, 0);
        assert_eq!(card.lapses, 0);
        assert_eq!(card.due, "2026-03-25");
        assert!(card.body_hash.is_none());
    }

    #[test]
    fn sync_deduplicates() {
        let store = SqliteStore::open_in_memory().unwrap();
        sync_deck(
            &store,
            &["src/a.rs".to_string(), "src/a.rs".to_string()],
            "2026-03-25",
        )
        .unwrap();

        let all = store.get_all_cards().unwrap();
        assert_eq!(all.len(), 1);
    }
}
