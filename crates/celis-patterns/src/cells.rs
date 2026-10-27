//! The cells of a store.

use crate::{Cell, Mask, Membership};
use celis_nostr::Note;
use std::collections::{BTreeMap, BTreeSet};

/// The cells of a store, each with the ids its memberships name, and the masks that turn them on or off,
/// oldest first.
#[derive(Debug, Default)]
pub struct Cells {
    members: BTreeMap<Cell, BTreeSet<String>>,
    masks: Vec<Mask>,
}

impl Cells {
    pub fn new(records: &[Note]) -> Cells {
        let mut members: BTreeMap<Cell, BTreeSet<String>> = BTreeMap::new();
        for membership in records.iter().filter_map(Membership::read) {
            members
                .entry(membership.cell)
                .or_default()
                .extend(membership.patterns);
        }
        let mut masks: Vec<&Note> = records.iter().filter(|n| n.kind == 1985).collect();
        masks.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        let masks = masks.into_iter().filter_map(Mask::read).collect();
        Cells { members, masks }
    }

    /// Each cell, by name, then by account.
    pub fn all(&self) -> impl Iterator<Item = &Cell> {
        self.members.keys()
    }

    /// The cells named `name`.
    pub fn named(&self, name: &str) -> Vec<&Cell> {
        self.all().filter(|c| c.name == name).collect()
    }

    /// The cells of `author` whose memberships name `id`.
    pub fn naming(&self, id: &str, author: &str) -> Vec<&Cell> {
        let members = self.members.iter();
        let found = members.filter(|(c, ids)| c.author == author && ids.contains(id));
        found.map(|(c, _)| c).collect()
    }

    /// The newest mask of `cell` in the project `project`, or of every project with none.
    pub fn newest(&self, cell: &Cell, project: Option<&str>) -> Option<&Mask> {
        let mut masks = self.masks.iter().rev();
        masks.find(|m| m.cell == *cell && m.project() == project)
    }

    /// Whether `cell` is on in the project `project` (`spec/09-cells.md`, "Masks").
    pub fn is_on(&self, cell: &Cell, project: &str) -> bool {
        let mask = self.newest(cell, Some(project));
        mask.or_else(|| self.newest(cell, None))
            .is_some_and(|m| m.on)
    }
}

#[cfg(test)]
mod tests;
