use std::path::Path;

use tate_core::card::TypedCard;
use tate_store::deck::sync_deck;

use crate::common;

pub fn run(
    repo_root: &Path,
    prefix: Option<&str>,
    due_only: bool,
    owned_only: bool,
    json: bool,
) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let store = common::open_store(&tate_dir)?;
    let deck = common::open_deck(&tate_dir);
    let today = common::today_str();

    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    sync_deck(&store, &entries, &today).map_err(|e| format!("failed to sync: {e}"))?;

    let cards = store.get_all_cards().map_err(|e| format!("{e}"))?;

    let mut rows: Vec<serde_json::Value> = Vec::new();

    for card in &cards {
        let typed = card.clone().into_typed();

        let (status, interval, due_str) = match &typed {
            TypedCard::New(c) => {
                let s = if c.state.due.format("%Y-%m-%d").to_string() <= today {
                    "due"
                } else {
                    "new"
                };
                (s, 0u32, c.state.due.format("%Y-%m-%d").to_string())
            }
            TypedCard::Learning(c) => {
                let s = if c.state.due.format("%Y-%m-%d").to_string() <= today {
                    "due"
                } else {
                    "learning"
                };
                (
                    s,
                    c.state.interval,
                    c.state.due.format("%Y-%m-%d").to_string(),
                )
            }
            TypedCard::Mature(c) => {
                let s = if c.state.due.format("%Y-%m-%d").to_string() <= today {
                    "due"
                } else {
                    "mature"
                };
                (
                    s,
                    c.state.interval,
                    c.state.due.format("%Y-%m-%d").to_string(),
                )
            }
            TypedCard::Retired(_) => ("owned", 0, "-".to_string()),
        };

        if owned_only && status != "owned" {
            continue;
        }
        if !owned_only && status == "owned" {
            continue;
        }
        if due_only && status != "due" {
            continue;
        }
        if let Some(p) = prefix {
            if !card.entry.starts_with(p) {
                continue;
            }
        }

        rows.push(serde_json::json!({
            "entry": card.entry,
            "status": status,
            "interval": interval,
            "due": due_str,
            "ease": card.ease,
            "reps": card.reps,
            "lapses": card.lapses,
            "added": card.added,
        }));
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        );
    } else {
        if rows.is_empty() {
            println!("No entries found.");
            return Ok(());
        }

        let max_entry = rows
            .iter()
            .map(|r| r["entry"].as_str().unwrap_or("").len())
            .max()
            .unwrap_or(5)
            .max(5);
        let header = format!(
            "{:<max_entry$}  {:<10}  {:<10}  DUE",
            "ENTRY", "STATUS", "INTERVAL"
        );
        println!("{header}");
        for row in &rows {
            let interval_str = if row["interval"].as_u64().unwrap_or(0) == 0 {
                "-".to_string()
            } else {
                format!("{}d", row["interval"])
            };
            let line = format!(
                "{:<max_entry$}  {:<10}  {:<10}  {}",
                row["entry"].as_str().unwrap_or(""),
                row["status"].as_str().unwrap_or(""),
                interval_str,
                row["due"].as_str().unwrap_or("")
            );
            println!("{line}");
        }
    }

    Ok(())
}
