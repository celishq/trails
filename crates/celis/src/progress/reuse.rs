//! What later work reused of the steps and decisions of a store.

use celis_nostr::Note;
use celis_trails::{Decision, Decisions, Step, Steps, Used};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

/// What later work reused of a store: which steps follow which, and how often a step used each decision.
pub struct Reuse<'a> {
    sources: HashMap<&'a str, Vec<Step<'a>>>,
    followers: HashMap<&'a str, Vec<Step<'a>>>,
    reach: HashMap<&'a str, usize>,
    followed: Vec<Step<'a>>,
    uses: HashMap<&'a str, usize>,
}

/// One follow: `step` follows `source`, `depth` follows away from the step a walk starts at.
#[derive(Clone, Copy)]
pub struct Follow<'a> {
    pub step: Step<'a>,
    pub source: Step<'a>,
    pub depth: usize,
}

impl<'a> Reuse<'a> {
    pub fn new(steps: &'a Steps<'a>) -> Reuse<'a> {
        let mut reuse = Reuse {
            sources: HashMap::new(),
            followers: HashMap::new(),
            reach: HashMap::new(),
            followed: Vec::new(),
            uses: HashMap::new(),
        };
        for step in steps.all() {
            for used in step.uses() {
                match used {
                    Used::Follow(source) => {
                        reuse.sources.entry(step.id()).or_default().push(source);
                        reuse.followers.entry(source.id()).or_default().push(step);
                    }
                    Used::Decision(id) => *reuse.uses.entry(id).or_default() += 1,
                }
            }
        }
        let mut followed: Vec<Step<'a>> = steps
            .all()
            .into_iter()
            .filter(|s| reuse.followers.contains_key(s.id()))
            .collect();
        for &step in &followed {
            reuse.reach.insert(step.id(), reuse.down(step).len());
        }
        followed.sort_by_key(|&s| {
            (
                Reverse(reuse.followers(s)),
                Reverse(s.note().created_at),
                s.id(),
            )
        });
        reuse.followed = followed;
        reuse
    }

    /// The steps that `step` follows.
    pub fn sources(&self, step: Step<'a>) -> Vec<Step<'a>> {
        self.sources.get(step.id()).cloned().unwrap_or_default()
    }

    /// Each step that `step` follows at any depth, depth first and once.
    pub fn up(&self, step: Step<'a>) -> Vec<Follow<'a>> {
        let mut found = Vec::new();
        self.walk(step, 1, true, &mut HashSet::new(), &mut found);
        found
    }

    /// Each step that follows `step` at any depth, depth first and once.
    pub fn down(&self, step: Step<'a>) -> Vec<Follow<'a>> {
        let mut found = Vec::new();
        self.walk(step, 1, false, &mut HashSet::new(), &mut found);
        found
    }

    /// The count of steps that follow `step` at any depth.
    pub fn followers(&self, step: Step<'a>) -> usize {
        self.reach.get(step.id()).copied().unwrap_or(0)
    }

    /// The count of follows in the store.
    pub fn total(&self) -> usize {
        self.sources.values().map(Vec::len).sum()
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
    /// then the follows from each step it reaches, each step once.
    fn walk(
        &self,
        from: Step<'a>,
        depth: usize,
        up: bool,
        seen: &mut HashSet<&'a str>,
        found: &mut Vec<Follow<'a>>,
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
            found.push(Follow {
                step,
                source,
                depth,
            });
            self.walk(to, depth + 1, up, seen, found);
        }
    }
}
