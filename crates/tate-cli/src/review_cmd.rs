use std::path::Path;

use tate_core::card::CardRow;
use tate_core::entry::Entry;
use tate_core::grade::Grade;
use tate_core::sm2;
use tate_store::deck::sync_deck;

use crate::common;

pub fn run(
    repo_root: &Path,
    json: bool,
    export: bool,
    grade_args: Option<Vec<String>>,
) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let store = common::open_store(&tate_dir)?;
    let deck = common::open_deck(&tate_dir);
    let config = common::load_config(&tate_dir)?;
    let today = common::today_str();

    if export {
        return run_export(&store, &deck, &config, repo_root, &today);
    }

    if let Some(args) = grade_args {
        if args.len() != 2 {
            return Err("usage: tate review --grade <entry> <1-4>".to_string());
        }
        return run_grade(&store, &deck, &args[0], &args[1], &today, json);
    }

    tate_review::shell::run_review(&store, &deck, &config, repo_root).map_err(|e| format!("{e}"))
}

fn run_export(
    store: &tate_store::db::SqliteStore,
    deck: &tate_store::deck::DeckFile,
    _config: &tate_store::config::Config,
    repo_root: &Path,
    today: &str,
) -> Result<(), String> {
    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    sync_deck(store, &entries, today).map_err(|e| format!("failed to sync: {e}"))?;

    let due_cards = store.due_cards(today).map_err(|e| format!("{e}"))?;

    let mut result = Vec::new();
    for card in &due_cards {
        let entry = Entry::parse(&card.entry).ok();
        let source = entry
            .as_ref()
            .and_then(|e| resolve_source_text(e, repo_root));
        let qa = store.get_question(&card.entry).ok().flatten();
        let question = qa.as_ref().map(|q| q.question.as_str());
        let answer = qa.as_ref().and_then(|q| q.answer.as_deref());

        result.push(serde_json::json!({
            "entry": card.entry,
            "source": source,
            "question": question,
            "answer": answer,
        }));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
    Ok(())
}

fn run_grade(
    store: &tate_store::db::SqliteStore,
    deck: &tate_store::deck::DeckFile,
    entry_str: &str,
    grade_str: &str,
    today: &str,
    json: bool,
) -> Result<(), String> {
    let entries = deck
        .read()
        .map_err(|e| format!("failed to read deck: {e}"))?;
    sync_deck(store, &entries, today).map_err(|e| format!("failed to sync: {e}"))?;

    let grade = match grade_str {
        "1" => Grade::Blank,
        "2" => Grade::Hard,
        "3" => Grade::Good,
        "4" => Grade::Easy,
        _ => return Err("grade must be 1-4".to_string()),
    };

    let card = store
        .get_card(entry_str)
        .map_err(|e| format!("{e}"))?
        .ok_or_else(|| format!("card not found: {entry_str}"))?;

    let today_date = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let typed = card.into_typed();
    let updated = sm2::sm2_update(
        typed,
        grade,
        today_date,
        store
            .get_card(entry_str)
            .ok()
            .flatten()
            .map(|_| 365u32)
            .unwrap_or(365),
    );
    let row: CardRow = updated.into_row();

    store.save_card(&row).map_err(|e| format!("{e}"))?;
    let grade_u8: u8 = grade.into();
    store
        .save_review(entry_str, grade_u8)
        .map_err(|e| format!("{e}"))?;

    let status = if row.reps == 0 {
        "new"
    } else if row.interval >= 21 {
        "mature"
    } else {
        "learning"
    };

    if json {
        println!(
            "{}",
            serde_json::json!({
                "entry": entry_str,
                "grade": grade_u8,
                "next_due": row.due,
                "interval": row.interval,
                "status": status,
            })
        );
    } else {
        println!("Graded: {entry_str} ({grade_u8})");
        println!("Next review: {}", row.due);
    }

    Ok(())
}

fn resolve_source_text(entry: &Entry, repo_root: &Path) -> Option<String> {
    match entry {
        Entry::Symbol { path, name } => {
            let full_path = repo_root.join(path);
            tate_symbols::resolver::resolve_symbol(&full_path, name)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
        }
        Entry::File(path) => {
            let full_path = repo_root.join(path);
            std::fs::read_to_string(&full_path).ok()
        }
        Entry::Range { path, start, end } => {
            let full_path = repo_root.join(path);
            tate_symbols::resolver::resolve_range(&full_path, *start, *end)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
        }
    }
}
