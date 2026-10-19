//! A count of each of the last 14 days, as the reuse, rules and work cards draw it.

use super::{DAY, DAYS};
use crate::progress::Period;
use crate::terminal::canvas::{cut, visible, Styled};
use crate::terminal::glyph::columns;
use crate::terminal::ink::Ink;
use crate::text::clock::{day, midnight};
use celis_trails::Steps;
use ratatui::text::Span;

/// What a column shows for a day that counts none.
const ZERO: &str = "·";

/// A count of each of the last 14 days: the start of each day, oldest first, its count, and the count
/// of the 14 days before.
pub(super) struct Daily {
    pub starts: Vec<u64>,
    pub counts: Vec<usize>,
    before: usize,
}

impl Daily {
    /// What `count` counts in each of the last 14 days up to `now`, and in `before`.
    pub fn new<'a>(
        steps: &'a Steps<'a>,
        now: u64,
        before: &Period<'a>,
        count: fn(&Period<'a>) -> usize,
    ) -> Daily {
        let today = midnight(now);
        let starts: Vec<u64> = (0..DAYS)
            .rev()
            .map(|i| today.saturating_sub(i * DAY))
            .collect();
        let counts = starts
            .iter()
            .map(|&s| count(&Period::new(steps, s, s + DAY)))
            .collect();
        Daily {
            starts,
            counts,
            before: count(before),
        }
    }

    /// The face of a card: the total big beside `what` and its change against the 14 days before, the
    /// days as columns or `none` when all are zero, then `top`.
    pub fn face(
        &self,
        what: &str,
        none: &str,
        top: Vec<Styled>,
        inside: usize,
        ink: Ink,
    ) -> Vec<Styled> {
        let total: usize = self.counts.iter().sum();
        let moved = match (self.before, total >= self.before) {
            (0, _) => None,
            (b, true) => Some(format!("▲ {} vs {DAYS} days before", total - b)),
            (b, false) => Some(format!("▼ {} vs {DAYS} days before", b - total)),
        };
        let moved = moved.map(|m| Styled::from(ink.dim(&m)));
        let side: Vec<Styled> = std::iter::once(ink.dim(what).into())
            .chain(moved.clone())
            .collect();
        let mut face = ink.beside(&total.to_string(), &side, 2);
        if ink.plain {
            face.extend(moved);
        }
        face.push(Styled::default());
        if total == 0 {
            face.push(ink.dim(none).into());
        } else {
            let rows = columns(&self.counts, 2, ZERO);
            face.extend(rows.iter().map(|row| dotted(row, ink)));
            let first = day(self.starts[0]);
            let pad = " ".repeat(27usize.min(inside).saturating_sub(first.len() + 5));
            face.push(ink.dim(&format!("{first}{pad}today")).into());
        }
        if !top.is_empty() {
            face.push(Styled::default());
            face.extend(top);
        }
        face
    }
}

/// One line of the top of a card: `name` cut to leave room for `count`, which ends the line.
pub(super) fn item(name: &str, count: usize, inside: usize) -> Styled {
    let count = format!("{count:>3}");
    let name = cut(name, inside.saturating_sub(visible(&count) + 2));
    let pad = " ".repeat(inside.saturating_sub(visible(&name) + visible(&count)));
    format!("{name}{pad}{count}").into()
}

/// `row` of the columns, each `ZERO` in it dim.
fn dotted(row: &str, ink: Ink) -> Styled {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (n, part) in row.split(ZERO).enumerate() {
        if n > 0 {
            spans.push(ink.dim(ZERO));
        }
        if !part.is_empty() {
            spans.push(part.to_string().into());
        }
    }
    Styled::from(spans)
}
