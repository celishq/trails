//! The patterns card: the pattern uses and the patterns used most.

use super::super::table::{Row, Table};
use super::super::Window;
use super::daily::Daily;
use super::face::face;
use super::{nothing, Card, DAYS};
use crate::progress::Model;
use crate::terminal::ink::Ink;

/// The patterns used: the pattern uses of each day of `window` against the 14 days before, then the
/// patterns used most.
pub(super) fn patterns(model: &Model, window: &Window, inside: usize, ink: Ink) -> Card {
    let (patterns, reuse) = (model.patterns, model.reuse);
    let used = reuse.used(patterns);
    let top = used.iter().take(3);
    let top: Vec<(String, usize)> = top
        .map(|(p, n)| (p.record.brief().trim_start_matches("When ").into(), *n))
        .collect();
    let none = match used.is_empty() {
        true => "no pattern used yet".to_string(),
        false => nothing("pattern used"),
    };
    let daily = Daily::new(window, |p| reuse.uses_of(&p.added(), patterns));
    let face = face(&daily, ("pattern uses", &none), &top, inside, ink);
    let rows = used
        .iter()
        .map(|(p, n)| Row {
            id: Some(p.note.id.clone()),
            cells: vec![p.record.content.clone(), n.to_string()],
        })
        .collect();
    Card {
        title: format!("01 PATTERNS USED · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Rule", "Used"],
            rows,
            wide: 0,
        },
    }
}
