use chrono::NaiveDate;

use crate::card::{CardRow, TypedCard};
use crate::entry::Entry;
use crate::grade::Grade;
use crate::sm2;
#[derive(Debug, Clone, PartialEq)]
pub enum SkipReason {
    FileNotFound,
    SymbolNotFound { found: Vec<String> },
    ParseFailed,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Next,
    SourceResolved(Entry, Result<String, SkipReason>),
    QuestionLoaded(Entry, Option<String>, Option<String>),
    Graded(Grade),
    Quit,
    Persisted(Entry),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ResolveSource(Entry),
    LoadQuestion(Entry),
    PresentCard {
        entry: Entry,
        source: String,
        question: Option<String>,
    },
    RevealAnswer {
        answer: String,
    },
    PromptGrade,
    PersistReview(Entry, CardRow),
    ShowSkipped(Entry, SkipReason),
    ShowSummary {
        reviewed: u32,
        skipped: u32,
    },
}
#[derive(Debug, Clone)]
pub struct ReviewState {
    cards: Vec<TypedCard>,
    current: usize,
    current_source: Option<String>,
    pub reviewed: u32,
    pub skipped: u32,
    today: NaiveDate,
    max_interval: u32,
}

impl ReviewState {
    pub fn new(cards: Vec<TypedCard>, today: NaiveDate, max_interval: u32) -> Self {
        ReviewState {
            cards,
            current: 0,
            current_source: None,
            reviewed: 0,
            skipped: 0,
            today,
            max_interval,
        }
    }

