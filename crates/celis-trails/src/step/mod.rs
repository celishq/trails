//! CIP-03: a step, what it says, the marks agents leave on it, and how it stands.

pub mod check;
pub mod mark;
pub mod state;
pub mod step_record;
pub mod steps;
pub mod used;

use crate::step::steps::{Close, Entry, LIFE};
use crate::{Mark, NamedTrail, State, StepId, StepRecord, Steps, Trail, Used};
use celis_nostr::Note;
use std::collections::HashSet;

/// One step of a store as it stands: its record with its edits, where it sits in its trail, and how its
/// marks left it. A step under no other is a trail.
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

    /// It, with the name it holds first (CIP-07); `None` when it holds none.
    pub fn named(self) -> Option<NamedTrail<'a>> {
        self.steps.named_of(self)
    }

    pub fn is_trail(self) -> bool {
        self.entry.step.under.is_none()
    }

    /// The step it is a part of; `None` for a trail.
    pub fn under(self) -> Option<Step<'a>> {
        self.steps.get(self.entry.step.under.as_deref()?)
    }

    /// What each of its `uses` names, once each, in the order it names them; an id that names no step
    /// and no rule is left out.
    pub fn uses(self) -> Vec<Used<'a>> {
        let mut seen = HashSet::new();
        let ids = self
            .entry
            .step
            .uses
            .iter()
            .filter(|id| seen.insert(id.as_str()));
        ids.filter_map(|id| match self.steps.get(id) {
            Some(step) => Some(Used::Follow(step)),
            None => self.steps.holds_rule(id).then_some(Used::Rule(id)),
        })
        .collect()
    }

    /// The trails that the steps on its `under` chain follow, then, past each named trail (CIP-07), the
    /// trails that its steps follow, at any depth; each once.
    pub fn followed(self) -> Vec<Step<'a>> {
        let follows = |s: Step<'a>| {
            let used = s.uses().into_iter();
            used.filter_map(|u| match u {
                Used::Follow(step) => Some(step.trail()),
                Used::Rule(_) => None,
            })
        };
        let chain = std::iter::successors(Some(self), |s| s.under());
        let mut next: Vec<Step<'a>> = chain.flat_map(follows).collect();
        next.reverse();
        let mut found: Vec<Step<'a>> = Vec::new();
        while let Some(trail) = next.pop() {
            if found.iter().any(|f| f.id() == trail.id()) {
                continue;
            }
            if trail.named().is_some() {
                next.extend(trail.tree().into_iter().flat_map(follows));
            }
            found.push(trail);
        }
        found
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

    /// The step on its `under` chain right under its trail; `None` for a trail.
    pub fn ask(self) -> Option<Step<'a>> {
        let mut ask = self;
        while let Some(up) = ask.under().filter(|u| !u.is_trail()) {
            ask = up;
        }
        (!ask.is_trail()).then_some(ask)
    }

    /// Whether it sits right under its trail.
    pub fn is_ask(self) -> bool {
        self.under().is_some_and(|u| u.is_trail())
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
    /// expired; none while it is open.
    pub fn closed_at(self) -> Option<u64> {
        match (self.closed(), self.state()) {
            (Some(close), _) => Some(close.created_at),
            (None, State::Expired) => Some(self.trail().newest()),
            _ => None,
        }
    }

    /// The done record that closed it.
    pub fn done(self) -> Option<&'a Note> {
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

    /// The drop that names it after its done, as a trail (CIP-03, "Dropping").
    pub fn dropped_after_done(self) -> Option<&'a Note> {
        self.entry.after_done
    }

    /// The steps that replace it after it was done, oldest first.
    pub fn revised(self) -> Vec<Step<'a>> {
        let Some(done) = self.done() else {
            return Vec::new();
        };
        let later = self.entry.replaced.iter();
        let later = later.filter(|n| n.created_at >= done.created_at);
        later.filter_map(|n| self.steps.get(&n.id)).collect()
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
        work.done()?.tag("changed")
    }

    /// Where it stands at the time of its store.
    pub fn state(self) -> State {
        if self.is_trail() {
            return self.trail_state();
        }
        match self.entry.close {
            Some((_, Close::Done)) => State::Done,
            Some((_, Close::Dropped)) => State::Dropped,
            None if self.trail().ended() => State::Expired,
            None if self.start().is_some() => State::Working,
            None => State::Open,
        }
    }

    /// The time of the newest record of its tree: its steps, and the marks that name them.
    pub fn newest(self) -> u64 {
        self.entry.newest
    }

    /// Whether, as a trail, it is open and ended: steps sit under it and none is open, or 30 minutes passed
    /// after the newest record of its tree.
    pub fn ended(self) -> bool {
        if self.closed().is_some() {
            return false;
        }
        let idle = !self.entry.children.is_empty() && !self.entry.loose;
        idle || self.steps.now().saturating_sub(self.newest()) >= LIFE
    }

    /// Where it stands as a trail: working while open and a step of it works, else open; expired when
    /// it ended with a step open; dropped; else done.
    fn trail_state(self) -> State {
        match self.entry.close {
            Some((_, Close::Dropped)) => State::Dropped,
            None if !self.ended() => match self.entry.busy {
                true => State::Working,
                false => State::Open,
            },
            None if self.entry.loose => State::Expired,
            _ => State::Done,
        }
    }
}

impl std::fmt::Debug for Step<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Step").field(&self.id()).finish()
    }
}
