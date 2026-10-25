//! The patterns of a store.

use crate::{Mark, Pattern, PatternEnd, PatternEntry, Place, Refusal, Scope, Step, Used};
use celis_nostr::Note;
use std::collections::{HashMap, HashSet};

/// The patterns of a store, read once, newest first, then by id. `trails` holds the live patterns of each
/// trail.
pub struct Patterns<'a> {
    every: Vec<PatternEntry<'a>>,
    trails: HashMap<String, Vec<usize>>,
}

impl<'a> Patterns<'a> {
    pub fn new(records: &'a [Note]) -> Patterns<'a> {
        let read: Vec<(&'a Note, Pattern)> = records
            .iter()
            .filter_map(|n| Some((n, Pattern::read(n)?)))
            .collect();
        let by: HashMap<&str, &'a Note> = read
            .iter()
            .filter_map(|(n, d)| Some((&**d.replaces.as_ref()?, *n)))
            .collect();
        let drops = Mark::drops(records);
        let mut every: Vec<PatternEntry<'a>> = read
            .iter()
            .map(|(note, pattern)| PatternEntry {
                note,
                pattern: pattern.clone(),
                end: by
                    .get(note.id.as_str())
                    .copied()
                    .map(PatternEnd::Replaced)
                    .or(drops.get(&note.id).map(|d| PatternEnd::Dropped(d))),
            })
            .collect();
        every.sort_by(|a, b| (b.note.created_at, &a.note.id).cmp(&(a.note.created_at, &b.note.id)));
        let mut trails: HashMap<String, Vec<usize>> = HashMap::new();
        for (n, entry) in every.iter().enumerate().filter(|(_, e)| e.is_live()) {
            if let Scope::Trail(trail) = &entry.pattern.scope {
                trails.entry(trail.clone()).or_default().push(n);
            }
        }
        Patterns { every, trails }
    }

    /// Every pattern of every project, live or replaced.
    pub fn every(&self) -> &[PatternEntry<'a>] {
        &self.every
    }

    /// The live patterns in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<(&'a Note, Pattern)> {
        let live = self.every.iter().filter(|e| e.is_live());
        live.filter(|e| e.pattern.scope == Scope::Me || e.pattern.anchor.project == project)
            .map(PatternEntry::said)
            .collect()
    }

    /// The live pattern of any project whose id is `id`.
    pub fn get(&self, id: &str) -> Option<(&'a Note, Pattern)> {
        let live = self.every.iter().find(|e| e.is_live() && e.note.id == id);
        live.map(PatternEntry::said)
    }

    /// The live pattern in scope in `project` whose id is `id`, or the only one whose id starts with `id`
    /// when `id` is 4 or more characters.
    pub fn find(&self, project: &str, id: &str) -> Result<&'a Note, Refusal> {
        let live: Vec<&'a Note> = self.live(project).into_iter().map(|(n, _)| n).collect();
        crate::records::id::find(live, id).ok_or_else(|| Refusal::NoPattern(id.to_string()))
    }

    /// The live patterns of `project` at `place`: those whose place, without its `:<line>`, is `place`,
    /// lies under it, or is a folder that holds it.
    pub fn at(&self, project: &str, place: &str) -> Vec<(&'a Note, Pattern)> {
        self.live(project)
            .into_iter()
            .filter(|(_, d)| match &d.scope {
                Scope::At(at) => Place::new(at).meets(Place::new(place)),
                _ => false,
            })
            .collect()
    }

    /// The live patterns of `trail`, oldest first.
    pub fn of(&self, trail: Step) -> Vec<(&'a Note, Pattern)> {
        let project = &trail.record().anchor.project;
        let patterns = self.trails.get(trail.id()).into_iter().flatten();
        let mut patterns: Vec<(&'a Note, Pattern)> = patterns
            .map(|&n| &self.every[n])
            .filter(|e| &e.pattern.anchor.project == project)
            .map(PatternEntry::said)
            .collect();
        patterns.sort_by(|(a, _), (b, _)| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        patterns
    }

    /// The live patterns the plan of `step` links, each once: its trail's patterns, each pattern a step on its
    /// `under` chain uses, and the patterns of each trail it follows (`Step::followed`).
    pub fn kept(&self, step: Step) -> Vec<(&'a Note, Pattern)> {
        let mut found = self.of(step.trail());
        let chain = std::iter::successors(Some(step), |s| s.under());
        for used in chain.flat_map(Step::uses) {
            if let Used::Pattern(id) = used {
                found.extend(self.get(id));
            }
        }
        for trail in step.followed() {
            found.extend(self.of(trail));
        }
        let mut seen = HashSet::new();
        found.retain(|(note, _)| seen.insert(note.id.clone()));
        found
    }
}
