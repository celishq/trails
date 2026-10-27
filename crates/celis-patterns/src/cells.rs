//! The cells of a store.

use crate::cell::MASK;
use crate::Cell;
use celis_nostr::Note;
use celis_store::Anchor;
use std::collections::{BTreeMap, BTreeSet};

/// The cells of a store, each with the ids its memberships name, and the masks that turn them on or off.
#[derive(Debug, Default)]
pub struct Cells {
    members: BTreeMap<Cell, BTreeSet<String>>,
    masks: Vec<Mask>,
}

/// A mask: the cell it turns on or off, and its project, none for every project.
#[derive(Debug)]
struct Mask {
    cell: Cell,
    on: bool,
    project: Option<String>,
}

impl Cells {
    pub fn new(records: &[Note]) -> Cells {
        let mut members: BTreeMap<Cell, BTreeSet<String>> = BTreeMap::new();
        for (cell, ids) in records.iter().filter_map(Cell::read) {
            let named = members.entry(cell).or_default();
            named.extend(ids.into_iter().map(str::to_string));
        }
        let mut masks: Vec<&Note> = records.iter().filter(|n| n.kind == 1985).collect();
        masks.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        let masks = masks.into_iter().filter_map(mask).collect();
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

    /// Whether `cell` is on in the project `project` (`spec/09-cells.md`, "Masks").
    pub fn is_on(&self, cell: &Cell, project: &str) -> bool {
        let newest = |of: Option<&str>| {
            let mut masks = self.masks.iter().rev();
            masks.find(|m| m.cell == *cell && m.project.as_deref() == of)
        };
        let mask = newest(Some(project)).or_else(|| newest(None));
        mask.is_some_and(|m| m.on)
    }
}

/// The mask `note` is; `None` when it is no mask.
fn mask(note: &Note) -> Option<Mask> {
    let label = note
        .tags
        .iter()
        .find(|t| t.len() >= 3 && t[0] == "l" && t[2] == MASK)?;
    let on = match label[1].as_str() {
        "on" => true,
        "off" => false,
        _ => return None,
    };
    let cell = Cell {
        name: note.tag("cell")?.to_string(),
        author: note.tag("p")?.to_string(),
    };
    let project = match note.tag("scope") {
        Some("me") => None,
        _ => Some(Anchor::read(note)?.project),
    };
    Some(Mask { cell, on, project })
}

#[cfg(test)]
mod tests;
