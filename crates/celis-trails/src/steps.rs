//! The steps of a store, as they stand at one moment.

use crate::{Failure, Grade, Mark, State, Status, Step};
use celis_wire::Note;

/// The step records and marks of a store at the time `now`: which steps are open, how they nest, and how
/// each went.
pub struct Steps<'a> {
    pub records: &'a [Note],
    pub now: u64,
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

    /// The step `note` holds, with each of its edits, oldest first; `None` when it is not a step.
    pub fn step(&self, note: &Note) -> Option<Step> {
        let mut step = Step::read(note)?;
        let mut edits: Vec<(&Note, Mark)> = self
            .records
            .iter()
            .filter_map(|n| match Mark::read(n) {
                Some((id, edit @ Mark::Edit { .. })) if id == note.id => Some((n, edit)),
                _ => None,
            })
            .collect();
        edits.sort_by(|(a, _), (b, _)| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        for (_, edit) in edits {
            let Mark::Edit {
                text,
                places,
                check,
            } = edit
            else {
                continue;
            };
            step.content = match (
                text.trim().is_empty(),
                text.lines().count(),
                step.content.split_once('\n'),
            ) {
                (true, _, _) => step.content,
                (false, 1, Some((_, detail))) => format!("{}\n{detail}", text.trim_end()),
                (false, _, _) => text,
            };
            if !places.is_empty() {
                step.places = places;
            }
            step.check = check.or(step.check);
        }
        Some(step)
    }

    /// The step whose id is `id`, or the only record whose id starts with `id` when `id` is 4 or more
    /// characters.
    pub fn find(&self, id: &str) -> Result<&'a Note, Failure> {
        crate::id::find(self.all(), id).ok_or_else(|| Failure::no_step(id))
    }

    /// The step `find` gives for `id`; `closed` when a done or a drop names it.
    pub fn find_open(&self, id: &str) -> Result<&'a Note, Failure> {
        let note = self.find(id)?;
        match self.closed(&note.id) {
            Some(_) => Err(Failure::closed(&note.id)),
            None => Ok(note),
        }
    }

    /// The first done record with a proof, or drop record with a reason, naming the step `id`; `None`
    /// while it is open.
    pub fn closed(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| match Mark::read(n) {
                Some((p, Mark::Done { proof: text, .. }))
                | Some((p, Mark::Drop { reason: text })) => p == id && !text.trim().is_empty(),
                _ => false,
            })
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// The done record that closed the step `id`; `None` while it is open or when it was dropped.
    pub fn done(&self, id: &str) -> Option<&'a Note> {
        self.closed(id)
            .filter(|n| matches!(Mark::read(n), Some((_, Mark::Done { .. }))))
    }

    /// The drop record that closed the step `id`; `None` while it is open or when it was done.
    pub fn dropped(&self, id: &str) -> Option<&'a Note> {
        self.closed(id)
            .filter(|n| matches!(Mark::read(n), Some((_, Mark::Drop { .. }))))
    }

    /// The first start record naming the step `id`; `None` before one.
    pub fn started(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| matches!(Mark::read(n), Some((s, Mark::Start)) if s == id))
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// Whether the step `id` is done, dropped, working or open, by its marks alone.
    pub fn status(&self, id: &str) -> Status {
        match (self.done(id), self.dropped(id), self.started(id)) {
            (Some(_), _, _) => Status::Done,
            (None, Some(_), _) => Status::Dropped,
            (None, None, Some(_)) => Status::Working,
            (None, None, None) => Status::Open,
        }
    }

    /// Every open step whose trail has not ended, oldest first.
    pub fn open(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| self.closed(&n.id).is_none() && !self.ended(self.trail(n)))
            .collect()
    }

    /// Whether the trail `note` is open and ended: 30 minutes after the newest record of its tree.
    pub fn ended(&self, note: &'a Note) -> bool {
        self.closed(&note.id).is_none() && self.now.saturating_sub(self.newest(note)) >= LIFE
    }

    /// The time of the newest record of the tree of `note`: its steps, and the marks that name them.
    pub fn newest(&self, note: &'a Note) -> u64 {
        let tree: Vec<&str> = self.tree(note).iter().map(|n| n.id.as_str()).collect();
        self.records
            .iter()
            .filter(|n| {
                tree.contains(&n.id.as_str())
                    || Mark::read(n).is_some_and(|(id, _)| tree.contains(&id.as_str()))
            })
            .map(|n| n.created_at)
            .max()
            .unwrap_or(note.created_at)
    }

    /// Every trail of the project `project`, oldest first.
    pub fn trails(&self, project: &str) -> Vec<&'a Note> {
        self.roots()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.anchor.project == project))
            .collect()
    }

    /// Every trail of every project, oldest first.
    pub fn roots(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.under.is_none()))
            .collect()
    }

    /// The open trail of the session `session` in the project `project`: the trail of its newest step,
    /// when it is not done or dropped.
    pub fn session(&self, project: &str, session: &str) -> Option<&'a Note> {
        let newest = self.all().into_iter().rev().find(|n| {
            Step::read(n).is_some_and(|s| {
                s.anchor.project == project && s.session.as_deref() == Some(session)
            })
        })?;
        let trail = self.trail(newest);
        self.closed(&trail.id).is_none().then_some(trail)
    }

    /// The steps directly under the step `id`, oldest first.
    pub fn under(&self, id: &str) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.under.as_deref() == Some(id)))
            .collect()
    }

    /// The trail at the top of the `under` chain of the step `note`.
    pub fn trail(&self, note: &'a Note) -> &'a Note {
        let mut top = note;
        while let Some(up) = Step::read(top)
            .and_then(|s| s.under)
            .and_then(|id| self.find(&id).ok())
        {
            top = up;
        }
        top
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

    /// The state of the trail `note`: active while open, dropped when dropped, expired when it ended with
    /// an open step, else reached alone or reached with help, as a steer names a step of its tree.
    pub fn state(&self, note: &'a Note) -> State {
        if self.closed(&note.id).is_none() && !self.ended(note) {
            return State::Active;
        }
        let tree = self.tree(note);
        if self.closed(&note.id).is_none() && tree[1..].iter().any(|n| self.closed(&n.id).is_none())
        {
            return State::Expired;
        }
        if self.dropped(&note.id).is_some() {
            return State::Dropped;
        }
        match tree.iter().any(|n| self.steers(&n.id) > 0) {
            true => State::ReachedWithHelp,
            false => State::ReachedAlone,
        }
    }

    /// How the work of the done step `note` went; `None` while it is open.
    pub fn grade(&self, note: &Note) -> Option<Grade> {
        let done = self.done(&note.id)?;
        let from = self.started(&note.id).unwrap_or(note).created_at;
        let rework = self
            .all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.fixes.as_deref() == Some(&note.id)))
            .count();
        Some(Grade {
            took: done.created_at.saturating_sub(from),
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

/// The seconds a trail stays active after the newest record of its tree.
pub const LIFE: u64 = 30 * 60;

#[cfg(test)]
mod tests;
