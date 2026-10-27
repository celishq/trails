//! The trails the graph view shows, in their project's folders.

use crate::command::trail::tree::today;
use celis_trails::{Step, Steps};
use std::cmp::Reverse;
use std::collections::HashMap;

/// Each trail of today in `steps`, a finished one too, grouped by project: each folder in the place its
/// oldest trail gives it, so a new trail never moves a folder, and in each folder the newest trail
/// first.
pub(super) fn folders<'a>(steps: &'a Steps<'a>) -> Vec<Step<'a>> {
    let mut trails = today(steps);
    let born = |s: &Step| (s.note().created_at, s.id().to_string());
    let mut oldest: HashMap<String, (u64, String)> = HashMap::new();
    for trail in &trails {
        let project = trail.record().anchor.project.clone();
        let entry = oldest.entry(project).or_insert_with(|| born(trail));
        *entry = entry.clone().min(born(trail));
    }
    trails.sort_by_key(|s| (oldest[&s.record().anchor.project].clone(), Reverse(born(s))));
    trails
}

#[cfg(test)]
mod tests;
