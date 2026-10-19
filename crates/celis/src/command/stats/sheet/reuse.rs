//! The reuse cards: the follows and the steps followed most, and the rule uses and the rules used most.

use super::super::table::{Row, Table};
use super::daily::{item, Daily};
use super::{Sheet, DAYS};
use crate::progress::{Period, Reuse};
use crate::terminal::ink::Ink;
use celis_trails::{Rules, Steps};

/// The reuse: the follows of each of the last 14 days up to `now` against `before`, then the steps
/// followed most.
pub(super) fn followed(
    reuse: &Reuse,
    all: &Steps,
    before: &Period,
    now: u64,
    inside: usize,
    ink: Ink,
) -> Sheet {
    let steps = reuse.followed();
    let top = steps.iter().take(3);
    let top = top
        .map(|&s| item(s.title(), reuse.followers(s), inside))
        .collect();
    let none = match steps.is_empty() {
        true => "no step followed another yet".to_string(),
        false => format!("no follow in {DAYS} days"),
    };
    let daily = Daily::new(all, now, before, Period::follows);
    let face = daily.face("follows", &none, top, inside, ink);
    let rows = steps
        .iter()
        .map(|&s| Row {
            id: Some(s.id().to_string()),
            cells: vec![
                s.title().to_string(),
                s.record().anchor.name.clone(),
                reuse.followers(s).to_string(),
            ],
        })
        .collect();
    Sheet {
        title: format!("01 REUSE · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Step", "Project", "Followed"],
            rows,
            wide: 0,
        },
    }
}

/// The rules used: the rule uses of each of the last 14 days up to `now` against `before`, then the
/// rules used most.
pub(super) fn rules(
    rules: &Rules,
    reuse: &Reuse,
    all: &Steps,
    before: &Period,
    now: u64,
    inside: usize,
    ink: Ink,
) -> Sheet {
    let used = reuse.used(rules);
    let top = used.iter().take(3);
    let top = top.map(|(_, d, n)| item(d.brief().trim_start_matches("When "), *n, inside));
    let none = match used.is_empty() {
        true => "no rule used yet".to_string(),
        false => format!("no rule used in {DAYS} days"),
    };
    let daily = Daily::new(all, now, before, Period::uses);
    let face = daily.face("rule uses", &none, top.collect(), inside, ink);
    let rows = used
        .iter()
        .map(|(note, d, n)| Row {
            id: Some(note.id.clone()),
            cells: vec![d.content.clone(), n.to_string()],
        })
        .collect();
    Sheet {
        title: format!("02 RULES USED · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Rule", "Used"],
            rows,
            wide: 0,
        },
    }
}
