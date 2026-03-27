use std::path::Path;

use crate::common;

pub fn run(repo_root: &Path, entry_str: &str, json: bool) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let store = common::open_store(&tate_dir)?;
    let deck = common::open_deck(&tate_dir);

    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    if !entries.contains(&entry_str.to_string()) {
        return Err("Entry not in deck.".to_string());
    }

    deck.remove(entry_str)
        .map_err(|e| format!("failed to update deck file: {e}"))?;

    store
        .retire_card(entry_str)
        .map_err(|e| format!("failed to retire card: {e}"))?;

    if json {
        println!(
            "{}",
            serde_json::json!({"action": "retired", "entry": entry_str})
        );
    } else {
        println!("Owned: {entry_str}");
    }
    Ok(())
}
