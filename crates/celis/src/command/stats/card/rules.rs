//! The rules card: the rule uses and the rules used most.

use super::super::table::{Row, Table};
use super::daily::Daily;
use super::face::face;
use super::{nothing, Card, DAYS};
use crate::progress::{Model, Window};
use crate::terminal::ink::Ink;

/// The rules used: the rule uses of each day of `window` against the 14 days before, then the
/// rules used most.
pub(super) fn rules(model: &Model, window: &Window, inside: usize, ink: Ink) -> Card {
    let (rules, reuse) = (model.rules, model.reuse);
    let used = reuse.used(rules);
    let top = used.iter().take(3);
    let top: Vec<(String, usize)> = top
        .map(|(_, d, n)| (d.brief().trim_start_matches("When ").into(), *n))
        .collect();
    let none = match used.is_empty() {
        true => "no rule used yet".to_string(),
        false => nothing("rule used"),
    };
    let daily = Daily::new(window, |p| reuse.uses_of(&p.added(), rules));
    let face = face(&daily, ("rule uses", &none), &top, inside, ink);
    let rows = used
        .iter()
        .map(|(note, d, n)| Row {
            id: Some(note.id.clone()),
            cells: vec![d.content.clone(), n.to_string()],
        })
        .collect();
    Card {
        title: format!("02 RULES USED · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Rule", "Used"],
            rows,
            wide: 0,
        },
    }
}
