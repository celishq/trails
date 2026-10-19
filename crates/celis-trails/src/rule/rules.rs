//! The rules of a store.

use crate::{Place, Refusal, Rule, Scope, Step, Used};
use celis_nostr::Note;
use std::collections::{HashMap, HashSet};

/// The rules of a store, read once, newest first, then by id; each with the rule that replaces
/// it, if one does. A rule no other replaces is live. An override (CIP-07) hides the rule it replaces in
/// its own project only, and is replaced by the rule that replaces that rule. `trails` holds the live
/// rules of each trail, and `hidden` each live override by its project and the id it hides.
pub struct Rules<'a> {
    every: Vec<(&'a Note, Rule, Option<&'a Note>)>,
    trails: HashMap<String, Vec<usize>>,
    hidden: HashMap<(String, String), usize>,
}

impl<'a> Rules<'a> {
    pub fn new(records: &'a [Note]) -> Rules<'a> {
        let read: Vec<(&'a Note, Rule)> = records
            .iter()
            .filter_map(|n| Some((n, Rule::read(n)?)))
            .collect();
        let rules: HashMap<&str, &Rule> = read.iter().map(|(n, d)| (n.id.as_str(), d)).collect();
        let overridden = |d: &Rule| overridden(d, &rules).map(str::to_string);
        let by: HashMap<&str, &'a Note> = read
            .iter()
            .filter(|(_, d)| overridden(d).is_none())
            .filter_map(|(n, d)| Some((&**d.replaces.as_ref()?, *n)))
            .collect();
        let mut every: Vec<_> = read
            .iter()
            .map(|(n, d)| {
                let ended = overridden(d).and_then(|old| by.get(old.as_str()).copied());
                (*n, d.clone(), by.get(n.id.as_str()).copied().or(ended))
            })
            .collect();
        every.sort_by(|(a, ..), (b, ..)| (b.created_at, &a.id).cmp(&(a.created_at, &b.id)));
        let mut trails: HashMap<String, Vec<usize>> = HashMap::new();
        let mut hidden = HashMap::new();
        for (n, (_, rule, by)) in every.iter().enumerate() {
            if let (Scope::Trail(trail), None) = (&rule.scope, by) {
                trails.entry(trail.clone()).or_default().push(n);
            }
            if let (Some(old), None) = (overridden(rule), by) {
                hidden.insert((rule.anchor.project.clone(), old), n);
            }
        }
        Rules {
            every,
            trails,
            hidden,
        }
    }

    /// Every rule of every project, live or replaced.
    pub fn every(&self) -> &[(&'a Note, Rule, Option<&'a Note>)] {
        &self.every
    }

    /// The live rules in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<(&'a Note, Rule)> {
        let live = self.every.iter().filter(|(.., by)| by.is_none());
        live.filter(|(_, d, _)| d.scope == Scope::Me || d.anchor.project == project)
            .filter(|(n, ..)| self.hiding(project, &n.id).is_none())
            .map(|(n, d, _)| (*n, d.clone()))
            .collect()
    }

    /// The live override of `project` that hides the rule `id` there.
    fn hiding(&self, project: &str, id: &str) -> Option<(&'a Note, Rule)> {
        let n = *self.hidden.get(&(project.to_string(), id.to_string()))?;
        let (note, rule, _) = &self.every[n];
        Some((*note, rule.clone()))
    }

    /// The live rule of a trail of `trails` whose id is `id`, or the only one whose id starts with `id`
    /// when `id` is 4 or more characters.
    pub fn find_of(&self, trails: &[Step], id: &str) -> Result<&'a Note, Refusal> {
        let live = trails.iter().flat_map(|t| self.of(*t)).map(|(n, _)| n);
        crate::records::id::find(live, id).ok_or_else(|| Refusal::NoRule(id.to_string()))
    }

    /// The live rule of any project whose id is `id`.
    pub fn get(&self, id: &str) -> Option<(&'a Note, Rule)> {
        let live = self
            .every
            .iter()
            .find(|(n, _, by)| by.is_none() && n.id == id);
        live.map(|(n, d, _)| (*n, d.clone()))
    }

