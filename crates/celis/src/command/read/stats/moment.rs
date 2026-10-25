//! One frame of the stats board: the month it reads, the time it draws at, and where the person is.

use super::kind::Kind;
use super::month::Month;
use super::outcomes::Outcomes;
use super::sheet::Group;
use super::total::Total;
use crate::terminal::canvas::{paint, DIM};
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

    /// The groups of a measure: each text of `lead` alone, a gap, then its change as `kind` reads it.
    pub fn measure(
        &self,
        lead: Vec<Span<'static>>,
        (now, prior): (Option<i64>, Option<i64>),
        kind: Kind,
        up: bool,
    ) -> Vec<Group> {
        let mut groups: Vec<Group> = lead.into_iter().map(|s| vec![s]).collect();
        groups.push(vec![paint(DIM, "  ")]);
        groups.extend(kind.change(now, prior, up));
        groups
    }

    /// The day `n` of the month, as `Oct 9`.
    pub fn date(&self, n: usize) -> String {
        day(self.month.start(n))
    }
}

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

/// `n` and `word`, `s` after it when `n` is not one: `2 reworked steps`.
pub(super) fn counted(n: usize, word: &str) -> String {
    match n {
        1 => format!("{n} {word}"),
        _ => format!("{n} {word}s"),
    }
}
