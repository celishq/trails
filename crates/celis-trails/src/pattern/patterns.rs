//! The patterns of a store.

use crate::{Fit, Mark, Pattern, PatternEnd, PatternRecord, Refusal, Scope, Step, Used};
use celis_nostr::Note;
use std::collections::{HashMap, HashSet};

/// The patterns of a store, read once, newest first, then by id. `trails` holds the live patterns of each
/// trail.
pub struct Patterns<'a> {
    every: Vec<Pattern<'a>>,
    trails: HashMap<String, Vec<usize>>,
}

impl<'a> Patterns<'a> {
    pub fn new(records: &'a [Note]) -> Patterns<'a> {
        let read: Vec<(&'a Note, PatternRecord)> = records
            .iter()
            .filter_map(|n| Some((n, PatternRecord::read(n)?)))
            .collect();
        let by: HashMap<String, &'a Note> = read
            .iter()
            .filter_map(|(n, d)| Some((d.replaces.as_ref()?.to_string(), *n)))
            .collect();
        let drops = Mark::drops(records);
        let mut every: Vec<Pattern<'a>> = read
            .into_iter()
            .map(|(note, record)| Pattern {
                note,
                record,
                end: by
                    .get(note.id.as_str())
                    .copied()
                    .map(PatternEnd::Replaced)
                    .or(drops.get(&note.id).map(|d| PatternEnd::Dropped(d))),
            })
            .collect();
        every.sort_by(|a, b| (b.note.created_at, &a.note.id).cmp(&(a.note.created_at, &b.note.id)));
        let mut trails: HashMap<String, Vec<usize>> = HashMap::new();
        for (n, pattern) in every.iter().enumerate().filter(|(_, p)| p.is_live()) {
            if let Scope::Trail(trail) = &pattern.record.scope {
                trails.entry(trail.clone()).or_default().push(n);
            }
        }
        Patterns { every, trails }
    }

    /// Every pattern of every project, live or ended.
    pub fn every(&self) -> &[Pattern<'a>] {
        &self.every
    }

    /// The live patterns in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<Pattern<'a>> {
        let live = self.every.iter().filter(|p| p.is_live());
        live.filter(|p| self.holds(p, project)).cloned().collect()
    }

    /// The live pattern in scope in `project`, or of any project with `None`, whose id is `id`, or the only
    /// one whose id starts with `id` when `id` is 4 or more characters.
    pub fn find(&self, project: Option<&str>, id: &str) -> Result<Pattern<'a>, Refusal> {
        let live: Vec<&Pattern<'a>> = self
            .every
            .iter()
            .filter(|p| p.is_live())
            .filter(|p| project.is_none_or(|project| self.holds(p, project)))
            .collect();
        let note = crate::records::id::find(live.iter().map(|p| p.note), id);
        let note = note.ok_or_else(|| Refusal::NoPattern(id.to_string()))?;
        let found = live.into_iter().find(|p| p.note.id == note.id);
        Ok(found.cloned().expect("the note is of a live pattern"))
    }

    /// The live patterns in scope in `project` that match one of `paths` or that `also` takes, in the order
    /// `search` prints them: those with a `replaces` tag first, then those of a named file, of a glob, and
    /// the rest; newest first within each (CIP-02, "Search").
    pub fn fitting(
        &self,
        project: &str,
        paths: &[String],
        also: impl Fn(&Pattern<'a>) -> bool,
    ) -> Vec<Pattern<'a>> {
        let mut found: Vec<(Option<Fit>, Pattern<'a>)> = self
            .live(project)
            .into_iter()
            .map(|p| (p.record.fit(paths), p))
            .filter(|(fit, p)| fit.is_some() || also(p))
            .collect();
        found.sort_by_key(|(fit, p)| (p.record.replaces.is_none(), fit.is_none(), *fit));
        found.into_iter().map(|(_, p)| p).collect()
    }

    /// The live patterns in scope in `project` whose `within` names `id`, newest first.
    pub fn smaller(&self, project: &str, id: &str) -> Vec<Pattern<'a>> {
        let live = self.live(project).into_iter();
        live.filter(|p| p.record.within.iter().any(|w| &**w == id))
            .collect()
    }

    /// The live patterns of `trail`, oldest first.
    pub fn of(&self, trail: Step) -> Vec<Pattern<'a>> {
        let project = &trail.record().anchor.project;
        let patterns = self.trails.get(trail.id()).into_iter().flatten();
        let mut patterns: Vec<Pattern<'a>> = patterns
            .map(|&n| &self.every[n])
            .filter(|p| &p.record.anchor.project == project)
            .cloned()
            .collect();
        patterns
            .sort_by(|a, b| (a.note.created_at, &a.note.id).cmp(&(b.note.created_at, &b.note.id)));
        patterns
    }

    /// The live patterns the plan of `step` links, each once: its trail's patterns, each pattern a step on its
    /// `under` chain uses, and the patterns of each trail it follows (`Step::followed`).
    pub fn kept(&self, step: Step) -> Vec<Pattern<'a>> {
        let mut found = self.of(step.trail());
        let chain = std::iter::successors(Some(step), |s| s.under());
        for used in chain.flat_map(Step::uses) {
            if let Used::Pattern(id) = used {
                found.extend(self.find(None, id));
            }
        }
        for trail in step.followed() {
            found.extend(self.of(trail));
        }
        let mut seen = HashSet::new();
        found.retain(|p| seen.insert(p.note.id.clone()));
        found
    }

    /// Whether `pattern` holds in `project`.
    fn holds(&self, pattern: &Pattern, project: &str) -> bool {
        pattern.record.scope == Scope::Me || pattern.record.anchor.project == project
    }
}
