//! The mark a view prints for a step or trail.

use celis_trails::{State, Step};

/// The mark a view prints for a step or trail: its state, or on an open one, started while it works and
/// stale once its trail ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Open,
    Started,
    Done,
    Dropped,
    Stale,
}

impl Phase {
    pub fn of(step: &Step) -> Phase {
        match step.state() {
            State::Done => Phase::Done,
            State::Dropped => Phase::Dropped,
            State::Open if step.stale() => Phase::Stale,
            State::Open if step.works() => Phase::Started,
            State::Open => Phase::Open,
        }
    }

    /// The word a view gives it: its state, or `stale`.
    pub fn text(self) -> &'static str {
        match self {
            Phase::Open | Phase::Started => State::Open.text(),
            Phase::Done => State::Done.text(),
            Phase::Dropped => State::Dropped.text(),
            Phase::Stale => "stale",
        }
    }

    /// Whether the work is still to do: open or started.
    pub fn active(self) -> bool {
        matches!(self, Phase::Open | Phase::Started)
    }

    /// Whether the work is over: done, dropped or stale.
    pub fn ended(self) -> bool {
        !self.active()
    }
}
