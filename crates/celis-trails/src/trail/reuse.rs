//! What later work reused of the steps and decisions of a store.

use crate::{Decision, Decisions, Steps};
use celis_nostr::Note;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

/// What later work reused of a store: which steps follow which, what each follow saved, and how often a
/// step used each decision.
pub struct Reuse<'a> {
    steps: &'a Steps<'a>,
    sources: HashMap<&'a str, Vec<&'a Note>>,
    followers: HashMap<&'a str, Vec<&'a Note>>,
    uses: HashMap<String, usize>,
}

/// One follow: `step` follows `source`, `depth` follows away from the step a walk starts at.
#[derive(Debug, Clone, Copy)]
pub struct Link<'a> {
    pub step: &'a Note,
    pub source: &'a Note,
    pub depth: usize,
}

impl<'a> Reuse<'a> {
    pub fn new(steps: &'a Steps<'a>) -> Reuse<'a> {
        let mut reuse = Reuse {
            steps,
            sources: HashMap::new(),
            followers: HashMap::new(),
            uses: HashMap::new(),
        };
        for note in steps.all() {
            let Some(step) = steps.step(note) else {
                continue;
            };
            for id in &step.uses {
                match steps.find(id).ok().filter(|n| n.id == *id) {
                    Some(source) => {
                        reuse.sources.entry(&note.id).or_default().push(source);
                        reuse.followers.entry(&source.id).or_default().push(note);
                    }
                    None => *reuse.uses.entry(id.clone()).or_default() += 1,
                }
            }
        }
        reuse
    }

    /// The steps that `note` follows.
    pub fn sources(&self, note: &Note) -> Vec<&'a Note> {
        self.sources
            .get(note.id.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Each step that `note` follows at any depth, depth first and once.
    pub fn up(&self, note: &'a Note) -> Vec<Link<'a>> {
        let mut found = Vec::new();
        self.walk(note, 1, true, &mut HashSet::new(), &mut found);
        found
    }

    /// Each step that follows `note` at any depth, depth first and once.
    pub fn down(&self, note: &'a Note) -> Vec<Link<'a>> {
        let mut found = Vec::new();
        self.walk(note, 1, false, &mut HashSet::new(), &mut found);
        found
    }

    /// The seconds the step of `link` saved against its source; `None` unless both are done.
    pub fn saving(&self, link: &Link<'a>) -> Option<i64> {
        let took = |n: &Note| self.steps.grade(n).map(|g| g.took as i64);
        Some(took(link.source)? - took(link.step)?)
    }

    /// The seconds the steps that follow `note` at any depth saved, each against its source.
    pub fn saved(&self, note: &'a Note) -> Option<i64> {
        sum(self.down(note).iter().map(|l| self.saving(l)))
    }

    /// Each follow of the store, once.
    pub fn links(&self) -> Vec<Link<'a>> {
        let all = self.steps.all().into_iter();
        all.flat_map(|step| {
            let sources = self.sources(step).into_iter();
            sources.map(move |source| Link {
                step,
                source,
                depth: 1,
            })
        })
        .collect()
    }

    /// Each step that a step follows, most followed at any depth first, then newest first, then by id.
    pub fn followed(&self) -> Vec<&'a Note> {
        let all = self.steps.all().into_iter();
        let mut followed: Vec<(usize, &'a Note)> = all
            .filter(|n| self.followers.contains_key(n.id.as_str()))
            .map(|n| (self.down(n).len(), n))
            .collect();
        followed.sort_by_key(|(count, n)| (Reverse(*count), Reverse(n.created_at), n.id.clone()));
        followed.into_iter().map(|(_, n)| n).collect()
    }

    /// Each live decision of `decisions` that a step used, with how often, most used first, then newest
    /// first, then by id.
    pub fn used(&self, decisions: &Decisions<'a>) -> Vec<(&'a Note, Decision, usize)> {
        let mut used: Vec<(&'a Note, Decision, usize)> = self
            .uses
            .iter()
            .filter_map(|(id, count)| {
                let (note, decision) = decisions.get(id)?;
                Some((note, decision, *count))
            })
            .collect();
        used.sort_by_key(|(n, _, count)| (Reverse(*count), Reverse(n.created_at), n.id.clone()));
        used
    }

    /// Adds each link from `from` at `depth`, toward its sources when `up` and else toward its followers,
    /// then the links from each step it reaches, each step once.
    fn walk(
        &self,
        from: &'a Note,
        depth: usize,
        up: bool,
        seen: &mut HashSet<&'a str>,
        found: &mut Vec<Link<'a>>,
    ) {
        let next = match up {
            true => self.sources(from),
            false => self
                .followers
                .get(from.id.as_str())
                .cloned()
                .unwrap_or_default(),
        };
        for to in next {
            if !seen.insert(&to.id) {
                continue;
            }
            let (step, source) = if up { (from, to) } else { (to, from) };
            found.push(Link {
                step,
                source,
                depth,
            });
            self.walk(to, depth + 1, up, seen, found);
        }
    }
}

/// The sum of each saving of `savings`; `None` when none is given.
pub fn sum(savings: impl Iterator<Item = Option<i64>>) -> Option<i64> {
    savings.flatten().reduce(|a, b| a + b)
}
