//! The work of a store in one span of time.

use crate::reuse::Link;
use crate::{Reuse, State, Step, Steps, Tally};

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
        done.filter(|s| !s.flags().changed).count()
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
        let added = self.added().into_iter();
        let decision = |id: &String| self.steps.get(id).is_none();
        added
            .filter(|s| s.record().uses.iter().any(decision))
            .count()
    }

    /// The steps done or dropped in it, and those expired with a trail whose newest record is in it,
    /// oldest first.
    pub fn closed(&self) -> Vec<Step<'a>> {
        self.parts()
            .filter(|s| match (s.closed(), s.state()) {
                (Some(close), _) => self.holds(close.created_at),
                (None, State::Expired) => self.holds(s.trail().newest()),
                _ => false,
            })
            .collect()
    }

    /// The count of follows whose step was done in it, and the seconds they saved, each against its
    /// source.
    pub fn follows(&self, reuse: &Reuse<'a>) -> (usize, Option<i64>) {
        let links: Vec<Link<'a>> = self
            .done()
            .into_iter()
            .flat_map(|step| {
                let sources = reuse.sources(step).into_iter();
                sources.map(move |source| Link {
                    step,
                    source,
                    depth: 1,
                })
            })
            .collect();
        let saved = links.iter().filter_map(|l| reuse.saving(l));
        (links.len(), saved.reduce(|a, b| a + b))
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
