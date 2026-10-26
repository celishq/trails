//! CIP-03: a step, what it says, the marks agents leave on it, and how it stands.

pub mod by;
pub mod check;
pub mod done;
pub mod mark;
pub mod notes;
pub mod state;
pub mod step_record;
pub mod steps;

use crate::step::steps::{Close, Entry, LIFE};
use crate::{Done, Mark, State, StepRecord, Steps, Trail};
use celis_nostr::Note;
use celis_store::StepId;
use std::collections::HashSet;

/// One step of a store as it stands: its record with its edits, where it sits in its trail, and how its
/// marks left it. A step under no other is an ask, and its trail.
#[derive(Clone, Copy)]
pub struct Step<'a> {
    steps: &'a Steps<'a>,
    entry: &'a Entry<'a>,
}

impl<'a> Step<'a> {
    pub(crate) fn new(steps: &'a Steps<'a>, entry: &'a Entry<'a>) -> Step<'a> {
        Step { steps, entry }
    }

    pub fn note(self) -> &'a Note {
        self.entry.note
    }

    pub fn id(self) -> &'a str {
        &self.entry.note.id
    }

    /// Its id, as a link to it holds it.
    pub fn step_id(self) -> StepId {
        StepId::new(self.id())
    }

    /// What the step says, with each of its edits.
    pub fn record(self) -> &'a StepRecord {
        &self.entry.step
    }

    pub fn title(self) -> &'a str {
        self.entry.step.title()
    }

    pub fn is_trail(self) -> bool {
        self.entry.step.under.is_none()
    }

    /// The step it is a part of; `None` for a trail.
    pub fn under(self) -> Option<Step<'a>> {
        self.steps.get(self.entry.step.under.as_deref()?)
    }

    /// The id of each pattern its `uses` names, once each, in the order it names them; an id that names a
    /// step or nothing is left out (CIP-06, "Uses").
    pub fn uses(self) -> Vec<&'a str> {
        let mut seen = HashSet::new();
        let ids = self.entry.step.uses.iter().map(String::as_str);
        ids.filter(|id| seen.insert(*id) && self.steps.holds_pattern(id))
            .collect()
    }

    /// The steps directly under it, oldest first.
    pub fn children(self) -> Vec<Step<'a>> {
        let children = self.entry.children.iter();
        children.filter_map(|n| self.steps.get(&n.id)).collect()
    }

    /// The step and every step under it, at any depth.
    pub fn tree(self) -> Vec<Step<'a>> {
        let mut tree = vec![self];
        let mut i = 0;
        while i < tree.len() {
            tree.extend(tree[i].children());
            i += 1;
        }
        tree
    }

    /// The trail at the top of its `under` chain.
    pub fn trail(self) -> Step<'a> {
        let mut top = self;
        while let Some(up) = top.under() {
            top = up;
        }
        top
    }

    /// Whether a judge confirms it: an ask, or another step with steps under it.
    pub fn is_judged(self) -> bool {
        self.is_trail() || !self.entry.children.is_empty()
    }

    /// The first start record that names it.
    pub fn start(self) -> Option<&'a Note> {
        self.entry.start
    }

    /// The first done record with a proof, or drop record with a reason, that names it.
    pub fn closed(self) -> Option<&'a Note> {
        self.entry.close.map(|(note, _)| note)
    }

    /// The record that closed it, with its mark: done with its proof and what changed, or dropped with
    /// its reason.
    pub fn close(self) -> Option<(&'a Note, Mark)> {
        let note = self.closed()?;
        Some((note, Mark::read(note)?.1))
    }

    /// The second it ended: when a done or a drop closed it, or the newest record of its trail when it
    /// is stale; none while it is open.
    pub fn closed_at(self) -> Option<u64> {
        match self.closed() {
            Some(close) => Some(close.created_at),
            None if self.stale() => Some(self.trail().newest()),
            None => None,
        }
    }

    /// What it was closed done with; none while it is not done.
    pub fn done(self) -> Option<Done> {
        match Mark::read(self.done_record()?)? {
            (_, Mark::Done(done)) => Some(done),
            _ => None,
        }
    }

    /// The done record that closed it.
    pub fn done_record(self) -> Option<&'a Note> {
        match self.entry.close {
            Some((note, Close::Done)) => Some(note),
            _ => None,
        }
    }

    /// The drop record that closed it.
    pub fn dropped(self) -> Option<&'a Note> {
        match self.entry.close {
            Some((note, Close::Dropped)) => Some(note),
            _ => None,
        }
    }

    /// The first drop that counts on it: the one that closed it, or, as a trail, one after its done
    /// (CIP-03, "Dropping").
    pub fn drop_mark(self) -> Option<&'a Note> {
        self.entry.drop
    }

    /// The steps that replace it after it was done, oldest first.
    pub fn revised(self) -> Vec<Step<'a>> {
        let Some(done) = self.done_record() else {
            return Vec::new();
        };
        let later = self.entry.replaced.iter();
        let later = later.filter(|n| n.created_at >= done.created_at);
        later.filter_map(|n| self.steps.get(&n.id)).collect()
    }

    /// Whether it fixes done work: the step it replaces is done.
    pub fn fixes(self) -> bool {
        let replaced = self.entry.step.replaces.as_deref();
        replaced
            .and_then(|id| self.steps.get(id))
            .is_some_and(|s| s.done_record().is_some())
    }

    /// The steps of its tree that a later step revised.
    pub fn reworked(self) -> Vec<Step<'a>> {
        let tree = self.tree().into_iter();
        tree.filter(|s| !s.revised().is_empty()).collect()
    }

    /// What its done says changed, from its first ask's done when it is an open trail.
    pub fn changed(self) -> Option<&'a str> {
        let work = match self.closed() {
            Some(_) => self,
            None => Trail::new(self).work(),
        };
        work.done_record()?.tag("changed")
    }

    /// Where it stands at the time of its store.
    pub fn state(self) -> State {
        match self.entry.close {
            _ if self.entry.drop.is_some() => State::Dropped,
            Some((_, Close::Done)) => State::Done,
            Some((_, Close::Dropped)) => State::Dropped,
            None => State::Open,
        }
    }

    /// Whether it is open and its trail ended.
    pub fn stale(self) -> bool {
        self.state() == State::Open && self.trail().ended()
    }

    /// Whether it is open and not stale.
    pub fn active(self) -> bool {
        self.state() == State::Open && !self.stale()
    }

    /// Whether it is open and not stale, and a start names it, or, as a trail, a step of it.
    pub fn works(self) -> bool {
        let started = match self.is_trail() {
            true => self.entry.busy,
            false => self.start().is_some(),
        };
        started && self.active()
    }

    /// The time of the newest record of its tree: its steps, and the marks that name them.
    pub fn newest(self) -> u64 {
        self.entry.newest
    }

    /// Whether, as a trail, it is open and ended: steps sit under it, none is open and no prompt came
    /// after the last of them closed, or 30 minutes passed after the newest record of its tree.
    pub fn ended(self) -> bool {
        if self.closed().is_some() {
            return false;
        }
        let asked = self.entry.asked > self.entry.shut;
        let idle = !self.entry.children.is_empty() && !self.entry.loose && !asked;
        idle || self.steps.now().saturating_sub(self.newest()) >= LIFE
    }
}

impl std::fmt::Debug for Step<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Step").field(&self.id()).finish()
    }
}
