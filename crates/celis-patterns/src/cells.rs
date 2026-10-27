//! The cells of a store.

use crate::Cell;
use celis_nostr::Note;
use std::collections::{BTreeMap, BTreeSet};

/// The cells of a store, each with the ids its memberships name.
#[derive(Debug, Default)]
pub struct Cells {
    members: BTreeMap<Cell, BTreeSet<String>>,
}

impl Cells {
    pub fn new(records: &[Note]) -> Cells {
        let mut members: BTreeMap<Cell, BTreeSet<String>> = BTreeMap::new();
        for (cell, ids) in records.iter().filter_map(Cell::read) {
            let named = members.entry(cell).or_default();
            named.extend(ids.into_iter().map(str::to_string));
        }
        Cells { members }
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
}
