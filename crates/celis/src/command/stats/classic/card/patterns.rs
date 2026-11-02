//! The patterns card: the pattern uses and the patterns used most.

use super::super::table::{Row, Table};
use super::super::Window;
use super::daily::Daily;
use super::face::face;
use super::{nothing, Card, DAYS};
use crate::progress::Model;
use crate::terminal::ink::Ink;
use celis_patterns::Pattern;

/// The patterns used: the pattern uses of the asks done in each day of `window` against the 14 days
/// before, then the live patterns used most in the last 14 days (`DESIGN.md`, decision 19).
pub(super) fn patterns(model: &Model, window: &Window, inside: usize, ink: Ink) -> Card {
    let patterns = model.patterns;
    let reuse = window.last.reuse();
    let used: Vec<(Pattern, usize)> = reuse
        .top()
        .into_iter()
        .filter_map(|(id, n)| Some((patterns.find(id).ok()?, n)))
        .collect();
    let top = used.iter().take(3);
    let top: Vec<(String, usize)> = top
        .map(|(p, n)| (p.record.brief().trim_start_matches("When ").into(), *n))
        .collect();
    let none = match model.reuse.used(patterns).is_empty() {
        true => "no pattern used yet".to_string(),
        false => nothing("pattern used"),
    };
    let daily = Daily::new(window, |p| p.reuse().count());
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