    /// The live rule in scope in `project` whose id is `id`, or the only one whose id starts with `id`
    /// when `id` is 4 or more characters.
    pub fn find(&self, project: &str, id: &str) -> Result<&'a Note, Refusal> {
        let live: Vec<&'a Note> = self.live(project).into_iter().map(|(n, _)| n).collect();
        crate::records::id::find(live, id).ok_or_else(|| Refusal::NoRule(id.to_string()))
    }

    /// The live rules of `project` at `place`: those whose place, without its `:<line>`, is `place`,
    /// lies under it, or is a folder that holds it.
    pub fn at(&self, project: &str, place: &str) -> Vec<(&'a Note, Rule)> {
        self.live(project)
            .into_iter()
            .filter(|(_, d)| match &d.scope {
                Scope::At(at) => Place::new(at).meets(Place::new(place)),
                _ => false,
            })
            .collect()
    }

    /// The live rules of `trail`, oldest first.
    pub fn of(&self, trail: Step) -> Vec<(&'a Note, Rule)> {
        let project = &trail.record().anchor.project;
        let rules = self.trails.get(trail.id()).into_iter().flatten();
        let mut rules: Vec<(&'a Note, Rule)> = rules
            .map(|&n| &self.every[n])
            .filter(|(_, d, _)| &d.anchor.project == project)
            .map(|(n, d, _)| (*n, d.clone()))
            .collect();
        rules.sort_by(|(a, _), (b, _)| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        rules
    }

    /// The live rules `step` must keep, each once: those at its places, its trail's rules, each
    /// rule a step on its `under` chain uses, the rules of the trail of each step one follows, and the
    /// rules of each named trail on the chain of the named trails it follows.
    pub fn kept(&self, step: Step) -> Vec<(&'a Note, Rule)> {
        let record = step.record();
        let places = record.places.iter();
        let mut found: Vec<_> = places
            .flat_map(|place| self.at(&record.anchor.project, place))
            .collect();
        found.extend(self.of(step.trail()));
        let chain = std::iter::successors(Some(step), |s| s.under());
        for used in chain.flat_map(Step::uses) {
            match used {
                Used::Follow(other) => found.extend(self.of(other.trail())),
                Used::Rule(id) => found.extend(self.get(id)),
            }
        }
        for trail in named(step) {
            found.extend(self.of(trail));
        }
        let project = &record.anchor.project;
        let found = found.into_iter();
        let mut found: Vec<_> = found
            .map(|(n, d)| self.hiding(project, &n.id).unwrap_or((n, d)))
            .collect();
        let mut seen = HashSet::new();
        found.retain(|(note, _)| seen.insert(note.id.clone()));
        found
    }
}

/// Each named trail that a step on the `under` chain of `step` follows, and each named trail that a step
/// of a named trail found follows, at any depth, each once.
fn named(step: Step) -> Vec<Step> {
    let chain = std::iter::successors(Some(step), |s| s.under());
    let mut found: Vec<Step> = Vec::new();
    let mut next: Vec<Step> = chain.flat_map(Step::uses).filter_map(followed).collect();
    while let Some(trail) = next.pop() {
        if found.iter().any(|f| f.id() == trail.id()) {
            continue;
        }
        let tree = trail.tree().into_iter();
        next.extend(tree.flat_map(Step::uses).filter_map(followed));
        found.push(trail);
    }
    found
}

/// The trail of the step `used` follows, when the trail is named.
fn followed(used: Used) -> Option<Step> {
    let Used::Follow(step) = used else {
        return None;
    };
    let trail = step.trail();
    trail.name().map(|_| trail)
}

/// The id of the rule `rule` overrides: a `trail:<id>` rule of another project of `rules` that it
/// replaces.
fn overridden<'r>(rule: &'r Rule, rules: &HashMap<&str, &Rule>) -> Option<&'r str> {
    let id: &str = rule.replaces.as_deref()?;
    let old = rules.get(id)?;
    let trail = matches!(old.scope, Scope::Trail(_));
    (trail && old.anchor.project != rule.anchor.project).then_some(id)
}
