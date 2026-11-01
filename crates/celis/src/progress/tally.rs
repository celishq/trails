//! How a set of steps ended.

use crate::progress::Phase;
use celis_trails::Step;

/// How many of a set of steps are done, dropped and stale, and how many a later step revised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    pub done: usize,
    pub dropped: usize,
    pub stale: usize,
    pub revised: usize,
}

impl Tally {
    pub fn of(steps: &[Step]) -> Tally {
        let mut tally = Tally::default();
        for step in steps {
            match Phase::of(step) {
                Phase::Done => tally.done += 1,
                Phase::Dropped => tally.dropped += 1,
                Phase::Stale => tally.stale += 1,
                Phase::Open | Phase::Started => {}
            }
            tally.revised += usize::from(!step.revised().is_empty());
        }
        tally
    }

    /// How many are in `state`; none for an active one.
    pub fn count(self, state: Phase) -> usize {
        match state {
            Phase::Done => self.done,
            Phase::Dropped => self.dropped,
            Phase::Stale => self.stale,
            Phase::Open | Phase::Started => 0,
        }
    }
}
