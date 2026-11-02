//! The effect card: as planned, rework and patterns reused, against the 14 days before.

use super::super::super::outcomes::{percent, share};
use super::super::table::{Row, Table};
use super::super::{Period, Window};
use super::change::Change;
use super::{Card, DAYS};
use crate::terminal::canvas::{clip, join, pad, Styled, DIM};
use crate::terminal::classic::{GREEN, RED};
use crate::terminal::ink::Ink;
use crate::text::line::plural;
use ratatui::text::Span;

/// The fewest records a number of the effect counts before it shows a change.
const FEW: usize = 10;

/// One number of the effect: its label, its share in each period, what its base counts, what its part
/// holds of the base, and whether more is better.
struct Measure {
    label: &'static str,
    last: Share,
    before: Share,
    noun: &'static str,
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

    /// `part` in `base`, in whole percent; none with no records.
    fn percent(self) -> Option<i64> {
        (self.base > 0).then(|| percent(self.part, self.base))
    }

    /// The percent as the card shows it.
    fn text(self) -> String {
        share(self.part, self.base)
    }
}

impl Measure {
    /// The measure `label` of `window`: the share of the steps done in each span that `count` counts.
    fn new<'a>(
        label: &'static str,
        count: fn(&Period<'a>) -> usize,
        window: &Window<'a>,
        part: &'static str,
        up: bool,
    ) -> Measure {
        let share = |p: &Period<'a>| Share::new(count(p), p.done().len());
        Measure {
            label,
            last: share(&window.last),
            before: share(&window.before),
            noun: "step",
            part,
            up,
        }
    }

    /// The share of the asks done in each span of `window` that used a pattern (`DESIGN.md`, decision
    /// 19).
    fn reuse(window: &Window) -> Measure {
        let share = |p: &Period| {
            let reuse = p.reuse();
            Share::new(reuse.reused, reuse.asks)
        };
        Measure {
            label: "PATTERNS REUSED",
            last: share(&window.last),
            before: share(&window.before),
            noun: "ask",
            part: "used a pattern",
            up: true,
        }
    }

    /// `of <n> done,` of steps, `of <n> asks,` of another noun.
    fn of(&self) -> String {
        match self.noun {
            "step" => format!("of {} done,", self.last.base),
            noun => format!("of {},", plural(self.last.base, noun)),
        }
    }

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
        let text = format!("{before}% → {last}%  {}", Change { before, last }.arrow());
        let tone = match (moved.abs() < 3, (moved > 0) == self.up) {
            (true, _) => DIM,
            (false, true) => GREEN,
            (false, false) => RED,
        };
        Some(ink.paint(tone, &text))
    }
}

/// The effect: as planned, rework and patterns reused in the last 14 days, against the 14 days before,
/// each with what its share counts, on two lines when one is too narrow.
pub(super) fn effect(window: &Window, inside: usize, ink: Ink) -> Card {
    let measures = [
        Measure::new("AS PLANNED", Period::planned, window, "no change", true),
        Measure::new("REWORK", Period::reworked, window, "revised", false),
        Measure::reuse(window),
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
    let counts = |m: &Measure| format!("{} {}", m.of(), m.part);
    match measures.iter().all(|m| counts(m).chars().count() < column) {
        true => face.push(join(measures.iter().map(|m| cell(ink.dim(&counts(m)))))),
        false => face.extend([
            join(measures.iter().map(|m| cell(ink.dim(&m.of())))),
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
                plural(m.last.base, m.noun),
            ],
        })
        .collect();
    Card {
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
