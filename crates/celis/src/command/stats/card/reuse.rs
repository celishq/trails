//! The reuse card: the follows and the steps followed most.

use super::super::table::{Row, Table};
use super::daily::Daily;
use super::face::face;
use super::{nothing, Card, DAYS};
use crate::progress::{Model, Window};
use crate::terminal::ink::Ink;

/// The reuse: the follows of each day of `window` against the 14 days before, then the steps
/// followed most.
pub(super) fn reuse(model: &Model, window: &Window, inside: usize, ink: Ink) -> Card {
    let reuse = model.reuse;
    let steps = reuse.followed();
    let top = steps.iter().take(3);
    let top: Vec<(String, usize)> = top
        .map(|&s| (s.title().into(), reuse.followers(s)))
        .collect();
    let none = match steps.is_empty() {
        true => "no step followed another yet".to_string(),
        false => nothing("follow"),
    };
    let daily = Daily::new(window, |p| reuse.follows_of(&p.added()));
    let face = face(&daily, ("follows", &none), &top, inside, ink);
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
    Card {
        title: format!("01 REUSE · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Step", "Project", "Followed"],
            rows,
            wide: 0,
        },
    }
}
