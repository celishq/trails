//! How a set of steps ended.

use crate::{State, Step};

/// How many of a set of steps are done, dropped and expired, and how many a later step revised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    pub done: usize,
    pub dropped: usize,
    pub expired: usize,
    pub revised: usize,
}

impl Tally {
    pub fn of(steps: &[Step]) -> Tally {
        let mut tally = Tally::default();
        for step in steps {
            match step.state() {
                State::Done => tally.done += 1,
                State::Dropped => tally.dropped += 1,
                State::Expired => tally.expired += 1,
                State::Open | State::Working => {}
            }
            tally.revised += usize::from(!step.revised().is_empty());
        }
        tally
    }

    /// How many are in `state`; none for an active one.
    pub fn count(self, state: State) -> usize {
        match state {
            State::Done => self.done,
            State::Dropped => self.dropped,
            State::Expired => self.expired,
            State::Open | State::Working => 0,
        }
    }
}
