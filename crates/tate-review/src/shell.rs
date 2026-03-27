use std::path::Path;

use tate_core::card::CardRow;
use tate_core::entry::Entry;
use tate_core::review::{self, Command, Message, ReviewState};
use tate_store::config::Config;
use tate_store::db::SqliteStore;
use tate_store::deck::{sync_deck, DeckFile};

use crate::change;
use crate::error::ReviewError;
use crate::terminal::{ReviewTui, UserInput};

pub fn run_review(
    store: &SqliteStore,
    deck: &DeckFile,
    config: &Config,
    repo_root: &Path,
) -> Result<(), ReviewError> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_date = chrono::Utc::now().date_naive();

    let entries = deck.read()?;
    sync_deck(store, &entries, &today)?;

    let due_cards = store.due_cards(&today)?;
    if due_cards.is_empty() {
        println!("No cards due today.");
        return Ok(());
    }

    let mut new_count = 0u32;
    let limited: Vec<CardRow> = due_cards
        .into_iter()
        .filter(|c| {
            if c.reps == 0 {
                if new_count >= config.scheduling.new_card_limit {
                    return false;
                }
                new_count += 1;
            }
            true
        })
        .collect();

    if limited.is_empty() {
        println!("No cards due today.");
        return Ok(());
    }

    let change_results = change::detect_changes(limited, store, &today, repo_root);

    let mut review_cards = Vec::new();
    let mut pre_skipped = Vec::new();
    for cr in change_results {
        if let Some(reason) = cr.skip {
            pre_skipped.push((cr.card.entry.clone(), reason));
        } else {
            review_cards.push(cr.card.into_typed());
        }
    }

    let mut terminal = ReviewTui::new(&config.display.theme)
        .map_err(|e| ReviewError::Other(format!("failed to initialize TUI: {e}")))?;
    terminal.set_progress((review_cards.len() + pre_skipped.len()) as u32);

    for (entry, reason) in &pre_skipped {
        terminal.show_skip(entry, reason);
    }

    if review_cards.is_empty() {
        terminal.show_summary(0, pre_skipped.len() as u32);
        return Ok(());
    }

    let mut state = ReviewState::new(review_cards, today_date, config.scheduling.max_interval);
    let mut msg = Message::Next;
    let mut user_quit = false;

    loop {
        let (new_state, commands) = review::review_update(state, msg);
        state = new_state;

        if commands.is_empty() {
            break;
        }

        let mut next_msg = None;

        for cmd in commands {
            match cmd {
                Command::ResolveSource(entry) => {
                    next_msg = Some(resolve_source(&entry, repo_root));
                }
                Command::LoadQuestion(entry) => {
                    let deck_line = entry.to_deck_line();
                    let qa = store.get_question(&deck_line).ok().flatten();
                    let question = qa.as_ref().map(|q| q.question.clone());
                    let answer = qa.and_then(|q| q.answer);
                    next_msg = Some(Message::QuestionLoaded(entry, question, answer));
                }
                Command::PresentCard {
                    entry,
                    source,
                    question,
                } => {
                    let deck_line = entry.to_deck_line();
                    let card_data = store.get_card(&deck_line).ok().flatten();
                    let review_num = card_data.as_ref().map(|c| c.reps).unwrap_or(0);
                    let lapses = card_data.as_ref().map(|c| c.lapses).unwrap_or(0);
                    let q = question
                        .as_deref()
                        .unwrap_or("Review this code. Can you explain the key decisions and potential edge cases?");
                    let src = if config.display.show_code {
                        Some(source.as_str())
                    } else {
                        None
                    };
                    terminal.show_card(&deck_line, review_num, lapses, src, q);
                }
                Command::RevealAnswer { answer } => {
                    if terminal.reveal_answer(&answer) {
                        user_quit = true;
                        next_msg = Some(Message::Quit);
                        break;
                    }
                }
                Command::PromptGrade => {
                    let input = terminal.prompt_grade();
                    match input {
                        UserInput::Grade(g) => next_msg = Some(Message::Graded(g)),
                        UserInput::Quit => {
                            user_quit = true;
                            next_msg = Some(Message::Quit);
                        }
                    }
                }
                Command::PersistReview(entry, row) => {
                    let deck_line = entry.to_deck_line();
                    store.save_card(&row).map_err(ReviewError::Storage)?;
                    store.save_review(&deck_line, 3).ok();
                    terminal.show_next_review(&row.due);
                    next_msg = Some(Message::Persisted(entry));
                }
                Command::ShowSkipped(entry, reason) => {
                    terminal.show_skip(&entry.to_deck_line(), &reason);
                }
                Command::ShowSummary { reviewed, skipped } => {
                    if !user_quit {
                        terminal.show_summary(reviewed, skipped + pre_skipped.len() as u32);
                    }
                    return Ok(());
                }
            }
        }

        msg = next_msg.unwrap_or(Message::Next);
    }

    Ok(())
}

fn resolve_source(entry: &Entry, repo_root: &Path) -> Message {
    match entry {
        Entry::Symbol { path, name } => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::resolve_symbol(&full_path, name) {
                Ok(bytes) => {
                    let source = String::from_utf8_lossy(&bytes).to_string();
                    Message::SourceResolved(entry.clone(), Ok(source))
                }
                Err(tate_symbols::error::SymbolError::SymbolNotFound { found, .. }) => {
                    Message::SourceResolved(
                        entry.clone(),
                        Err(tate_core::review::SkipReason::SymbolNotFound { found }),
                    )
                }
                Err(tate_symbols::error::SymbolError::Io { .. }) => Message::SourceResolved(
                    entry.clone(),
                    Err(tate_core::review::SkipReason::FileNotFound),
                ),
                Err(_) => Message::SourceResolved(
                    entry.clone(),
                    Err(tate_core::review::SkipReason::ParseFailed),
                ),
            }
        }
        Entry::File(path) => {
            let full_path = repo_root.join(path);
            match std::fs::read_to_string(&full_path) {
                Ok(source) => Message::SourceResolved(entry.clone(), Ok(source)),
                Err(_) => Message::SourceResolved(
                    entry.clone(),
                    Err(tate_core::review::SkipReason::FileNotFound),
                ),
            }
        }
        Entry::Range { path, start, end } => {
            let full_path = repo_root.join(path);
            match tate_symbols::resolver::resolve_range(&full_path, *start, *end) {
                Ok(bytes) => {
                    let source = String::from_utf8_lossy(&bytes).to_string();
                    Message::SourceResolved(entry.clone(), Ok(source))
                }
                Err(_) => Message::SourceResolved(
                    entry.clone(),
                    Err(tate_core::review::SkipReason::FileNotFound),
                ),
            }
        }
    }
}
