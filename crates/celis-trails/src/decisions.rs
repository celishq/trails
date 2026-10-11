//! The live decisions of a store.

use crate::{Decision, Failure, Scope};
use celis_wire::Note;

/// The decisions of a store.
pub struct Decisions<'a> {
    pub records: &'a [Note],
}

impl<'a> Decisions<'a> {
    /// The live decisions in scope in the project `project`: by scope, narrowest first, then newest
    /// first, then by id.
    pub fn live(&self, project: &str) -> Vec<(&'a Note, Decision)> {
        let counted: Vec<(&'a Note, Decision)> = self
            .records
            .iter()
            .filter_map(|n| Some((n, Decision::read(n)?)))
            .collect();
        let replaced: Vec<String> = counted
            .iter()
            .filter_map(|(_, d)| d.replaces.clone())
            .collect();
        let mut live: Vec<(&'a Note, Decision)> = counted
            .into_iter()
            .filter(|(_, d)| d.scope == Scope::Me || d.anchor.project == project)
            .filter(|(n, _)| !replaced.contains(&n.id))
            .collect();
        live.sort_by(|(a, x), (b, y)| {
            (x.scope.rank(), b.created_at, &a.id).cmp(&(y.scope.rank(), a.created_at, &b.id))
        });
        live
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
