//! The face the reuse, rules and work cards share.

use super::super::glyph::columns;
use super::super::window::DAYS;
use super::change::Change;
use super::daily::Daily;
use crate::terminal::canvas::{cut, visible, Styled};
use crate::terminal::ink::Ink;
use crate::text::clock::day;
use ratatui::text::Span;

/// What a column shows for a day that counts none.
const ZERO: &str = "·";

/// The characters the columns take: one a day, a space between two.
const WIDE: usize = DAYS as usize * 2 - 1;

/// The face of `daily`: its total big beside `what` and its change against the 14 days before, the days
/// as columns or `none` when all are zero, then each name of `top` with its count at the right.
pub(super) fn face(
    daily: &Daily,
    (what, none): (&str, &str),
    top: &[(String, usize)],
    inside: usize,
    ink: Ink,
) -> Vec<Styled> {
    let total: usize = daily.counts.iter().sum();
    let change = Change {
        before: daily.before as i64,
        last: total as i64,
    };
    let moved = (daily.before > 0).then(|| format!("{} vs {DAYS} days before", change.arrow()));
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
        let rows = columns(&daily.counts, 2, ZERO);
        face.extend(rows.iter().map(|row| dotted(row, ink)));
        let first = day(daily.starts[0]);
        let pad = " ".repeat(WIDE.min(inside).saturating_sub(first.len() + 5));
        face.push(ink.dim(&format!("{first}{pad}today")).into());
    }
    if !top.is_empty() {
        face.push(Styled::default());
        face.extend(top.iter().map(|(name, n)| item(name, *n, inside)));
    }
    face
}

/// `name` cut to leave room for `count`, which ends the line.
fn item(name: &str, count: usize, inside: usize) -> Styled {
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
