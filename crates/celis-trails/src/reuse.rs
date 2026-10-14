//! What later work reused of the steps and decisions of a store.

use crate::{Decision, Decisions, Step, Steps};
use celis_nostr::Note;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

/// What later work reused of a store: which steps follow which, what each follow saved, and how often a
/// step used each decision.
pub struct Reuse<'a> {
    sources: HashMap<&'a str, Vec<Step<'a>>>,
    followers: HashMap<&'a str, Vec<Step<'a>>>,
    followed: Vec<Step<'a>>,
    uses: HashMap<&'a str, usize>,
}

/// One follow: `step` follows `source`, `depth` follows away from the step a walk starts at.
#[derive(Clone, Copy)]
pub struct Link<'a> {
    pub step: Step<'a>,
    pub source: Step<'a>,
    pub depth: usize,
}

impl<'a> Reuse<'a> {
    pub fn new(steps: &'a Steps<'a>) -> Reuse<'a> {
        let mut reuse = Reuse {
            sources: HashMap::new(),
            followers: HashMap::new(),
            followed: Vec::new(),
            uses: HashMap::new(),
        };
        for step in steps.all() {
            for id in &step.record().uses {
                match steps.get(id) {
                    Some(source) => {
                        reuse.sources.entry(step.id()).or_default().push(source);
                        reuse.followers.entry(source.id()).or_default().push(step);
                    }
                    None => *reuse.uses.entry(id.as_str()).or_default() += 1,
                }
            }
        }
        let all = steps.all().into_iter();
        let mut followed: Vec<(usize, Step<'a>)> = all
            .filter(|s| reuse.followers.contains_key(s.id()))
            .map(|s| (reuse.down(s).len(), s))
            .collect();
        followed.sort_by_key(|(count, s)| (Reverse(*count), Reverse(s.note().created_at), s.id()));
        reuse.followed = followed.into_iter().map(|(_, s)| s).collect();
        reuse
    }

    /// The steps that `step` follows.
    pub fn sources(&self, step: Step<'a>) -> Vec<Step<'a>> {
        self.sources.get(step.id()).cloned().unwrap_or_default()
    }

    /// Each step that `step` follows at any depth, depth first and once.
    pub fn up(&self, step: Step<'a>) -> Vec<Link<'a>> {
        let mut found = Vec::new();
        self.walk(step, 1, true, &mut HashSet::new(), &mut found);
        found
    }

    /// Each step that follows `step` at any depth, depth first and once.
    pub fn down(&self, step: Step<'a>) -> Vec<Link<'a>> {
        let mut found = Vec::new();
        self.walk(step, 1, false, &mut HashSet::new(), &mut found);
        found
    }

    /// The seconds the step of `link` saved against its source; `None` unless both are done.
    pub fn saving(&self, link: &Link<'a>) -> Option<i64> {
        let took = |s: Step| s.grade().map(|g| g.took as i64);
        Some(took(link.source)? - took(link.step)?)
    }

    /// The seconds the steps that follow `step` at any depth saved, each against its source.
    pub fn saved(&self, step: Step<'a>) -> Option<i64> {
        sum(self.down(step).iter().map(|l| self.saving(l)))
    }

    /// The count of follows in the store, and the seconds they saved, each against its source.
    pub fn total(&self) -> (usize, Option<i64>) {
        let links: Vec<Link<'a>> = self
            .followers
            .values()
            .flatten()
            .flat_map(|&step| {
                let sources = self.sources(step).into_iter();
                sources.map(move |source| Link {
                    step,
                    source,
                    depth: 1,
                })
            })
            .collect();
        let mut links = links;
        links.sort_by_key(|l| (l.step.id(), l.source.id()));
        links.dedup_by_key(|l| (l.step.id(), l.source.id()));
        (links.len(), sum(links.iter().map(|l| self.saving(l))))
    }

    /// Each step that a step follows, most followed at any depth first, then newest first, then by id.
    pub fn followed(&self) -> &[Step<'a>] {
        &self.followed
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
        from: Step<'a>,
        depth: usize,
        up: bool,
        seen: &mut HashSet<&'a str>,
        found: &mut Vec<Link<'a>>,
    ) {
        let next = match up {
            true => self.sources.get(from.id()),
            false => self.followers.get(from.id()),
        };
        for &to in next.into_iter().flatten() {
            if !seen.insert(to.id()) {
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
fn sum(savings: impl Iterator<Item = Option<i64>>) -> Option<i64> {
    savings.flatten().reduce(|a, b| a + b)
}
