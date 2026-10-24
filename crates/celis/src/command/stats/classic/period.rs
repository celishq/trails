//! The work of a store in one span of time.

use crate::progress::Flags;
use celis_trails::{Step, Steps, Used};

/// The work of a store from `from` up to `to`: the steps done, added and closed in that time. A trail
/// counts as none of them.
pub struct Period<'a> {
    steps: &'a Steps<'a>,
    from: u64,
    to: u64,
}

impl<'a> Period<'a> {
    pub fn new(steps: &'a Steps<'a>, from: u64, to: u64) -> Period<'a> {
        Period { steps, from, to }
    }

    /// The second it starts.
    pub fn from(&self) -> u64 {
        self.from
    }

    /// The steps done in it, oldest first.
    pub fn done(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| s.done_record().is_some_and(|d| self.holds(d.created_at)))
            .collect()
    }

    /// How many of its done steps went as planned: done with no `changed`.
    pub fn planned(&self) -> usize {
        let done = self.done().into_iter();
        done.filter(|s| !Flags::of(*s).changed).count()
    }

    /// How many of its done steps a later step revised.
    pub fn reworked(&self) -> usize {
        let done = self.done().into_iter();
        done.filter(|s| Flags::of(*s).revised).count()
    }

    /// The steps done in it with no step under them, oldest first.
    pub fn leaves(&self) -> Vec<Step<'a>> {
        let done = self.done().into_iter();
        done.filter(|s| s.children().is_empty()).collect()
    }

    /// The steps added in it, oldest first.
    pub fn added(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| self.holds(s.note().created_at))
            .collect()
    }

    /// How many of its done steps use a rule.
    pub fn ruled(&self) -> usize {
        let rule = |u: &Used| matches!(u, Used::Rule(_));
        let done = self.done().into_iter();
        done.filter(|s| s.uses().iter().any(rule)).count()
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
