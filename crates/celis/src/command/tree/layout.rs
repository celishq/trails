//! The day groups a watch laid out, kept to print again.

use super::{Frame, View};
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};

/// The day groups a watch laid out from one model, by day, to print again while nothing they show
/// changed.
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

    /// Keeps `group` as the group of `day`.
    pub(super) fn keep(&self, day: String, group: Group) {
        self.groups.borrow_mut().insert(day, group);
    }
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
