//! The time card: the time a step took.

use super::super::glyph::bar;
use super::super::table::{Row, Table};
use super::super::Period;
use super::{nothing, Card};
use crate::progress::Grade;
use crate::terminal::canvas::{Styled, BOLD};
use crate::terminal::ink::Ink;
use crate::text::clock::took;
use celis_trails::Step;

/// The time a step took: its median and its spread over five spans, as bars.
pub(super) fn time(last: &Period, inside: usize, ink: Ink) -> Card {
    let mut done: Vec<(u64, Step)> = last
        .leaves()
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
        None => vec![ink.dim(&nothing("step done")).into()],
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
    Card {
        title: "03 TIME A STEP TOOK".into(),
        face,
        table: Table {
            head: vec!["Step", "Took"],
            rows,
            wide: 0,
        },
    }
}
