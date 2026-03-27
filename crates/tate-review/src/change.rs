use std::path::Path;

use tate_core::card::CardRow;
use tate_core::entry::Entry;
use tate_core::review::SkipReason;
use tate_core::sm2;
use tate_store::db::SqliteStore;

pub struct ChangeResult {
    pub card: CardRow,
    pub skip: Option<SkipReason>,
}

pub fn detect_changes(
    cards: Vec<CardRow>,
    store: &SqliteStore,
    today: &str,
    repo_root: &Path,
) -> Vec<ChangeResult> {
    cards
        .into_iter()
        .map(|card| detect_one(card, store, today, repo_root))
        .collect()
}

fn detect_one(card: CardRow, store: &SqliteStore, today: &str, repo_root: &Path) -> ChangeResult {
    let entry = match Entry::parse(&card.entry) {
        Ok(e) => e,
        Err(_) => {
            return ChangeResult {
                card,
                skip: Some(SkipReason::ParseFailed),
            };
        }
    };

    let current_hash = match &entry {
        Entry::Symbol { path, name } => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::hash_symbol(&full_path, name) {
                Ok(h) => Some(h),
                Err(tate_symbols::error::SymbolError::Io { .. }) => {
                    return ChangeResult {
                        card,
                        skip: Some(SkipReason::FileNotFound),
                    };
                }
                Err(tate_symbols::error::SymbolError::SymbolNotFound { found, .. }) => {
                    return ChangeResult {
                        card,
                        skip: Some(SkipReason::SymbolNotFound { found }),
                    };
                }
                Err(_) => {
                    return ChangeResult {
                        card,
                        skip: Some(SkipReason::ParseFailed),
                    };
                }
            }
        }
        Entry::File(path) => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::hash_file(&full_path) {
                Ok(h) => Some(h),
                Err(_) => {
                    return ChangeResult {
                        card,
                        skip: Some(SkipReason::FileNotFound),
                    };
                }
            }
        }
        Entry::Range { path, start, end } => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::hash_range(&full_path, *start, *end) {
                Ok(h) => Some(h),
                Err(_) => {
                    return ChangeResult {
                        card,
                        skip: Some(SkipReason::FileNotFound),
                    };
                }
            }
        }
    };

    let changed = match (&card.body_hash, &current_hash) {
        (Some(stored), Some(current)) => stored != current,
        (None, Some(_)) => true,
        _ => false,
    };

    if changed {
        let typed = card.into_typed();
        let reset = sm2::change_reset(
            typed,
            chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive()),
        );
        let mut row = reset.into_row();
        row.body_hash = current_hash;

        let _ = store.save_card(&row);
        ChangeResult {
            card: row,
            skip: None,
        }
    } else {
        if card.body_hash.is_none() {
            if let Some(ref h) = current_hash {
                let _ = store.update_body_hash(&card.entry, Some(h));
            }
        }
        let mut updated = card;
        if updated.body_hash.is_none() {
            updated.body_hash = current_hash;
        }
        ChangeResult {
            card: updated,
            skip: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_card(entry: &str, body_hash: Option<&str>) -> CardRow {
        CardRow {
            entry: entry.to_string(),
            ease: 2.5,
            interval: 0,
            due: "2026-03-27".to_string(),
            reps: 0,
            lapses: 0,
            added: "2026-03-27".to_string(),
            retired: false,
            body_hash: body_hash.map(|s| s.to_string()),
        }
    }

    fn setup() -> (SqliteStore, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let store = SqliteStore::open_in_memory().unwrap();
        (store, dir)
    }

    #[test]
    fn no_change_when_hash_matches() {
        let (store, dir) = setup();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        let hash = tate_symbols::resolver::hash_file(&dir.path().join("main.rs")).unwrap();

        let card = make_card("main.rs", Some(&hash));
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        assert_eq!(results.len(), 1);
        assert!(results[0].skip.is_none());
        assert_eq!(results[0].card.body_hash.as_deref(), Some(hash.as_str()));
    }

    #[test]
    fn change_detected_when_hash_mismatches() {
        let (store, dir) = setup();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let card = make_card("main.rs", Some("oldhash"));
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        assert_eq!(results.len(), 1);
        assert!(results[0].skip.is_none());
        assert_ne!(
            results[0].card.body_hash.as_deref(),
            Some("oldhash"),
            "hash should be updated after change detection"
        );
    }

    #[test]
    fn first_hash_computed_when_none() {
        let (store, dir) = setup();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let card = make_card("main.rs", None);
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        assert!(
            results[0].card.body_hash.is_some(),
            "body_hash should be computed on first detection"
        );
    }

    #[test]
    fn missing_file_skips_card() {
        let (store, dir) = setup();

        let card = make_card("nonexistent.rs", Some("hash"));
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        assert!(matches!(results[0].skip, Some(SkipReason::FileNotFound)));
    }

    #[test]
    fn invalid_entry_skips_with_parse_failed() {
        let (store, dir) = setup();

        let card = make_card("", None);
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        assert!(matches!(results[0].skip, Some(SkipReason::ParseFailed)));
    }

    #[test]
    fn change_resets_card_preserving_ease() {
        let (store, dir) = setup();
        std::fs::write(dir.path().join("main.rs"), "fn changed() {}\n").unwrap();

        let card = CardRow {
            ease: 1.8,
            reps: 5,
            interval: 30,
            ..make_card("main.rs", Some("oldhash"))
        };
        store.save_card(&card).unwrap();

        let results = detect_changes(vec![card], &store, "2026-03-27", dir.path());
        let result = &results[0].card;
        assert_eq!(result.ease, 1.8, "ease should be preserved across reset");
        assert_eq!(result.reps, 0, "reps should reset to 0");
        assert_eq!(result.interval, 0, "interval should reset to 0");
    }
}
