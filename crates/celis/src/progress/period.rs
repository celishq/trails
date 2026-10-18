//! The work of a store in one span of time.

use super::{Flags, Tally};
use celis_trails::{Step, Steps, Used};

/// The work of a store from `from` up to `to`: the steps done, added and closed in that time, and the
/// follows they made. A trail counts as none of them.
pub struct Period<'a> {
    steps: &'a Steps<'a>,
    from: u64,
    to: u64,
}

impl<'a> Period<'a> {
    pub fn new(steps: &'a Steps<'a>, from: u64, to: u64) -> Period<'a> {
        Period { steps, from, to }
    }

    /// The steps done in it, oldest first.
    pub fn done(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| s.done().is_some_and(|d| self.holds(d.created_at)))
            .collect()
    }

    /// How many of its done steps went as planned: done with no `changed`.
    pub fn planned(&self) -> usize {
        let done = self.done().into_iter();
        done.filter(|s| !Flags::of(*s).changed).count()
    }

    /// How many of its done steps a later step revised.
    pub fn reworked(&self) -> usize {
        Tally::of(&self.done()).revised
    }

    /// The steps added in it, oldest first.
    pub fn added(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| self.holds(s.note().created_at))
            .collect()
    }

    /// How many of the steps added in it use a decision.
    pub fn ruled(&self) -> usize {
        let decision = |u: &Used| matches!(u, Used::Decision(_));
        let added = self.added().into_iter();
        added.filter(|s| s.uses().iter().any(decision)).count()
    }

    /// The steps done or dropped in it, and those expired with a trail whose newest record is in it,
    /// oldest first.
    pub fn closed(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| s.closed_at().is_some_and(|at| self.holds(at)))
            .collect()
    }

    /// Every step of the store that is not a trail, oldest first.
    fn parts(&self) -> impl Iterator<Item = Step<'a>> {
        self.steps.all().into_iter().filter(|s| !s.is_trail())
    }

    /// Whether the second `at` is in it.
    fn holds(&self, at: u64) -> bool {
        (self.from..self.to).contains(&at)
    }
}

#[cfg(test)]
mod tests;
