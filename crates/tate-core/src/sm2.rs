use chrono::{NaiveDate, TimeDelta};

use crate::card::*;
use crate::grade::Grade;

pub fn sm2_update(card: TypedCard, grade: Grade, today: NaiveDate, max_interval: u32) -> TypedCard {
    let quality = grade.sm2_quality();

    match card {
        TypedCard::New(c) => update_new(c, grade, quality, today, max_interval),
        TypedCard::Learning(c) => update_learning(c, grade, quality, today, max_interval),
        TypedCard::Mature(c) => update_mature(c, grade, quality, today, max_interval),
        TypedCard::Retired(_) => card,
    }
}

pub fn change_reset(card: TypedCard, today: NaiveDate) -> TypedCard {
    let (entry, ease, added, body_hash, lapses) = match card {
        TypedCard::New(c) => (c.entry, c.ease, c.added, c.body_hash, c.state.lapses),
        TypedCard::Learning(c) => (c.entry, c.ease, c.added, c.body_hash, c.state.lapses),
        TypedCard::Mature(c) => (c.entry, c.ease, c.added, c.body_hash, c.state.lapses),
        TypedCard::Retired(c) => (c.entry, c.ease, c.added, c.body_hash, 0),
    };

    TypedCard::New(Card {
        entry,
        ease,
        added,
        body_hash,
        state: New { due: today, lapses },
    })
}

fn update_new(
    c: Card<New>,
    grade: Grade,
    quality: f64,
    today: NaiveDate,
    max_interval: u32,
) -> TypedCard {
    let new_ease = c.ease.update(quality);

    if grade.is_lapse() {
        TypedCard::New(Card {
            ease: new_ease,
            state: New {
                due: today,
                lapses: c.state.lapses,
            },
            ..c
        })
    } else {
        let interval = 1u32.min(max_interval);
        TypedCard::Learning(Card {
            entry: c.entry,
            ease: new_ease,
            added: c.added,
            body_hash: c.body_hash,
            state: Learning {
                reps: 1,
                interval,
                due: today + TimeDelta::days(interval as i64),
                lapses: c.state.lapses,
            },
        })
    }
}

fn update_learning(
    c: Card<Learning>,
    grade: Grade,
    quality: f64,
    today: NaiveDate,
    max_interval: u32,
) -> TypedCard {
    let new_ease = c.ease.update(quality);

    if grade.is_lapse() {
        TypedCard::New(Card {
            entry: c.entry,
            ease: new_ease,
            added: c.added,
            body_hash: c.body_hash,
            state: New {
                due: today,
                lapses: c.state.lapses + 1,
            },
        })
    } else {
        let new_reps = c.state.reps + 1;
        let raw_interval = if c.state.reps == 1 {
            6
        } else {
            (c.state.interval as f64 * new_ease.inner()).round() as u32
        };
        let interval = raw_interval.min(max_interval);

        if interval >= 21 {
            TypedCard::Mature(Card {
                entry: c.entry,
                ease: new_ease,
                added: c.added,
                body_hash: c.body_hash,
                state: Mature {
                    reps: new_reps,
                    interval,
                    due: today + TimeDelta::days(interval as i64),
                    lapses: c.state.lapses,
                },
            })
        } else {
            TypedCard::Learning(Card {
                entry: c.entry,
                ease: new_ease,
                added: c.added,
                body_hash: c.body_hash,
                state: Learning {
                    reps: new_reps,
                    interval,
                    due: today + TimeDelta::days(interval as i64),
                    lapses: c.state.lapses,
                },
            })
        }
    }
}

