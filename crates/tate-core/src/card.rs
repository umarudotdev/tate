use chrono::NaiveDate;

use crate::ease::Ease;
use crate::entry::Entry;
#[derive(Debug, Clone, PartialEq)]
pub struct New {
    pub due: NaiveDate,
    pub lapses: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Learning {
    pub reps: u32,
    pub interval: u32,
    pub due: NaiveDate,
    pub lapses: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mature {
    pub reps: u32,
    pub interval: u32,
    pub due: NaiveDate,
    pub lapses: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Retired;
#[derive(Debug, Clone, PartialEq)]
pub struct Card<S> {
    pub entry: Entry,
    pub ease: Ease,
    pub added: NaiveDate,
    pub body_hash: Option<String>,
    pub state: S,
}

impl<S> Card<S> {
    pub fn due(&self) -> Option<NaiveDate>
    where
        S: HasDue,
    {
        Some(self.state.due())
    }
}

pub trait HasDue {
    fn due(&self) -> NaiveDate;
}

impl HasDue for New {
    fn due(&self) -> NaiveDate {
        self.due
    }
}

impl HasDue for Learning {
    fn due(&self) -> NaiveDate {
        self.due
    }
}

impl HasDue for Mature {
    fn due(&self) -> NaiveDate {
        self.due
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypedCard {
    New(Card<New>),
    Learning(Card<Learning>),
    Mature(Card<Mature>),
    Retired(Card<Retired>),
}

impl TypedCard {
    pub fn entry(&self) -> &Entry {
        match self {
            TypedCard::New(c) => &c.entry,
            TypedCard::Learning(c) => &c.entry,
            TypedCard::Mature(c) => &c.entry,
            TypedCard::Retired(c) => &c.entry,
        }
    }

    pub fn ease(&self) -> Ease {
        match self {
            TypedCard::New(c) => c.ease,
            TypedCard::Learning(c) => c.ease,
            TypedCard::Mature(c) => c.ease,
            TypedCard::Retired(c) => c.ease,
        }
    }

    pub fn into_row(self) -> CardRow {
        match self {
            TypedCard::New(c) => CardRow {
                entry: c.entry.to_deck_line(),
                ease: c.ease.inner(),
                interval: 0,
                due: c.state.due.format("%Y-%m-%d").to_string(),
                reps: 0,
                lapses: c.state.lapses,
                added: c.added.format("%Y-%m-%d").to_string(),
                retired: false,
                body_hash: c.body_hash,
            },
            TypedCard::Learning(c) => CardRow {
                entry: c.entry.to_deck_line(),
                ease: c.ease.inner(),
                interval: c.state.interval,
                due: c.state.due.format("%Y-%m-%d").to_string(),
                reps: c.state.reps,
                lapses: c.state.lapses,
                added: c.added.format("%Y-%m-%d").to_string(),
                retired: false,
                body_hash: c.body_hash,
            },
            TypedCard::Mature(c) => CardRow {
                entry: c.entry.to_deck_line(),
                ease: c.ease.inner(),
                interval: c.state.interval,
                due: c.state.due.format("%Y-%m-%d").to_string(),
                reps: c.state.reps,
                lapses: c.state.lapses,
                added: c.added.format("%Y-%m-%d").to_string(),
                retired: false,
                body_hash: c.body_hash,
            },
            TypedCard::Retired(c) => CardRow {
                entry: c.entry.to_deck_line(),
                ease: c.ease.inner(),
                interval: 0,
                due: String::new(),
                reps: 0,
                lapses: 0,
                added: c.added.format("%Y-%m-%d").to_string(),
                retired: true,
                body_hash: c.body_hash,
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CardRow {
    pub entry: String,
    pub ease: f64,
    pub interval: u32,
    pub due: String,
    pub reps: u32,
    pub lapses: u32,
    pub added: String,
    pub retired: bool,
    pub body_hash: Option<String>,
}

impl CardRow {
    pub fn into_typed(self) -> TypedCard {
        let entry = match Entry::parse(&self.entry) {
            Ok(e) => e,
            Err(_) => Entry::File(std::path::PathBuf::from(&self.entry)),
        };
        let ease = Ease::new(self.ease);
        let added = NaiveDate::parse_from_str(&self.added, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());

        if self.retired {
            return TypedCard::Retired(Card {
                entry,
                ease,
                added,
                body_hash: self.body_hash,
                state: Retired,
            });
        }

        let due = NaiveDate::parse_from_str(&self.due, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());

        if self.reps == 0 {
            return TypedCard::New(Card {
                entry,
                ease,
                added,
                body_hash: self.body_hash,
                state: New {
                    due,
                    lapses: self.lapses,
                },
            });
        }

        if self.interval == 0 {
            return TypedCard::New(Card {
                entry,
                ease,
                added,
                body_hash: self.body_hash,
                state: New {
                    due,
                    lapses: self.lapses,
                },
            });
        }

        if self.interval >= 21 {
            TypedCard::Mature(Card {
                entry,
                ease,
                added,
                body_hash: self.body_hash,
                state: Mature {
                    reps: self.reps,
                    interval: self.interval,
                    due,
                    lapses: self.lapses,
                },
            })
        } else {
            TypedCard::Learning(Card {
                entry,
                ease,
                added,
                body_hash: self.body_hash,
                state: Learning {
                    reps: self.reps,
                    interval: self.interval,
                    due,
                    lapses: self.lapses,
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(reps: u32, interval: u32, retired: bool, lapses: u32) -> CardRow {
        CardRow {
            entry: "src/main.rs".to_string(),
            ease: 2.5,
            interval,
            due: "2026-03-25".to_string(),
            reps,
            lapses,
            added: "2026-03-20".to_string(),
            retired,
            body_hash: Some("abc123".to_string()),
        }
    }
    #[test]
    fn hydrate_new_preserves_due_and_lapses() {
        let card = make_row(0, 0, false, 5).into_typed();
        if let TypedCard::New(c) = card {
            assert_eq!(c.state.lapses, 5, "lapses should be preserved from row");
            assert_eq!(
                c.state.due.format("%Y-%m-%d").to_string(),
                "2026-03-25",
                "due should be preserved from row"
            );
        } else {
            panic!("expected New, got {:?}", card);
        }
    }

    #[test]
    fn hydrate_retired_ignores_scheduling_fields() {
        let card = make_row(10, 30, true, 5).into_typed();
        assert!(
            matches!(card, TypedCard::Retired(_)),
            "retired=true should always produce Retired regardless of reps/interval"
        );
    }

    #[test]
    fn hydrate_corrupt_reps_with_zero_interval_becomes_new() {
        let card = make_row(5, 0, false, 3).into_typed();
        if let TypedCard::New(c) = card {
            assert_eq!(c.state.lapses, 3, "lapses preserved on corruption reset");
        } else {
            panic!(
                "reps=5, interval=0 should be treated as corruption -> New, got {:?}",
                card
            );
        }
    }
    #[test]
    fn hydrate_interval_20_is_learning() {
        let card = make_row(3, 20, false, 0).into_typed();
        assert!(
            matches!(card, TypedCard::Learning(_)),
            "interval=20 should be Learning (threshold is >=21 for Mature)"
        );
    }

    #[test]
    fn hydrate_interval_21_is_mature() {
        let card = make_row(3, 21, false, 0).into_typed();
        assert!(
            matches!(card, TypedCard::Mature(_)),
            "interval=21 should be Mature (threshold is >=21)"
        );
    }
    #[test]
    fn hydrate_invalid_date_falls_back_to_today() {
        let row = CardRow {
            due: "not-a-date".to_string(),
            added: "also-invalid".to_string(),
            ..make_row(0, 0, false, 0)
        };
        let card = row.into_typed();
        assert!(matches!(card, TypedCard::New(_)));
    }
    #[test]
    fn round_trip_preserves_all_fields() {
        for (reps, interval, retired, lapses, expected) in [
            (0, 0, false, 0, "New"),
            (2, 6, false, 1, "Learning"),
            (5, 30, false, 2, "Mature"),
            (0, 0, true, 0, "Retired"),
        ] {
            let row = make_row(reps, interval, retired, lapses);
            let back = row.clone().into_typed().into_row();
            assert_eq!(row.entry, back.entry, "{expected} round-trip: entry");
            assert_eq!(row.reps, back.reps, "{expected} round-trip: reps");
            assert_eq!(
                row.interval, back.interval,
                "{expected} round-trip: interval"
            );
            assert_eq!(row.lapses, back.lapses, "{expected} round-trip: lapses");
            assert_eq!(row.retired, back.retired, "{expected} round-trip: retired");
        }
    }

    #[test]
    fn round_trip_none_body_hash() {
        let row = CardRow {
            body_hash: None,
            ..make_row(2, 6, false, 0)
        };
        let back = row.clone().into_typed().into_row();
        assert_eq!(
            back.body_hash, None,
            "None body_hash should round-trip as None"
        );
    }
}
