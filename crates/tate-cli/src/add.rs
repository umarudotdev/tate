use std::path::Path;

use tate_core::entry::Entry;
use tate_store::deck::sync_deck;

use crate::common;

pub fn run(
    repo_root: &Path,
    entry_str: &str,
    question: Option<&str>,
    answer: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let store = common::open_store(&tate_dir)?;
    let deck = common::open_deck(&tate_dir);
    let today = common::today_str();

    let entry = Entry::parse(entry_str).map_err(|e| format!("invalid entry: {e}"))?;

    let file_path = repo_root.join(entry.path());
    if !file_path.exists() {
        return Err(format!("file not found: {}", entry.path().display()));
    }

    let body_hash = match &entry {
        Entry::Symbol { path, name } => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::hash_symbol(&full_path, name) {
                Ok(hash) => hash,
                Err(tate_symbols::error::SymbolError::SymbolNotFound { found, .. }) => {
                    let mut msg = format!("symbol `{name}` not found in {}", path.display());
                    if !found.is_empty() {
                        msg.push_str("\nFound symbols:");
                        for s in &found {
                            msg.push_str(&format!("\n  {}", s));
                        }
                    }
                    return Err(msg);
                }
                Err(tate_symbols::error::SymbolError::UnsupportedLanguage { ext }) => {
                    return Err(format!("unsupported language for symbol tracking: .{ext}"));
                }
                Err(e) => return Err(format!("failed to resolve symbol: {e}")),
            }
        }
        Entry::File(path) => {
            let full_path = repo_root.join(path);
            tate_symbols::resolver::hash_file(&full_path)
                .map_err(|e| format!("failed to hash file: {e}"))?
        }
        Entry::Range { path, start, end } => {
            let full_path = repo_root.join(path);
            tate_symbols::resolver::hash_range(&full_path, *start, *end)
                .map_err(|e| format!("failed to hash range: {e}"))?
        }
    };

    let source_text = if let Entry::Range { path, start, end } = &entry {
        let full_path = repo_root.join(path);
        let bytes = tate_symbols::resolver::resolve_range(&full_path, *start, *end)
            .map_err(|e| format!("failed to read range: {e}"))?;
        Some(String::from_utf8_lossy(&bytes).to_string())
    } else {
        None
    };

    let deck_line = entry.to_deck_line();

    let entries = deck.read().unwrap_or_default();
    if entries.contains(&deck_line) {
        if question.is_some() || answer.is_some() {
            let q = question.unwrap_or("Review this code.");
            store
                .save_question(&deck_line, &body_hash, q, answer, source_text.as_deref())
                .map_err(|e| format!("failed to save question: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"action": "updated", "entry": deck_line})
                );
            } else {
                println!("Updated question: {deck_line}");
            }
        } else if json {
            println!(
                "{}",
                serde_json::json!({"action": "exists", "entry": deck_line})
            );
        } else {
            println!("Already tracked.");
        }
        return Ok(());
    }

    deck.append(&deck_line)
        .map_err(|e| format!("failed to append to deck: {e}"))?;

    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    sync_deck(&store, &entries, &today).map_err(|e| format!("failed to sync: {e}"))?;

    store
        .update_body_hash(&deck_line, Some(&body_hash))
        .map_err(|e| format!("failed to store hash: {e}"))?;

    if question.is_some() || source_text.is_some() {
        let q = question.unwrap_or("Review this code.");
        store
            .save_question(&deck_line, &body_hash, q, answer, source_text.as_deref())
            .map_err(|e| format!("failed to save question: {e}"))?;
    }

    if json {
        println!(
            "{}",
            serde_json::json!({"action": "added", "entry": deck_line})
        );
    } else {
        println!("Added: {deck_line}");
    }
    Ok(())
}