fn update_mature(
    c: Card<Mature>,
    grade: Grade,
    quality: f64,
    today: NaiveDate,
    max_interval: u32,
) -> TypedCard {
    let new_ease = c.ease.update(quality);

    if grade.is_lapse() {
        TypedCard::New(Card {
            entry: c.entry,
            ease: new_ease,
            added: c.added,
            body_hash: c.body_hash,
            state: New {
                due: today,
                lapses: c.state.lapses + 1,
            },
        })
    } else {
        let new_reps = c.state.reps + 1;
        let raw_interval = (c.state.interval as f64 * new_ease.inner()).round() as u32;
        let interval = raw_interval.min(max_interval);

        TypedCard::Mature(Card {
            entry: c.entry,
            ease: new_ease,
            added: c.added,
            body_hash: c.body_hash,
            state: Mature {
                reps: new_reps,
                interval,
                due: today + TimeDelta::days(interval as i64),
                lapses: c.state.lapses,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ease::Ease;
    use crate::entry::Entry;
    use std::path::PathBuf;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()
    }

    fn new_card(lapses: u32) -> TypedCard {
        TypedCard::New(Card {
            entry: Entry::File(PathBuf::from("src/main.rs")),
            ease: Ease::default_ease(),
            added: today(),
            body_hash: None,
            state: New {
                due: today(),
                lapses,
            },
        })
    }

    fn learning_card(reps: u32, interval: u32, lapses: u32) -> TypedCard {
        TypedCard::Learning(Card {
            entry: Entry::File(PathBuf::from("src/main.rs")),
            ease: Ease::default_ease(),
            added: today(),
            body_hash: None,
            state: Learning {
                reps,
                interval,
                due: today(),
                lapses,
            },
        })
    }

    fn mature_card(reps: u32, interval: u32, lapses: u32) -> TypedCard {
        TypedCard::Mature(Card {
            entry: Entry::File(PathBuf::from("src/main.rs")),
            ease: Ease::default_ease(),
            added: today(),
            body_hash: None,
            state: Mature {
                reps,
                interval,
                due: today(),
                lapses,
            },
        })
    }

    #[test]
    fn new_blank_stays_new() {
        let result = sm2_update(new_card(0), Grade::Blank, today(), 365);
        if let TypedCard::New(c) = result {
            assert_eq!(
                c.state.due,
                today(),
                "failed New card should be due today, not tomorrow"
            );
            assert_eq!(c.state.lapses, 0);
        } else {
            panic!("expected New");
        }
    }

    #[test]
    fn new_hard_to_learning() {
        let result = sm2_update(new_card(0), Grade::Hard, today(), 365);
        if let TypedCard::Learning(c) = result {
            assert_eq!(c.state.reps, 1);
            assert_eq!(c.state.interval, 1);
        } else {
            panic!("expected Learning");
        }
    }

    #[test]
    fn new_good_to_learning() {
        let result = sm2_update(new_card(0), Grade::Good, today(), 365);
        assert!(matches!(result, TypedCard::Learning(_)));
    }

    #[test]
    fn new_easy_to_learning() {
        let result = sm2_update(new_card(0), Grade::Easy, today(), 365);
        assert!(matches!(result, TypedCard::Learning(_)));
    }

    #[test]
    fn learning_reps1_good_interval_6() {
        let result = sm2_update(learning_card(1, 1, 0), Grade::Good, today(), 365);
        if let TypedCard::Learning(c) = result {
            assert_eq!(c.state.reps, 2);
            assert_eq!(c.state.interval, 6);
        } else {
            panic!("expected Learning");
        }
    }

    #[test]
    fn learning_reps2_good_grows_interval() {
        let result = sm2_update(learning_card(2, 6, 0), Grade::Good, today(), 365);
        match result {
            TypedCard::Learning(c) => {
                assert_eq!(c.state.reps, 3);
                assert!(c.state.interval > 6);
            }
            TypedCard::Mature(c) => {
                assert_eq!(c.state.reps, 3);
                assert!(c.state.interval >= 21);
            }
            _ => panic!("expected Learning or Mature"),
        }
    }

    #[test]
    fn learning_blank_lapses_to_new() {
        let result = sm2_update(learning_card(2, 6, 0), Grade::Blank, today(), 365);
        if let TypedCard::New(c) = result {
            assert_eq!(c.state.lapses, 1);
            assert_eq!(c.state.due, today(), "lapsed card should be due today");
        } else {
            panic!("expected New");
        }
    }

    #[test]
    fn mature_good_grows() {
        let result = sm2_update(mature_card(5, 30, 0), Grade::Good, today(), 365);
        if let TypedCard::Mature(c) = result {
            assert!(c.state.interval > 30);
            assert_eq!(c.state.reps, 6);
        } else {
            panic!("expected Mature");
        }
    }

    #[test]
    fn mature_blank_lapses_to_new() {
        let result = sm2_update(mature_card(5, 30, 2), Grade::Blank, today(), 365);
        if let TypedCard::New(c) = result {
            assert_eq!(c.state.lapses, 3);
            assert_eq!(c.state.due, today(), "lapsed card should be due today");
        } else {
            panic!("expected New");
        }
    }

    #[test]
    fn ease_clamping_after_repeated_blanks() {
        let mut card = new_card(0);
        for _ in 0..50 {
            card = sm2_update(card, Grade::Blank, today(), 365);
        }
        assert!(card.ease().inner() >= 1.3);
    }

    #[test]
    fn interval_respects_max() {
        let max = 30;
        let result = sm2_update(mature_card(10, 29, 0), Grade::Easy, today(), max);
        if let TypedCard::Mature(c) = result {
            assert!(c.state.interval <= max);
        } else {
            panic!("expected Mature");
        }
    }

    #[test]
    fn change_reset_preserves_ease_and_lapses() {
        let card = mature_card(5, 30, 3);
        let original_ease = card.ease();
        let result = change_reset(card, today());
        if let TypedCard::New(c) = result {
            assert_eq!(c.state.due, today());
            assert_eq!(c.state.lapses, 3);
            assert_eq!(c.ease, original_ease);
        } else {
            panic!("expected New");
        }
    }

    #[test]
    fn retired_card_unchanged() {
        let card = TypedCard::Retired(Card {
            entry: Entry::File(PathBuf::from("src/main.rs")),
            ease: Ease::default_ease(),
            added: today(),
            body_hash: None,
            state: Retired,
        });
        let result = sm2_update(card.clone(), Grade::Good, today(), 365);
        assert_eq!(result, card);
    }

    #[test]
    fn learning_to_mature_boundary_at_21() {
        let result = sm2_update(learning_card(2, 8, 0), Grade::Good, today(), 365);
        if let TypedCard::Learning(c) = &result {
            assert_eq!(
                c.state.interval, 20,
                "round(8 * ~2.5) should be 20 -> Learning"
            );
        } else {
            panic!("expected Learning for interval=20, got {:?}", result);
        }

        let result = sm2_update(learning_card(2, 9, 0), Grade::Good, today(), 365);
        if let TypedCard::Mature(c) = &result {
            assert!(
                c.state.interval >= 21,
                "round(9 * ~2.5) should be >=21 -> Mature"
            );
        } else {
            panic!("expected Mature for interval>=21, got {:?}", result);
        }
    }

    #[test]
    fn ease_update_formula_correctness() {
        let ease = Ease::default_ease();
        let updated = ease.update(4.0);
        assert!(
            (updated.inner() - 2.5).abs() < 0.001,
            "quality=4 should leave ease unchanged, got {}",
            updated.inner()
        );

        let updated = ease.update(5.0);
        assert!(
            (updated.inner() - 2.6).abs() < 0.001,
            "quality=5 should increase ease by 0.1, got {}",
            updated.inner()
        );

        let updated = ease.update(3.0);
        assert!(
            (updated.inner() - 2.36).abs() < 0.001,
            "quality=3 should decrease ease to ~2.36, got {}",
            updated.inner()
        );
    }

    #[test]
    fn max_interval_zero_clamps_everything() {
        let result = sm2_update(new_card(0), Grade::Good, today(), 0);
        if let TypedCard::Learning(c) = result {
            assert_eq!(
                c.state.interval, 0,
                "max_interval=0 should clamp interval to 0"
            );
        } else {
            panic!("expected Learning, got {:?}", result);
        }
    }
}
