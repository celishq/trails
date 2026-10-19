//! The last 14 days as a card draws them: a column a day.

use super::{DAY, DAYS};
use crate::terminal::canvas::Styled;
use crate::terminal::glyph::columns;
use crate::terminal::ink::Ink;
use crate::text::clock::{day, midnight};
use ratatui::text::Span;

/// What a column shows for a day that counts none.
const ZERO: &str = "·";

/// The start of each of the last 14 days up to `now`, oldest first.
pub(super) fn days(now: u64) -> Vec<u64> {
    let today = midnight(now);
    (0..DAYS)
        .rev()
        .map(|i| today.saturating_sub(i * DAY))
        .collect()
}

/// `counts` as columns two rows high, a column a day of `starts`, then the first day and `today` under
/// them, dim.
pub(super) fn chart(counts: &[usize], starts: &[u64], inside: usize, ink: Ink) -> Vec<Styled> {
    let mut lines: Vec<Styled> = columns(counts, 2, ZERO)
        .iter()
        .map(|row| dotted(row, ink))
        .collect();
    let first = day(starts[0]);
    let pad = " ".repeat(27usize.min(inside).saturating_sub(first.len() + 5));
    lines.push(ink.dim(&format!("{first}{pad}today")).into());
    lines
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
