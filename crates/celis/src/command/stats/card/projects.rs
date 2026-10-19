//! The projects card: the time of each project.

use super::super::table::{Row, Table};
use super::{nothing, Card};
use crate::progress::{Grade, Period};
use crate::terminal::canvas::{cut, visible, Styled, BOLD};
use crate::terminal::glyph::bar;
use crate::terminal::ink::Ink;
use crate::text::clock::took;
use std::collections::BTreeMap;

/// The projects: the time the steps done in the last 14 days took in each, most first, as bars.
pub(super) fn projects(last: &Period, inside: usize, ink: Ink) -> Card {
    let mut spent: BTreeMap<String, (u64, usize)> = BTreeMap::new();
    for step in last.leaves() {
        let entry = spent.entry(step.record().anchor.name.clone()).or_default();
        entry.0 += Grade::of(step).map_or(0, |g| g.took);
        entry.1 += 1;
    }
    let mut spent: Vec<(String, (u64, usize))> = spent.into_iter().collect();
    spent.sort_by_key(|(name, (t, _))| (std::cmp::Reverse(*t), name.clone()));
    let most = spent.first().map_or(0, |(_, (t, _))| *t as usize);
    let label = 14.min(inside / 2);
    let shown = spent.iter().take(4);
    let time = shown.clone().map(|(_, (t, _))| visible(&took(*t))).max();
    let time = time.unwrap_or(0).max(6);
    let width = inside.saturating_sub(label + time + 3);
    let mut face: Vec<Styled> = shown
        .map(|(name, (t, _))| {
            let name = cut(name, label);
            let pad = " ".repeat(label - visible(&name));
            let bar = bar(*t as usize, most, width);
            let rest = format!("{pad} {bar} {:>time$}", took(*t));
            Styled::from(vec![ink.paint(BOLD, &name), rest.into()])
        })
        .collect();
    if face.is_empty() {
        face.push(ink.dim(&nothing("step done")).into());
    }
    let rows = spent
        .iter()
        .map(|(name, (t, n))| Row {
            id: None,
            cells: vec![name.clone(), took(*t), n.to_string()],
        })
        .collect();
    Card {
        title: "06 PROJECTS · TIME".into(),
        face,
        table: Table {
            head: vec!["Project", "Time", "Steps"],
            rows,
            wide: 0,
        },
    }
}
