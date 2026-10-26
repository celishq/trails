//! The day groups a watch laid out, kept to print again across models.

use super::{Frame, View};
use crate::progress::Model;
use celis_nostr::Note;
use celis_trails::{Mark, StepRecord};
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};

/// The day groups a watch laid out, by day, to print again while nothing they show changed.
#[derive(Default)]
pub(crate) struct Layout {
    groups: RefCell<HashMap<String, Group>>,
}

/// A laid-out day group: what it was laid out for, the ids of its lines and of what they hold, the
/// lines it opened or closed and selected of those, and what it printed.
pub(super) struct Group {
    pub key: Key,
    pub ids: HashSet<String>,
    pub flipped: Vec<String>,
    pub selected: Option<String>,
    pub part: Frame,
}

/// What a day group is laid out for: its trails, the width, and whether full and all.
#[derive(PartialEq)]
pub(super) struct Key {
    pub trails: Vec<String>,
    pub width: Option<usize>,
    pub full: bool,
    pub all: bool,
}

impl Layout {
    /// The group of `day` laid out for `key`, when `view` at `now` would print it the same.
    pub(super) fn get(
        &self,
        day: &str,
        key: &Key,
        view: &View,
        now: u64,
    ) -> Option<Ref<'_, Group>> {
        let groups = self.groups.borrow();
        Ref::filter_map(groups, |groups| {
            let group = groups.get(day)?;
            let same = &group.key == key
                && group.part.next.is_none_or(|next| now < next)
                && group.flipped == flipped(&group.ids, view)
                && group.selected == selected(&group.ids, view);
            same.then_some(group)
        })
        .ok()
    }

    /// Forgets the groups that show the trail the records `landed` change in `model`, or every group
    /// when `landed` is `None` or `touched` finds no one trail.
    pub(crate) fn renew(&self, model: &Model, landed: Option<&[Note]>) {
        let mut groups = self.groups.borrow_mut();
        match landed.and_then(|notes| touched(model, notes)) {
            Some(trails) => groups.retain(|_, g| !g.key.trails.iter().any(|t| trails.contains(t))),
            None => groups.clear(),
        }
    }

    /// Keeps `group` as the group of `day`.
    pub(super) fn keep(&self, day: String, group: Group) {
        self.groups.borrow_mut().insert(day, group);
    }
}

/// The trail the records `notes` change in `model`, at most one: each is a step or a mark of a step of
/// it, and uses nothing. `None` when a record does otherwise.
fn touched(model: &Model, notes: &[Note]) -> Option<HashSet<String>> {
    let mut trails = HashSet::new();
    for note in notes {
        let ids = match (StepRecord::read(note), Mark::read(note)) {
            (Some(step), _) if step.uses.is_empty() => {
                let named = [step.under, step.replaces].into_iter().flatten();
                std::iter::once(note.id.clone())
                    .chain(named.map(|id| id.to_string()))
                    .collect()
            }
            (None, Some((id, _))) => vec![id],
            _ => return None,
        };
        for id in ids {
            let step = model.steps.get(&id)?;
            trails.insert(step.trail().id().to_string());
        }
    }
    (trails.len() <= 1).then_some(trails)
}

/// The lines of `ids` that `view` opened or closed, in order.
pub(super) fn flipped(ids: &HashSet<String>, view: &View) -> Vec<String> {
    let mut flipped: Vec<String> = view
        .flipped
        .iter()
        .filter(|id| ids.contains(*id))
        .cloned()
        .collect();
    flipped.sort();
    flipped
}

/// The line of `ids` that `view` selected.
pub(super) fn selected(ids: &HashSet<String>, view: &View) -> Option<String> {
    view.selected.clone().filter(|id| ids.contains(id))
}

#[cfg(test)]
mod tests;
