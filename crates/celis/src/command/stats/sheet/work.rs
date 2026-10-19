//! The work cards: the steps done each day, the time a step took, and the time of each project.

use super::super::table::{Row, Table};
use super::daily::{chart, days};
use super::{nothing, Sheet, DAY, DAYS};
use crate::progress::{Grade, Period};
use crate::terminal::canvas::{cut as text_cut, visible, Styled, BOLD};
use crate::terminal::glyph::bar;
use crate::terminal::ink::Ink;
use crate::text::clock::{day, took};
use celis_trails::Step;
use std::collections::BTreeMap;

/// The work: the steps done each day of the last 14, as columns, and their total against the 14 days
/// before when those hold a done step.
pub(super) fn work(
    steps: &celis_trails::Steps,
    before: &Period,
    now: u64,
    inside: usize,
    ink: Ink,
) -> Sheet {
    let starts = days(now);
    let counts: Vec<usize> = starts
        .iter()
        .map(|&s| Period::new(steps, s, s + DAY).done().len())
        .collect();
    let total: usize = counts.iter().sum();
    let earlier = before.done().len();
    let moved = match (earlier, total >= earlier) {
        (0, _) => None,
        (_, true) => Some(format!("▲ {} vs {DAYS} days before", total - earlier)),
        (_, false) => Some(format!("▼ {} vs {DAYS} days before", earlier - total)),
    };
    let moved = moved.map(|m| Styled::from(ink.dim(&m)));
    let side: Vec<Styled> = std::iter::once(ink.dim("steps done").into())
        .chain(moved.clone())
        .collect();
    let mut face = ink.beside(&total.to_string(), &side, 2);
    if ink.plain {
        face.extend(moved);
    }
    face.push(Styled::default());
    if total == 0 {
        face.push(ink.dim(&nothing("done")).into());
    } else {
        face.extend(chart(&counts, &starts, inside, ink));
    }
    let rows = starts
        .iter()
        .zip(&counts)
        .rev()
        .map(|(&s, n)| Row {
            id: None,
            cells: vec![day(s), n.to_string()],
        })
        .collect();
    Sheet {
        title: format!("03 WORK · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Day", "Steps done"],
            rows,
            wide: 0,
        },
    }
}

/// The steps done in `period` with no step under them, which the time a step took counts.
fn leaves<'a>(period: &Period<'a>) -> Vec<Step<'a>> {
    period
        .done()
        .into_iter()
        .filter(|s| s.children().is_empty())
        .collect()
}

/// The time a step took: its median and its spread over five spans, as bars.
pub(super) fn time(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let mut done: Vec<(u64, Step)> = leaves(last)
        .into_iter()
        .filter_map(|s| Some((Grade::of(s)?.took, s)))
        .collect();
    done.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    let spans: [(&str, u64); 5] = [
        ("< 5m", 300),
        ("5-15m", 900),
        ("15m-1h", 3_600),
        ("1-4h", 14_400),
        ("> 4h", u64::MAX),
    ];
    let mut counts = [0usize; 5];
    for (t, _) in &done {
        let at = spans.iter().position(|(_, top)| t < top).unwrap_or(4);
        counts[at] += 1;
    }
    let most = counts.iter().copied().max().unwrap_or(0);
    let width = inside.saturating_sub(13);
    let face = match done.get(done.len() / 2) {
        None => vec![ink.dim(&nothing("done")).into()],
        Some((median, _)) => {
            let median = vec![
                ink.dim("median"),
                " ".into(),
                ink.paint(BOLD, &took(*median)),
            ];
            let mut face = vec![Styled::from(median), Styled::default()];
            for ((label, _), n) in spans.iter().zip(counts) {
                face.push(format!("{label:<7} {} {n:>3}", bar(n, most, width)).into());
            }
            face
        }
    };
    let rows = done
        .iter()
        .map(|(t, s)| Row {
            id: Some(s.id().to_string()),
            cells: vec![s.title().to_string(), took(*t)],
        })
        .collect();
    Sheet {
        title: "04 TIME A STEP TOOK".into(),
        face,
        table: Table {
            head: vec!["Step", "Took"],
            rows,
            wide: 0,
        },
    }
}

/// The projects: the time the steps done in the last 14 days took in each, most first, as bars.
pub(super) fn projects(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let mut spent: BTreeMap<String, (u64, usize)> = BTreeMap::new();
    for step in leaves(last) {
        let entry = spent.entry(step.record().anchor.name.clone()).or_default();
        entry.0 += Grade::of(step).map_or(0, |g| g.took);
        entry.1 += 1;
    }
    let mut spent: Vec<(String, (u64, usize))> = spent.into_iter().collect();
    spent.sort_by_key(|(name, (t, _))| (std::cmp::Reverse(*t), name.clone()));
    let most = spent.first().map_or(0, |(_, (t, _))| *t as usize);
    let label = 14.min(inside / 2);
    let width = inside.saturating_sub(label + 9);
    let mut face: Vec<Styled> = spent
        .iter()
        .take(4)
        .map(|(name, (t, _))| {
            let name = text_cut(name, label);
            let pad = " ".repeat(label - visible(&name));
            let rest = format!("{pad} {} {:>6}", bar(*t as usize, most, width), took(*t));
            Styled::from(vec![ink.paint(BOLD, &name), rest.into()])
        })
        .collect();
    if face.is_empty() {
        face.push(ink.dim(&nothing("done")).into());
    }
    let rows = spent
        .iter()
        .map(|(name, (t, n))| Row {
            id: None,
            cells: vec![name.clone(), took(*t), n.to_string()],
        })
        .collect();
    Sheet {
        title: "06 PROJECTS · TIME".into(),
        face,
        table: Table {
            head: vec!["Project", "Time", "Steps"],
            rows,
            wide: 0,
        },
    }
}
