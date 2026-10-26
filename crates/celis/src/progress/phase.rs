//! How a step or trail shows in a view.

use celis_trails::{State, Step};

/// How a step or trail shows in a view: its state, with an open one that works shown as started, and an
/// open one of an ended trail shown as stale.
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

    /// Whether work on it may go on: open and not stale.
    pub fn active(self) -> bool {
        matches!(self, Phase::Open | Phase::Started)
    }

    /// Whether work on it stopped: done, dropped or stale.
    pub fn ended(self) -> bool {
        !self.active()
    }

    /// Whether its work ended: done or dropped.
    pub fn closed(self) -> bool {
        matches!(self, Phase::Done | Phase::Dropped)
    }
}
