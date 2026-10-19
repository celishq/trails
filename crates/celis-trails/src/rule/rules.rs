//! The rules of a store.

use crate::{Place, Refusal, Rule, RuleEntry, Scope, Step, Used};
use celis_nostr::Note;
use std::collections::{HashMap, HashSet};

/// The rules of a store, read once, newest first, then by id. `trails` holds the live rules of each
/// trail, and `hidden` each live override by its project and the id it hides.
pub struct Rules<'a> {
    every: Vec<RuleEntry<'a>>,
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
        let overrides: Vec<Option<String>> = read
            .iter()
            .map(|(_, d)| overridden(d, &rules).map(str::to_string))
            .collect();
        let by: HashMap<&str, &'a Note> = read
            .iter()
            .zip(&overrides)
            .filter(|(_, overrides)| overrides.is_none())
            .filter_map(|((n, d), _)| Some((&**d.replaces.as_ref()?, *n)))
            .collect();
        let mut every: Vec<RuleEntry<'a>> = read
            .iter()
            .zip(overrides)
            .map(|((note, rule), overrides)| {
                let ended = overrides.as_deref().and_then(|old| by.get(old).copied());
                RuleEntry {
                    note,
                    rule: rule.clone(),
                    by: by.get(note.id.as_str()).copied().or(ended),
                    overrides,
                }
            })
            .collect();
        every.sort_by(|a, b| (b.note.created_at, &a.note.id).cmp(&(a.note.created_at, &b.note.id)));
        let mut trails: HashMap<String, Vec<usize>> = HashMap::new();
        let mut hidden = HashMap::new();
        for (n, entry) in every.iter().enumerate().filter(|(_, e)| e.is_live()) {
            if let Scope::Trail(trail) = &entry.rule.scope {
                trails.entry(trail.clone()).or_default().push(n);
            }
            if let Some(old) = &entry.overrides {
                hidden.insert((entry.rule.anchor.project.clone(), old.clone()), n);
            }
        }
        Rules {
            every,
            trails,
            hidden,
        }
    }

    /// Every rule of every project, live or replaced.
    pub fn every(&self) -> &[RuleEntry<'a>] {
        &self.every
    }

    /// The live rules in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<(&'a Note, Rule)> {
        let live = self.every.iter().filter(|e| e.is_live());
        live.filter(|e| e.rule.scope == Scope::Me || e.rule.anchor.project == project)
            .filter(|e| self.hiding(project, &e.note.id).is_none())
            .map(RuleEntry::said)
            .collect()
    }

    /// The live override of `project` that hides the rule `id` there.
    fn hiding(&self, project: &str, id: &str) -> Option<(&'a Note, Rule)> {
        let n = *self.hidden.get(&(project.to_string(), id.to_string()))?;
        Some(self.every[n].said())
    }

    /// Each live override of the rule `id`, in any project.
    pub fn overrides(&self, id: &str) -> Vec<(&'a Note, Rule)> {
        let hiding = self.hidden.iter().filter(|((_, old), _)| old == id);
        hiding.map(|(_, &n)| self.every[n].said()).collect()
    }

    /// The live rule of a trail of `trails` whose id is `id`, or the only one whose id starts with `id`
    /// when `id` is 4 or more characters.
    pub fn find_of(&self, trails: &[Step], id: &str) -> Result<&'a Note, Refusal> {
        let live = trails.iter().flat_map(|t| self.of(*t)).map(|(n, _)| n);
        crate::records::id::find(live, id).ok_or_else(|| Refusal::NoRule(id.to_string()))
    }

    /// The live rule of any project whose id is `id`.
    pub fn get(&self, id: &str) -> Option<(&'a Note, Rule)> {
        let live = self.every.iter().find(|e| e.is_live() && e.note.id == id);
        live.map(RuleEntry::said)
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
            .filter(|e| &e.rule.anchor.project == project)
            .map(RuleEntry::said)
            .collect();
        rules.sort_by(|(a, _), (b, _)| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        rules
    }

    /// The live rules `step` must keep, each once: those at its places, its trail's rules, each
    /// rule a step on its `under` chain uses, and the rules of each trail it follows (`Step::followed`).
    pub fn kept(&self, step: Step) -> Vec<(&'a Note, Rule)> {
        let record = step.record();
        let places = record.places.iter();
        let mut found: Vec<_> = places
            .flat_map(|place| self.at(&record.anchor.project, place))
            .collect();
        found.extend(self.of(step.trail()));
        let chain = std::iter::successors(Some(step), |s| s.under());
        for used in chain.flat_map(Step::uses) {
            if let Used::Rule(id) = used {
                found.extend(self.get(id));
            }
        }
        for trail in step.followed() {
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

/// The id of the rule `rule` overrides: a `trail:<id>` rule of another project of `rules` that it
/// replaces.
fn overridden<'r>(rule: &'r Rule, rules: &HashMap<&str, &Rule>) -> Option<&'r str> {
    let id: &str = rule.replaces.as_deref()?;
    let old = rules.get(id)?;
    let trail = matches!(old.scope, Scope::Trail(_));
    (trail && old.anchor.project != rule.anchor.project).then_some(id)
}
