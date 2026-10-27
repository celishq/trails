//! One frame of the stats board: the month it reads, the time it draws at, and where the person is.

use super::kind::Kind;
use super::measures::Cell;
use super::month::Month;
use super::outcomes::Outcomes;
use super::total::Total;
use crate::terminal::grid::Grid;
use crate::terminal::unfold::ease;
use crate::text::clock::day;
use ratatui::text::Span;

/// A frame of the board: the month it reads, what the store holds of all time, the seconds since the board opened, its width, the day
/// picked, the section picked, and the sections opened.
pub(super) struct Moment<'a> {
    pub month: &'a Month,
    pub total: &'a Total,
    pub t: f64,
    pub width: u16,
    pub day: usize,
    pub focus: usize,
    pub open: [bool; 4],
}

impl Moment<'_> {
    /// How far a part that starts `start` seconds in has faded in.
    pub fn fade(&self, start: f64) -> f64 {
        ease((self.t - start) / 0.45)
    }

    /// How the steps of the last 14 days ended.
    pub fn now(&self) -> Outcomes {
        self.month.now()
    }

    /// How the steps of the prior 14 days ended.
    pub fn prior(&self) -> Outcomes {
        self.month.prior()
    }

    /// The value `value` gives of the prior 14 days, none when they resolved no step.
    pub fn before(&self, value: impl Fn(Outcomes) -> i64) -> Option<i64> {
        let prior = self.prior();
        (prior.resolved() > 0).then(|| value(prior))
    }

    /// The cell of a measure: `lead`, then its change as `kind` reads it.
    pub fn measure(
        &self,
        lead: Vec<Span<'static>>,
        (now, prior): (Option<i64>, Option<i64>),
        kind: Kind,
        up: bool,
    ) -> Cell {
        let change = kind.change(now, prior, up).into_iter().flatten().collect();
        (lead, change)
    }

    /// The grid the board lays out on.
    pub fn grid(&self) -> Grid {
        Grid::of(self.width)
    }

    /// The column where the bars of a card at `x`, `w` wide start, the same in every card: the text
    /// column two tracks in, or the first track column past a label as wide as the widest one.
    pub fn bars(&self, x: u16, w: u16) -> u16 {
        let label = if w >= 44 { LABEL } else { LABEL / 2 };
        self.grid().inside(x, 2, x + 2 + label + 2)
    }

    /// The day `n` of the month, as `Oct 9`.
    pub fn date(&self, n: usize) -> String {
        day(self.month.start(n))
    }
}

/// The columns the widest label before a bar takes: `Oct 13 – Oct 19`.
const LABEL: u16 = 15;

/// `seconds` as whole minutes, `25m`, or hours and minutes, `4h 32m`.
pub(super) fn minutes(seconds: u64) -> String {
    let m = (seconds + 30) / 60;
    match m {
        m if m < 60 => format!("{m}m"),
        m => format!("{}h {}m", m / 60, m % 60),
    }
}

/// `word`, `s` after it when `n` is not one: `steps`.
pub(super) fn plural(n: usize, word: &str) -> String {
    match n {
        1 => word.to_string(),
        _ => format!("{word}s"),
    }
}
