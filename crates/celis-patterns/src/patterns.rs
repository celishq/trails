//! The patterns of a store.

use crate::{Cell, Cells, Pattern, PatternEnd, PatternRecord, View};
use celis_nostr::Note;
use celis_store::Refusal;
use std::collections::HashMap;

/// The patterns of a store, read once, newest first, then by id, and the cells that hold them.
pub struct Patterns<'a> {
    every: Vec<Pattern<'a>>,
    cells: Cells,
}

impl<'a> Patterns<'a> {
    pub fn new(records: &'a [Note]) -> Patterns<'a> {
        let read: Vec<(&'a Note, PatternRecord)> = records
            .iter()
            .filter_map(|n| Some((n, PatternRecord::read(n)?)))
            .collect();
        let by: HashMap<String, &'a Note> = read
            .iter()
            .filter_map(|(n, d)| Some((d.replaces.as_ref()?.to_string(), *n)))
            .collect();
        let drops = drops(records);
        let mut every: Vec<Pattern<'a>> = read
            .into_iter()
            .map(|(note, record)| Pattern {
                note,
                record,
                end: by
                    .get(note.id.as_str())
                    .copied()
                    .map(PatternEnd::Replaced)
                    .or(drops.get(note.id.as_str()).map(|d| PatternEnd::Dropped(d))),
            })
            .collect();
        every.sort_by(|a, b| (b.note.created_at, &a.note.id).cmp(&(a.note.created_at, &b.note.id)));
        Patterns {
            every,
            cells: Cells::new(records),
        }
    }

    pub fn cells(&self) -> &Cells {
        &self.cells
    }

    /// Every pattern of every project, live or ended.
    pub fn every(&self) -> &[Pattern<'a>] {
        &self.every
    }

    /// The patterns as the project `project` sees them.
    pub fn view<'p>(&'p self, project: &str) -> View<'p, 'a> {
        View::new(self, project)
    }

    /// The live pattern of any project whose id is `id`, or the only one whose id starts with `id` when
    /// `id` is 4 or more characters.
    pub fn find(&self, id: &str) -> Result<Pattern<'a>, Refusal> {
        find(self.every.iter().filter(|p| p.is_live()), id)
    }

    /// The cells that hold `pattern`: those of its account that name it, or a pattern of that account
    /// it replaces.
    pub fn holding(&self, pattern: &Pattern) -> Vec<&Cell> {
        let mut cells = self.cells.naming(pattern.id(), &pattern.note.pubkey);
        let replaced = self.replaced(pattern);
        if let Some(replaced) = replaced.filter(|r| r.note.pubkey == pattern.note.pubkey) {
            cells.extend(self.holding(replaced));
        }
        cells
    }

    /// The pattern that `pattern` replaces, when the store holds it.
    pub(crate) fn replaced(&self, pattern: &Pattern) -> Option<&Pattern<'a>> {
        let id = pattern.record.replaces.as_deref()?;
        self.every.iter().find(|p| p.id() == id)
    }
}

/// The live pattern of `live` whose id is `id`, or the only one whose id starts with `id` when `id` is 4
/// or more characters.
pub(crate) fn find<'p, 'a: 'p>(
    live: impl Iterator<Item = &'p Pattern<'a>>,
    id: &str,
) -> Result<Pattern<'a>, Refusal> {
    let live: Vec<&Pattern<'a>> = live.collect();
    let note = celis_store::records::id::find(live.iter().map(|p| p.note), id);
    let note = note.ok_or_else(|| Refusal::NoPattern(id.to_string()))?;
    let found = live.into_iter().find(|p| p.note.id == note.id);
    Ok(found.cloned().expect("the note is of a live pattern"))
}

/// The first drop of each record id that `records` holds, by when it was written, then by id: a kind
/// 1985 record with the label `drop` in `celis.step`, a reason, and an `e` tag (CIP-03, "Dropping").
fn drops(records: &[Note]) -> HashMap<&str, &Note> {
    let mut drops: HashMap<&str, &Note> = HashMap::new();
    let drop = |n: &Note| {
        let label = n.label("celis.step");
        n.kind == 1985 && label == Some("drop") && !n.content.trim().is_empty()
    };
    for note in records.iter().filter(|n| drop(n)) {
        let Some(id) = note.tag("e") else { continue };
        let first = drops.entry(id).or_insert(note);
        if (note.created_at, &note.id) < (first.created_at, &first.id) {
            *first = note;
        }
    }
    drops
}
