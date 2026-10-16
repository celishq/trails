//! A step: what it says, the marks agents leave on it, and how it stands.

pub mod check;
pub mod flags;
pub mod grade;
pub mod mark;
pub mod state;
pub mod step_record;
pub mod steps;
pub mod tally;

use crate::step::steps::{Close, Entry, LIFE};
use crate::{Flags, Grade, State, StepId, StepRecord, Steps, Trail};
use celis_nostr::Note;

/// One step of a store as it stands: its record with its edits, where it sits in its trail, how its
/// marks left it, and how its work went. A step under no other is a trail.
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

    pub fn flags(self) -> Flags {
        let revised = match self.is_trail() {
            true => !self.reworked().is_empty(),
            false => !self.revised().is_empty(),
        };
        Flags {
            changed: self.changed().is_some(),
            revised,
        }
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

    /// How its work went; `None` while it is not done.
    pub fn grade(self) -> Option<Grade> {
        let done = self.done()?;
        let from = self.start().unwrap_or(self.note()).created_at;
        Some(Grade {
            took: done.created_at.saturating_sub(from),
        })
    }

    /// The time of the newest record of its tree: its steps, and the marks that name them.
    pub fn newest(self) -> u64 {
        let tree = self.tree().into_iter();
        tree.map(|s| s.entry.newest)
            .max()
            .unwrap_or(self.note().created_at)
    }

    /// Whether, as a trail, it is open and ended: steps sit under it and none is open, or 30 minutes passed
    /// after the newest record of its tree.
    pub fn ended(self) -> bool {
        if self.closed().is_some() {
            return false;
        }
        let tree = self.tree();
        let idle = tree.len() > 1 && tree[1..].iter().all(|s| s.closed().is_some());
        idle || self.steps.now.saturating_sub(self.newest()) >= LIFE
    }

    /// Where it stands as a trail: working while open and a step of it works, else open; expired when
    /// it ended with a step open; dropped; else done.
    fn trail_state(self) -> State {
        let tree = self.tree();
        let parts = &tree[1..];
        match self.entry.close {
            Some((_, Close::Dropped)) => State::Dropped,
            None if !self.ended() => {
                let works = |s: &Step| s.start().is_some() && s.closed().is_none();
                match works(&self) || parts.iter().any(works) {
                    true => State::Working,
                    false => State::Open,
                }
            }
            None if parts.iter().any(|s| s.closed().is_none()) => State::Expired,
            _ => State::Done,
        }
    }
}

impl std::fmt::Debug for Step<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Step").field(&self.id()).finish()
    }
}
