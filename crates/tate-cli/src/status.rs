use std::path::Path;

use tate_store::deck::sync_deck;

use crate::common;

pub fn run(repo_root: &Path, json: bool) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let store = common::open_store(&tate_dir)?;
    let deck = common::open_deck(&tate_dir);
    let today = common::today_str();

    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    sync_deck(&store, &entries, &today).map_err(|e| format!("failed to sync: {e}"))?;

    let counts = store.card_counts().map_err(|e| format!("{e}"))?;
    let due_today = store.due_cards(&today).map_err(|e| format!("{e}"))?;

    let next_week = (chrono::Utc::now() + chrono::TimeDelta::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let due_week = store.due_cards(&next_week).map_err(|e| format!("{e}"))?;

    let streak = store.streak().map_err(|e| format!("{e}"))?;
    let total = counts.new + counts.learning + counts.mature;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "deck": { "total": total, "new": counts.new, "learning": counts.learning, "mature": counts.mature, "retired": counts.retired },
                "due": { "today": due_today.len(), "this_week": due_week.len() },
                "streak": streak
            })
        );
    } else {
        println!("Deck:     {} entries", total);
        println!(
            "Due:      {} today, {} this week",
            due_today.len(),
            due_week.len()
        );
        println!(
            "Streak:   {} day{}",
            streak,
            if streak == 1 { "" } else { "s" }
        );
        println!(
            "Progress: {} new / {} learning / {} mature",
            counts.new, counts.learning, counts.mature
        );
    }

    Ok(())
}