    pub fn remaining(&self) -> usize {
        self.cards.len().saturating_sub(self.current)
    }
}
pub fn review_update(mut state: ReviewState, msg: Message) -> (ReviewState, Vec<Command>) {
    match msg {
        Message::Next => {
            if state.current >= state.cards.len() {
                let cmds = vec![Command::ShowSummary {
                    reviewed: state.reviewed,
                    skipped: state.skipped,
                }];
                return (state, cmds);
            }
            let entry = state.cards[state.current].entry().clone();
            (state, vec![Command::ResolveSource(entry)])
        }

        Message::SourceResolved(entry, Ok(source)) => {
            state.current_source = Some(source);
            (state, vec![Command::LoadQuestion(entry)])
        }

        Message::SourceResolved(entry, Err(reason)) => {
            state.skipped += 1;
            let cmds = vec![Command::ShowSkipped(entry, reason)];
            state.current += 1;
            state.current_source = None;
            let (state, mut next_cmds) = review_update(state, Message::Next);
            let mut all = cmds;
            all.append(&mut next_cmds);
            (state, all)
        }

        Message::QuestionLoaded(entry, question, answer) => {
            let source = state.current_source.take().unwrap_or_default();
            let mut cmds = vec![Command::PresentCard {
                entry,
                source,
                question,
            }];
            if let Some(ans) = answer {
                cmds.push(Command::RevealAnswer { answer: ans });
            }
            cmds.push(Command::PromptGrade);
            (state, cmds)
        }

        Message::Graded(grade) => {
            let card = state.cards[state.current].clone();
            let entry = card.entry().clone();
            let updated = sm2::sm2_update(card, grade, state.today, state.max_interval);
            let row = updated.clone().into_row();
            state.cards[state.current] = updated;
            (state, vec![Command::PersistReview(entry, row)])
        }

        Message::Persisted(_) => {
            state.reviewed += 1;
            state.current += 1;
            state.current_source = None;
            review_update(state, Message::Next)
        }

        Message::Quit => {
            let cmds = vec![Command::ShowSummary {
                reviewed: state.reviewed,
                skipped: state.skipped,
            }];
            (state, cmds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::*;
    use crate::ease::Ease;
    use std::path::PathBuf;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()
    }

    fn test_card(name: &str) -> TypedCard {
        TypedCard::New(Card {
            entry: Entry::File(PathBuf::from(name)),
            ease: Ease::default_ease(),
            added: today(),
            body_hash: None,
            state: New {
                due: today(),
                lapses: 0,
            },
        })
    }

    fn entry(name: &str) -> Entry {
        Entry::File(PathBuf::from(name))
    }

    #[test]
    fn happy_path_single_card() {
        let state = ReviewState::new(vec![test_card("a.rs")], today(), 365);

        let (state, cmds) = review_update(state, Message::Next);
        assert_eq!(cmds, vec![Command::ResolveSource(entry("a.rs"))]);

        let (state, cmds) = review_update(
            state,
            Message::SourceResolved(entry("a.rs"), Ok("fn main() {}".into())),
        );
        assert_eq!(cmds, vec![Command::LoadQuestion(entry("a.rs"))]);

        let (state, cmds) = review_update(
            state,
            Message::QuestionLoaded(entry("a.rs"), Some("What does this do?".into()), None),
        );
        assert_eq!(cmds.len(), 2);
        assert!(matches!(&cmds[0], Command::PresentCard { .. }));
        assert!(matches!(&cmds[1], Command::PromptGrade));

        let (state, cmds) = review_update(state, Message::Graded(Grade::Good));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], Command::PersistReview(_, _)));

        let (state, cmds) = review_update(state, Message::Persisted(entry("a.rs")));
        assert_eq!(
            cmds,
            vec![Command::ShowSummary {
                reviewed: 1,
                skipped: 0
            }]
        );
        assert_eq!(state.reviewed, 1);
        assert_eq!(state.skipped, 0);
    }

    #[test]
    fn skip_on_file_not_found() {
        let state = ReviewState::new(vec![test_card("a.rs")], today(), 365);

        let (state, cmds) = review_update(state, Message::Next);
        assert_eq!(cmds, vec![Command::ResolveSource(entry("a.rs"))]);

        let (state, cmds) = review_update(
            state,
            Message::SourceResolved(entry("a.rs"), Err(SkipReason::FileNotFound)),
        );

        assert!(matches!(
            &cmds[0],
            Command::ShowSkipped(_, SkipReason::FileNotFound)
        ));
        assert!(matches!(
            cmds.last().unwrap(),
            Command::ShowSummary {
                reviewed: 0,
                skipped: 1
            }
        ));
        assert_eq!(state.skipped, 1);
    }

    #[test]
    fn quit_mid_session() {
        let state = ReviewState::new(vec![test_card("a.rs"), test_card("b.rs")], today(), 365);

        let (state, _) = review_update(state, Message::Next);
        let (state, _) = review_update(
            state,
            Message::SourceResolved(entry("a.rs"), Ok("code".into())),
        );
        let (state, _) = review_update(state, Message::QuestionLoaded(entry("a.rs"), None, None));
        let (state, _) = review_update(state, Message::Graded(Grade::Good));
        let (_state, _) = review_update(state, Message::Persisted(entry("a.rs")));

        let state = ReviewState::new(vec![test_card("a.rs"), test_card("b.rs")], today(), 365);
        let (state, _) = review_update(state, Message::Next);
        let (state, _) = review_update(
            state,
            Message::SourceResolved(entry("a.rs"), Ok("code".into())),
        );
        let (state, _) = review_update(state, Message::QuestionLoaded(entry("a.rs"), None, None));
        let (state, _) = review_update(state, Message::Graded(Grade::Good));
        let (state, _) = review_update(state, Message::Persisted(entry("a.rs")));
        let (state, cmds) = review_update(state, Message::Quit);
        assert_eq!(
            cmds,
            vec![Command::ShowSummary {
                reviewed: 1,
                skipped: 0
            }]
        );
        assert_eq!(state.reviewed, 1);
    }

    #[test]
    fn no_cards_immediate_summary() {
        let state = ReviewState::new(vec![], today(), 365);
        let (state, cmds) = review_update(state, Message::Next);
        assert_eq!(
            cmds,
            vec![Command::ShowSummary {
                reviewed: 0,
                skipped: 0
            }]
        );
        assert_eq!(state.remaining(), 0);
    }

    #[test]
    fn mixed_review_and_skip() {
        let state = ReviewState::new(
            vec![test_card("a.rs"), test_card("b.rs"), test_card("c.rs")],
            today(),
            365,
        );

        let (state, _) = review_update(state, Message::Next);
        let (state, _) = review_update(
            state,
            Message::SourceResolved(entry("a.rs"), Ok("code".into())),
        );
        let (state, _) = review_update(state, Message::QuestionLoaded(entry("a.rs"), None, None));
        let (state, _) = review_update(state, Message::Graded(Grade::Good));
        let (state, _) = review_update(state, Message::Persisted(entry("a.rs")));

        let (state, _) = review_update(
            state,
            Message::SourceResolved(entry("b.rs"), Err(SkipReason::ParseFailed)),
        );

        let (state, _) = review_update(
            state,
            Message::SourceResolved(entry("c.rs"), Ok("code".into())),
        );
        let (state, _) = review_update(state, Message::QuestionLoaded(entry("c.rs"), None, None));
        let (state, _) = review_update(state, Message::Graded(Grade::Easy));
        let (state, cmds) = review_update(state, Message::Persisted(entry("c.rs")));

        assert!(matches!(
            cmds.last().unwrap(),
            Command::ShowSummary {
                reviewed: 2,
                skipped: 1
            }
        ));
        assert_eq!(state.reviewed, 2);
        assert_eq!(state.skipped, 1);
    }
}
