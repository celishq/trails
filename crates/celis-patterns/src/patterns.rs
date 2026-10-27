//! The patterns of a store.

use crate::{Cell, Cells, Pattern, PatternEnd, PatternRecord, Scope};
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

    /// The live patterns in scope in the project `project`.
    pub fn live(&self, project: &str) -> Vec<Pattern<'a>> {
        let scoped = self.scoped(project).into_iter();
        scoped.filter(|p| p.is_live()).collect()
    }

    /// The patterns in scope in the project `project`, live or ended.
    pub fn scoped(&self, project: &str) -> Vec<Pattern<'a>> {
        let every = self.every.iter();
        every.filter(|p| self.holds(p, project)).cloned().collect()
    }

    /// The live pattern in scope in `project`, or of any project with `None`, whose id is `id`, or the only
    /// one whose id starts with `id` when `id` is 4 or more characters.
    pub fn find(&self, project: Option<&str>, id: &str) -> Result<Pattern<'a>, Refusal> {
        let live: Vec<&Pattern<'a>> = self
            .every
            .iter()
            .filter(|p| p.is_live())
            .filter(|p| project.is_none_or(|project| self.holds(p, project)))
            .collect();
        let note = celis_store::records::id::find(live.iter().map(|p| p.note), id);
        let note = note.ok_or_else(|| Refusal::NoPattern(id.to_string()))?;
        let found = live.into_iter().find(|p| p.note.id == note.id);
        Ok(found.cloned().expect("the note is of a live pattern"))
    }

    /// The live patterns in scope in `project` that match one of `paths`, by id.
    pub fn matching(&self, project: &str, paths: &[String]) -> Vec<Pattern<'a>> {
        let mut found: Vec<Pattern<'a>> = self
            .live(project)
            .into_iter()
            .filter(|p| p.record.matches(paths))
            .collect();
        found.sort_by_key(|p| p.id());
        found
    }

    /// The live patterns in scope in `project` whose `within` names `id`, newest first.
    pub fn smaller(&self, project: &str, id: &str) -> Vec<Pattern<'a>> {
        let live = self.live(project).into_iter();
        live.filter(|p| p.record.within.iter().any(|w| &**w == id))
            .collect()
    }

    /// Whether `pattern` is in the view of `project` (`spec/09-cells.md`, "The view").
    pub fn in_view(&self, pattern: &Pattern, project: &str) -> bool {
        let cells = self.holding(pattern);
        let off = !cells.is_empty() && cells.iter().all(|c| !self.cells.is_on(c, project));
        let replaced = pattern
            .record
            .replaces
            .as_deref()
            .and_then(|id| self.by_id(id));
        !off && replaced.is_none_or(|r| self.in_view(r, project))
    }

    /// The cells that hold `pattern`: those of its account that name it, or a pattern of that account
    /// it replaces.
    pub fn holding(&self, pattern: &Pattern) -> Vec<&Cell> {
        let author = pattern.note.pubkey.hex();
        let mut cells = self.cells.naming(pattern.id(), &author);
        let replaced = pattern
            .record
            .replaces
            .as_deref()
            .and_then(|id| self.by_id(id));
        if let Some(replaced) = replaced.filter(|r| r.note.pubkey == pattern.note.pubkey) {
            cells.extend(self.holding(replaced));
        }
        cells
    }

    fn by_id(&self, id: &str) -> Option<&Pattern<'a>> {
        self.every.iter().find(|p| p.id() == id)
    }

    /// Whether `pattern` holds in `project`: in the view, and in a cell that is on there or in its scope.
    fn holds(&self, pattern: &Pattern, project: &str) -> bool {
        let on = || {
            let mut cells = self.holding(pattern).into_iter();
            cells.any(|c| self.cells.is_on(c, project))
        };
        let scoped = pattern.record.scope == Scope::Me || pattern.record.anchor.project == project;
        self.in_view(pattern, project) && (scoped || on())
    }
}

/// The first drop of each record id that `records` holds, by when it was written, then by id: a kind
/// 1985 record with the label `drop` in `celis.step`, a reason, and an `e` tag (CIP-03, "Dropping").
fn drops(records: &[Note]) -> HashMap<&str, &Note> {
    let mut drops: HashMap<&str, &Note> = HashMap::new();
    let drop = |n: &Note| {
        let label = |t: &Vec<String>| t.len() >= 3 && t[0] == "l" && t[2] == "celis.step";
        let label = n.tags.iter().find(|t| label(t));
        n.kind == 1985 && label.is_some_and(|t| t[1] == "drop") && !n.content.trim().is_empty()
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
