//! The decisions of a store.

use crate::{Decision, Failure, Scope};
use celis_nostr::Note;
use std::collections::HashMap;

/// The decisions of a store, read once, newest first, then by id; each with the decision that replaces
/// it, if one does. A decision no other replaces is live.
pub struct Decisions<'a> {
    every: Vec<(&'a Note, Decision, Option<&'a Note>)>,
}

impl<'a> Decisions<'a> {
    pub fn new(records: &'a [Note]) -> Decisions<'a> {
        let read: Vec<(&'a Note, Decision)> = records
            .iter()
            .filter_map(|n| Some((n, Decision::read(n)?)))
            .collect();
        let by: HashMap<&str, &'a Note> = read
            .iter()
            .filter_map(|(n, d)| Some((&**d.replaces.as_ref()?, *n)))
            .collect();
        let mut every: Vec<_> = read
            .iter()
            .map(|(n, d)| (*n, d.clone(), by.get(n.id.as_str()).copied()))
            .collect();
        every.sort_by(|(a, ..), (b, ..)| (b.created_at, &a.id).cmp(&(a.created_at, &b.id)));
        Decisions { every }
    }

    /// Every decision of every project, live or replaced.
    pub fn every(&self) -> &[(&'a Note, Decision, Option<&'a Note>)] {
        &self.every
    }

    /// The live decisions in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<(&'a Note, Decision)> {
        let live = self.every.iter().filter(|(.., by)| by.is_none());
        live.filter(|(_, d, _)| d.scope == Scope::Me || d.anchor.project == project)
            .map(|(n, d, _)| (*n, d.clone()))
            .collect()
    }

    /// The live decision of any project whose id is `id`.
    pub fn get(&self, id: &str) -> Option<(&'a Note, Decision)> {
        let live = self
            .every
            .iter()
            .find(|(n, _, by)| by.is_none() && n.id == id);
        live.map(|(n, d, _)| (*n, d.clone()))
    }

    /// The live decision in scope in `project` whose id is `id`, or the only one whose id starts with `id`
    /// when `id` is 4 or more characters.
    pub fn find(&self, project: &str, id: &str) -> Result<&'a Note, Failure> {
        let live: Vec<&'a Note> = self.live(project).into_iter().map(|(n, _)| n).collect();
        crate::id::find(live, id).ok_or_else(|| Failure::no_decision(id))
    }

    /// The live decisions of `project` at `place`: those whose place, without its `:<line>`, is `place`,
    /// lies under it, or is a folder that holds it.
    pub fn at(&self, project: &str, place: &str) -> Vec<(&'a Note, Decision)> {
        let place = path(place).trim_end_matches('/');
        self.live(project)
            .into_iter()
            .filter(|(_, d)| match &d.scope {
                Scope::At(at) => {
                    let path = path(at);
                    path == place || under(path, place) || under(place, path)
                }
                _ => false,
            })
            .collect()
    }

    /// The live rules of the trail `id` of `project`, oldest first.
    pub fn rules(&self, project: &str, id: &str) -> Vec<(&'a Note, Decision)> {
        let mut rules: Vec<(&'a Note, Decision)> = self
            .live(project)
            .into_iter()
            .filter(|(_, d)| d.scope == Scope::Trail(id.to_string()))
            .collect();
        rules.sort_by(|(a, _), (b, _)| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        rules
    }
}

/// The file or folder of the place `at`, without a trailing `:<line>`.
pub(crate) fn path(at: &str) -> &str {
    match at.rsplit_once(':') {
        Some((path, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => path,
        _ => at,
    }
}

/// Whether the path `inner` lies in the folder `outer`.
fn under(inner: &str, outer: &str) -> bool {
    inner
        .strip_prefix(outer)
        .is_some_and(|rest| rest.starts_with('/'))
}
