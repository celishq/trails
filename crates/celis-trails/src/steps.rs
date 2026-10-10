//! The steps of a store, as they stand at one moment.

use crate::{Failure, Grade, Mark, Step};
use celis_wire::Note;

/// The step records and marks of a store: which steps are open, how they nest, and how each went.
pub struct Steps<'a> {
    pub records: &'a [Note],
}

impl<'a> Steps<'a> {
    /// Every step record, oldest first.
    pub fn all(&self) -> Vec<&'a Note> {
        let mut all: Vec<&'a Note> = self
            .records
            .iter()
            .filter(|n| Step::read(n).is_some())
            .collect();
        all.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        all
    }

    /// The step whose id is `id`, or the only record whose id starts with `id` when `id` is 8
    /// characters.
    pub fn find(&self, id: &str) -> Result<&'a Note, Failure> {
        crate::id::find(self.all(), id).ok_or_else(|| Failure::no_step(id))
    }

    /// The first done record with a proof naming the step `id`; `None` while it is open.
    pub fn done(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| match Mark::read(n) {
                Some((p, Mark::Done { proof, .. })) => p == id && !proof.trim().is_empty(),
                _ => false,
            })
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// The first start record naming the step `id`; `None` before one.
    pub fn started(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| matches!(Mark::read(n), Some((s, Mark::Start)) if s == id))
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// Whether the step `id` is `done`, `working` or `open`.
    pub fn status(&self, id: &str) -> &'static str {
        match (self.done(id), self.started(id)) {
            (Some(_), _) => "done",
            (None, Some(_)) => "working",
            (None, None) => "open",
        }
    }

    /// Every open step, oldest first.
    pub fn open(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| self.done(&n.id).is_none())
            .collect()
    }

    /// Every trail of the project `project`, oldest first.
    pub fn trails(&self, project: &str) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| {
                Step::read(n).is_some_and(|s| s.under.is_none() && s.anchor.project == project)
            })
            .collect()
    }

    /// The steps directly under the step `id`, oldest first.
    pub fn under(&self, id: &str) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.under.as_deref() == Some(id)))
            .collect()
    }

    /// The step `note` and every step under it, at any depth.
    pub fn tree(&self, note: &'a Note) -> Vec<&'a Note> {
        let mut tree = vec![note];
        let mut i = 0;
        while i < tree.len() {
            tree.extend(self.under(&tree[i].id));
            i += 1;
        }
        tree
    }

    /// The state of the trail `note`: active while open, else reached alone or reached with help, as a
    /// steer names a step of its tree.
    pub fn state(&self, note: &'a Note) -> &'static str {
        if self.done(&note.id).is_none() {
            return ACTIVE;
        }
        match self.tree(note).iter().any(|n| self.steers(&n.id) > 0) {
            true => "reached with help",
            false => "reached alone",
        }
    }

    /// How the work of the done step `note` went; `None` while it is open.
    pub fn grade(&self, note: &Note) -> Option<Grade> {
        let done = self.done(&note.id)?;
        let rework = self
            .all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.fixes.as_deref() == Some(&note.id)))
            .count();
        Some(Grade {
            took: done.created_at.saturating_sub(note.created_at),
            rework,
            steered: self.steers(&note.id),
        })
    }

    /// The steers with content that name the step `id`.
    pub fn steers(&self, id: &str) -> usize {
        self.records
            .iter()
            .filter(|n| matches!(Mark::read(n), Some((s, Mark::Steer { text })) if s == id && !text.trim().is_empty()))
            .count()
    }
}

/// The state of a trail that is still open.
pub const ACTIVE: &str = "active";

#[cfg(test)]
mod tests;
