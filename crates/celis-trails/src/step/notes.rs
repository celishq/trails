//! The notes of a store.

use crate::{Mark, Step};
use celis_nostr::Note;
use std::collections::HashMap;

/// The notes of a store by the step they are on, oldest first.
pub struct Notes<'a> {
    on: HashMap<&'a str, Vec<&'a Note>>,
}

impl<'a> Notes<'a> {
    pub fn new(records: &'a [Note]) -> Notes<'a> {
        let mut on: HashMap<&'a str, Vec<&'a Note>> = HashMap::new();
        for note in records {
            if let Some(step) = Notes::step(note) {
                on.entry(step).or_default().push(note);
            }
        }
        for notes in on.values_mut() {
            notes.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        }
        Notes { on }
    }

    /// The whole id of the step that `note` is on, when it is a note mark.
    pub fn step(note: &Note) -> Option<&str> {
        match Mark::read(note) {
            Some((_, Mark::Note { .. })) => note.tag("e"),
            _ => None,
        }
    }

    /// The notes on the step whose whole id is `id`.
    pub fn on(&self, id: &str) -> &[&'a Note] {
        self.on.get(id).map_or(&[], Vec::as_slice)
    }

    /// The notes of `trail`: those on each step of its tree, oldest first (CIP-04, "Notes").
    pub fn of(&self, trail: Step) -> Vec<&'a Note> {
        let tree = trail.tree().into_iter();
        let mut notes: Vec<&'a Note> = tree.flat_map(|s| self.on(s.id()).iter().copied()).collect();
        notes.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        notes
    }

    /// The note whose whole id is `id`.
    pub fn get(&self, id: &str) -> Option<&'a Note> {
        self.on.values().flatten().copied().find(|n| n.id == id)
    }
}

#[cfg(test)]
mod tests;
