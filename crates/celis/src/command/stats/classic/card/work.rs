//! The work card: the steps done each day.

use super::super::table::{Row, Table};
use super::super::Window;
use super::daily::Daily;
use super::face::face;
use super::{nothing, Card, DAYS};
use crate::terminal::ink::Ink;
use crate::text::clock::day;

/// The work: the steps done each day of the last 14, as columns, and their total against the 14 days
/// before when those hold a done step.
pub(super) fn work(window: &Window, inside: usize, ink: Ink) -> Card {
    let daily = Daily::new(window, |p| p.done().len());
    let face = face(
        &daily,
        ("steps done", &nothing("step done")),
        &[],
        inside,
        ink,
    );
    let (starts, counts) = (&daily.starts, &daily.counts);
    let rows = starts
        .iter()
        .zip(counts)
        .rev()
        .map(|(&s, n)| Row {
            id: None,
            cells: vec![day(s), n.to_string()],
        })
        .collect();
    Card {
        title: format!("02 WORK · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Day", "Steps done"],
            rows,
            wide: 0,
        },
    }
}
