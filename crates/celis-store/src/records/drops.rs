//! The drops that count on the records of a store.

use celis_nostr::Note;
use std::collections::HashMap;

/// The first drop of each record id, by when it was written, then by id: a kind 1985 record with the
/// label `drop` in `celis.step`, a reason, and an `e` tag (`spec/03-steps.md`, "Dropping").
pub struct Drops<'a> {
    first: HashMap<&'a str, &'a Note>,
}

impl<'a> Drops<'a> {
    /// The drops that `records` hold.
    pub fn of(records: &'a [Note]) -> Drops<'a> {
        let mut first: HashMap<&'a str, &'a Note> = HashMap::new();
        let drop = |n: &Note| {
            let label = n.label("celis.step");
            n.kind == 1985 && label == Some("drop") && !n.content.trim().is_empty()
        };
        for note in records.iter().filter(|n| drop(n)) {
            let Some(id) = note.tag("e") else { continue };
            let kept = first.entry(id).or_insert(note);
            if (note.created_at, &note.id) < (kept.created_at, &kept.id) {
                *kept = note;
            }
        }
        Drops { first }
    }

    /// The drop that counts on the record `id`.
    pub fn get(&self, id: &str) -> Option<&'a Note> {
        self.first.get(id).copied()
    }
}

#[cfg(test)]
mod tests;
