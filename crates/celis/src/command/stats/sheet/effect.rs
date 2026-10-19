//! The effect card: as planned, rework and rules reused, against the 14 days before.

use super::super::table::{Row, Table};
use super::{Sheet, DAYS};
use crate::progress::Period;
use crate::terminal::canvas::{clip, join, pad, Styled, DIM, GREEN, RED};
use crate::terminal::ink::Ink;
use crate::text::line::plural;
use ratatui::text::Span;

/// The fewest records a number of the effect counts before it shows a change.
const FEW: usize = 10;

/// One number of the effect: its label, its share in each period, what its part holds of the steps
/// done, and whether more is better.
struct Measure {
    label: &'static str,
    last: Share,
    before: Share,
    part: &'static str,
    up: bool,
}

/// The records of one period that count toward a measure, `part`, among the records it counts, `base`.
#[derive(Clone, Copy)]
struct Share {
    part: usize,
    base: usize,
}

impl Share {
    fn new(part: usize, base: usize) -> Share {
        Share { part, base }
    }

    /// `part` in `base`, in percent; none with no records.
    fn percent(self) -> Option<i64> {
        (self.base > 0).then(|| (self.part * 100 / self.base) as i64)
    }

    /// The percent as the card shows it, `—` with no records.
    fn text(self) -> String {
        self.percent().map_or("—".to_string(), |v| format!("{v}%"))
    }
}

impl Measure {
    /// The change from the period before, coloured green when better and red when worse, dim when it
    /// moved little; none when the period before counts none, `· few records` when a period counts
    /// fewer than `FEW`.
    fn change(&self, ink: Ink) -> Option<Span<'static>> {
        if self.before.base == 0 {
            return None;
        }
        let (Some(last), Some(before)) = (self.last.percent(), self.before.percent()) else {
            return Some(ink.dim("· few records"));
        };
        if self.last.base < FEW || self.before.base < FEW {
            return Some(ink.dim("· few records"));
        }
        let moved = last - before;
        let by = match moved.signum() {
            0 => "=".to_string(),
            1 => format!("▲ {moved}"),
            _ => format!("▼ {}", -moved),
        };
        let text = format!("{before}% → {last}%  {by}");
        let tone = match (moved.abs() < 3, (moved > 0) == self.up) {
            (true, _) => DIM,
            (false, true) => GREEN,
            (false, false) => RED,
        };
        Some(ink.paint(tone, &text))
    }
}

/// The effect: as planned, rework and rules reused in the last 14 days, against the 14 days before,
/// each with what its share counts, on two lines when one is too narrow.
pub(super) fn effect(last: &Period, before: &Period, inside: usize, ink: Ink) -> Sheet {
    let done = |p: &Period| p.done().len();
    let measures = [
        Measure {
            label: "AS PLANNED",
            last: Share::new(last.planned(), done(last)),
            before: Share::new(before.planned(), done(before)),
            part: "no change",
            up: true,
        },
        Measure {
            label: "REWORK",
            last: Share::new(last.reworked(), done(last)),
            before: Share::new(before.reworked(), done(before)),
            part: "revised",
            up: false,
        },
        Measure {
            label: "RULES REUSED",
            last: Share::new(last.ruled(), done(last)),
            before: Share::new(before.ruled(), done(before)),
            part: "used a rule",
            up: true,
        },
    ];
    let column = (inside / measures.len()).max(1);
    let cell = |text: Span<'static>| pad(clip(&Styled::from(text), column - 1), column);
    let mut face = vec![join(measures.iter().map(|m| cell(ink.dim(m.label))))];
    let bigs: Vec<Vec<Span>> = measures.iter().map(|m| ink.big(&m.last.text())).collect();
    for r in 0..bigs[0].len() {
        face.push(join(bigs.iter().map(|b| cell(b[r].clone()))));
    }
    let changes: Vec<Option<Span>> = measures.iter().map(|m| m.change(ink)).collect();
    let compared = changes.iter().any(Option::is_some);
    if compared {
        face.push(join(
            changes.into_iter().map(|c| cell(c.unwrap_or_default())),
        ));
    }
    let counts = |m: &Measure| format!("of {} done, {}", m.last.base, m.part);
    match measures.iter().all(|m| counts(m).chars().count() < column) {
        true => face.push(join(measures.iter().map(|m| cell(ink.dim(&counts(m)))))),
        false => face.extend([
            join(
                measures
                    .iter()
                    .map(|m| cell(ink.dim(&format!("of {} done,", m.last.base)))),
            ),
            join(measures.iter().map(|m| cell(ink.dim(m.part)))),
        ]),
    }
    let rows = measures
        .iter()
        .map(|m| Row {
            id: None,
            cells: vec![
                title_case(m.label),
                m.last.text(),
                m.before.text(),
                plural(m.last.base, "step"),
            ],
        })
        .collect();
    Sheet {
        title: match compared {
            true => format!("00 EFFECT · LAST {DAYS} DAYS vs {DAYS} BEFORE"),
            false => format!("00 EFFECT · LAST {DAYS} DAYS"),
        },
        face,
        table: Table {
            head: vec!["Measure", "Last 14 days", "14 days before", "Base"],
            rows,
            wide: 0,
        },
    }
}

/// `label` in sentence case, as `As planned`.
fn title_case(label: &str) -> String {
    let lower = label.to_lowercase();
    let mut chars = lower.chars();
    chars.next().map_or(String::new(), |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}
